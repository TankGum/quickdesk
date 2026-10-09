// QuickDesk Cloud sync.
//
// A deliberately dumb object store: each sync account gets a private
// namespace in one R2 bucket, reachable with the account's token. The app
// encrypts everything before upload (XChaCha20-Poly1305, key derived from the
// user's passphrase), so this Worker only ever sees ciphertext, key names and
// sizes. No accounts in the usual sense: no email, no profile. An account is
// a random id plus a random token, which the app shows as the "sync code".
//
// API (all JSON errors are { error }):
//   POST   /v1/accounts                    → 201 { account, token }
//   GET    /v1/:account                    → 200 { rev, bytes, objects }; `rev` changes on every write
//                                            or delete, so an idle app checks it instead of listing
//   GET    /v1/:account/objects?prefix=&start_after=
//                                          → 200 { objects: [{ key, uploaded }] }  (uploaded: unix ms)
//   GET    /v1/:account/objects/:key       → 200 bytes | 404
//   PUT    /v1/:account/objects/:key       → 200 | 412 if `If-None-Match: *` and it exists
//   DELETE /v1/:account/objects/:key       → 204
//   DELETE /v1/:account                    → 204, deletes the account and all its data
// Everything except POST /v1/accounts needs `Authorization: Bearer <token>`.

export interface Env {
  SYNC: R2Bucket;
  CREATE_LIMIT?: RateLimit;
}

/** Limits per account, generous for notes (text, compacted logs). */
const MAX_OBJECT_BYTES = 8 * 1024 * 1024;
const MAX_ACCOUNT_BYTES = 256 * 1024 * 1024;
const MAX_OBJECTS = 20_000;
const KEY = /^[A-Za-z0-9._-]+(\/[A-Za-z0-9._-]+)*$/;
const ACCOUNT = /^[a-z0-9]{16}$/;

interface Meta {
  tokenHash: string;
  created: number;
  bytes: number;
  objects: number;
  /** New random value on every write or delete (absent on accounts from before it existed). */
  rev?: string;
}

const json = (body: unknown, status = 200) =>
  new Response(JSON.stringify(body), { status, headers: { "content-type": "application/json", "cache-control": "no-store" } });
const fail = (status: number, error: string) => json({ error }, status);

const metaKey = (account: string) => `accounts/${account}/meta.json`;
const objectRoot = (account: string) => `accounts/${account}/o/`;

function randomHex(bytes: number): string {
  const b = crypto.getRandomValues(new Uint8Array(bytes));
  return [...b].map((x) => x.toString(16).padStart(2, "0")).join("");
}

/** Lowercase base32 (RFC 4648 alphabet) for the account id: easy to read aloud. */
function randomId(): string {
  const alphabet = "abcdefghijklmnopqrstuvwxyz234567";
  const b = crypto.getRandomValues(new Uint8Array(16));
  return [...b].map((x) => alphabet[x % 32]).join("");
}

async function sha256(text: string): Promise<string> {
  const d = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(text));
  return [...new Uint8Array(d)].map((x) => x.toString(16).padStart(2, "0")).join("");
}

function sameString(a: string, b: string): boolean {
  if (a.length !== b.length) return false;
  let diff = 0;
  for (let i = 0; i < a.length; i++) diff |= a.charCodeAt(i) ^ b.charCodeAt(i);
  return diff === 0;
}

async function readMeta(env: Env, account: string): Promise<Meta | null> {
  const obj = await env.SYNC.get(metaKey(account));
  return obj ? ((await obj.json()) as Meta) : null;
}

// A random value rather than a counter: two writes racing on meta.json must
// not both produce the value a reader already saw.
async function writeMeta(env: Env, account: string, meta: Meta) {
  meta.rev = randomHex(8);
  await env.SYNC.put(metaKey(account), JSON.stringify(meta), { httpMetadata: { contentType: "application/json" } });
}

async function authorize(req: Request, env: Env, account: string): Promise<Meta | Response> {
  if (!ACCOUNT.test(account)) return fail(404, "no such sync account");
  const token = (req.headers.get("authorization") ?? "").replace(/^Bearer\s+/i, "");
  const meta = await readMeta(env, account);
  if (!meta) return fail(404, "no such sync account");
  if (!token || !sameString(await sha256(token), meta.tokenHash)) return fail(401, "wrong sync code");
  return meta;
}

async function createAccount(req: Request, env: Env): Promise<Response> {
  const ip = req.headers.get("cf-connecting-ip") ?? "unknown";
  if (env.CREATE_LIMIT && !(await env.CREATE_LIMIT.limit({ key: ip })).success) {
    return fail(429, "too many new sync accounts from this address; try again in a minute");
  }
  for (let attempt = 0; attempt < 5; attempt++) {
    const account = randomId();
    if (await env.SYNC.head(metaKey(account))) continue;
    const token = randomHex(16);
    await writeMeta(env, account, { tokenHash: await sha256(token), created: Date.now(), bytes: 0, objects: 0 });
    return json({ account, token }, 201);
  }
  return fail(503, "could not create a sync account; try again");
}

