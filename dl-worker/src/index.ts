// Download counts for dl.quickdesk.click. The Worker sits on the host's route
// and passes every request through to the R2 bucket unchanged (fetch() to the
// same host goes to the origin; any error here also falls through to it), then
// adds one to a daily total in D1 (table `downloads`, see migrations/): the
// file's kind, system and version, where the request came from, and the
// country Cloudflare reports. No IP address, User-Agent string or other
// identifier is stored, and no row per request. The counter is written after
// the response has gone out, so it never slows a download.
// Read the counts in the dashboard (D1 → quickdesk-downloads) or with
// `npm run stats` (scripts/downloads.py).
import { classify, counts } from "./classify";

export interface Env {
  DB: D1Database;
}

const COUNT =
  "INSERT INTO downloads (day, kind, system, version, source, country, n) VALUES (?, ?, ?, ?, ?, ?, 1) " +
  "ON CONFLICT (day, kind, system, version, source, country) DO UPDATE SET n = n + 1";

export default {
  async fetch(request, env, ctx): Promise<Response> {
    ctx.passThroughOnException();
    const response = await fetch(request);
    try {
      if (counts(request.method, response.status, request.headers.get("range"))) {
        const url = new URL(request.url);
        const d = classify(url.pathname, request.headers.get("user-agent") ?? "", request.headers.get("referer") ?? "");
        const country = (request.cf?.country as string | undefined) ?? "";
        const day = new Date().toISOString().slice(0, 10);
        ctx.waitUntil(
          env.DB.prepare(COUNT)
            .bind(day, d.kind, d.system, d.version, d.source, country)
            .run()
            .catch(() => undefined),
        );
      }
    } catch {
      /* counting must never get in the way of a download */
    }
    return response;
  },
} satisfies ExportedHandler<Env>;