async function list(env: Env, account: string, url: URL): Promise<Response> {
  const root = objectRoot(account);
  const prefix = url.searchParams.get("prefix") ?? "";
  const startAfter = url.searchParams.get("start_after");
  const objects: { key: string; uploaded: number }[] = [];
  let cursor: string | undefined;
  do {
    const page = await env.SYNC.list({
      prefix: root + prefix,
      cursor,
      startAfter: startAfter ? root + startAfter : undefined,
    });
    for (const o of page.objects) objects.push({ key: o.key.slice(root.length), uploaded: o.uploaded.getTime() });
    cursor = page.truncated ? page.cursor : undefined;
  } while (cursor);
  objects.sort((a, b) => (a.key < b.key ? -1 : a.key > b.key ? 1 : 0));
  return json({ objects });
}

async function put(req: Request, env: Env, account: string, meta: Meta, key: string): Promise<Response> {
  const body = await req.arrayBuffer();
  if (body.byteLength > MAX_OBJECT_BYTES) return fail(413, "object too large");
  const full = objectRoot(account) + key;
  const existing = await env.SYNC.head(full);
  if (req.headers.get("if-none-match") === "*") {
    if (existing) return fail(412, "already exists");
  }
  const grow = body.byteLength - (existing?.size ?? 0);
  const more = existing ? 0 : 1;
  if (meta.bytes + grow > MAX_ACCOUNT_BYTES || meta.objects + more > MAX_OBJECTS) {
    return fail(507, "sync storage for this account is full");
  }
  const written = await env.SYNC.put(full, body, {
    // A concurrent create between the head() above and here still loses.
    onlyIf: req.headers.get("if-none-match") === "*" ? new Headers({ "if-none-match": "*" }) : undefined,
  });
  if (!written) return fail(412, "already exists");
  meta.bytes = Math.max(0, meta.bytes + grow);
  meta.objects += more;
  await writeMeta(env, account, meta);
  return json({ ok: true });
}

async function remove(env: Env, account: string, meta: Meta, key: string): Promise<Response> {
  const full = objectRoot(account) + key;
  const existing = await env.SYNC.head(full);
  if (existing) {
    await env.SYNC.delete(full);
    meta.bytes = Math.max(0, meta.bytes - existing.size);
    meta.objects = Math.max(0, meta.objects - 1);
    await writeMeta(env, account, meta);
  }
  return new Response(null, { status: 204 });
}

async function deleteAccount(env: Env, account: string): Promise<Response> {
  let cursor: string | undefined;
  do {
    const page = await env.SYNC.list({ prefix: `accounts/${account}/`, cursor });
    if (page.objects.length) await env.SYNC.delete(page.objects.map((o) => o.key));
    cursor = page.truncated ? page.cursor : undefined;
  } while (cursor);
  return new Response(null, { status: 204 });
}

export default {
  async fetch(req: Request, env: Env): Promise<Response> {
    const url = new URL(req.url);
    const parts = url.pathname.split("/").filter(Boolean);
    try {
      if (parts[0] !== "v1") {
        return url.pathname === "/" ? json({ service: "QuickDesk Cloud sync", api: "v1" }) : fail(404, "not found");
      }
      if (parts[1] === "accounts" && parts.length === 2) {
        return req.method === "POST" ? createAccount(req, env) : fail(405, "method not allowed");
      }
      const account = parts[1] ?? "";
      const auth = await authorize(req, env, account);
      if (auth instanceof Response) return auth;
      if (parts.length === 2) {
        if (req.method === "GET") return json({ rev: auth.rev ?? "", bytes: auth.bytes, objects: auth.objects });
        return req.method === "DELETE" ? deleteAccount(env, account) : fail(405, "method not allowed");
      }
      if (parts[2] !== "objects") return fail(404, "not found");
      if (parts.length === 3) return req.method === "GET" ? list(env, account, url) : fail(405, "method not allowed");
      const key = decodeURIComponent(parts.slice(3).join("/"));
      if (!KEY.test(key) || key.length > 512 || key.split("/").includes("..")) return fail(400, "bad object key");
      switch (req.method) {
        case "GET": {
          const obj = await env.SYNC.get(objectRoot(account) + key);
          return obj
            ? new Response(obj.body, { headers: { "content-type": "application/octet-stream", "cache-control": "no-store" } })
            : fail(404, "not found");
        }
        case "PUT":
          return put(req, env, account, auth, key);
        case "DELETE":
          return remove(env, account, auth, key);
        default:
          return fail(405, "method not allowed");
      }
    } catch (e) {
      console.error(e);
      return fail(500, "internal error");
    }
  },
} satisfies ExportedHandler<Env>;
