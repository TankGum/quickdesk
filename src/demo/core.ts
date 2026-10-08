// Demo build only: replaces `@tauri-apps/api/core`. Every command the UI calls
// is answered from sample data kept in memory, so the real React UI runs in a
// browser (the landing page embeds it). Nothing here ships in the app.
import type {
  ClipEntry,
  ClipKind,
  ClipStatus,
  HotkeyConfig,
  Note,
  PortEntry,
  SyncStatus,
  UsageSnapshot,
} from "../shared/ipc";
import { emit, params, toPage } from "./bus";

const now = Date.now();
const MIN = 60_000;
const HOUR = 60 * MIN;
const DAY = 24 * HOUR;

// ---------- Notes ----------

let notes: Note[] = [
  {
    id: "n1",
    title: "Deploy checklist",
    body: "- [x] Bump version to 0.2.0\n- [x] Run migrations on staging\n- [ ] Smoke-test clipboard images\n- [ ] Tag release & publish the .deb",
    pinned: true,
    createdAt: now - 3 * DAY,
    updatedAt: now - 2 * MIN,
    conflictOf: null,
  },
  {
    id: "n2",
    title: "Standup — Tuesday",
    body: "- Shipped clipboard images and files\n- Reviewing the release flow\n- No blockers",
    pinned: false,
    createdAt: now - DAY,
    updatedAt: now - 5 * HOUR,
    conflictOf: null,
  },
  {
    id: "n3",
    title: "Ý tưởng cho v0.3",
    body: "Ghi chú nhanh từ thanh trên cùng, chia sẻ ghi chú bằng link, bản cho Windows.",
    pinned: false,
    createdAt: now - 2 * DAY,
    updatedAt: now - 26 * HOUR,
    conflictOf: null,
  },
  {
    id: "n4",
    title: "Server IPs",
    body: "staging `10.0.0.12`\nprod `10.0.1.4` (ssh via bastion)",
    pinned: false,
    createdAt: now - 9 * DAY,
    updatedAt: now - 4 * DAY,
    conflictOf: null,
  },
];
let trash: Note[] = [];

const fold = (s: string) => s.normalize("NFD").replace(/\p{M}/gu, "").replace(/đ/g, "d").replace(/Đ/g, "D").toLowerCase();
const sortNotes = (list: Note[]) => [...list].sort((a, b) => Number(b.pinned) - Number(a.pinned) || b.updatedAt - a.updatedAt);

// ---------- Clipboard ----------

const thumb =
  "data:image/svg+xml;utf8," +
  encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="160" height="90"><defs><radialGradient id="a" cx=".85" cy="1.1" r=".8"><stop offset="0" stop-color="#e95420"/><stop offset="1" stop-color="#e95420" stop-opacity="0"/></radialGradient><radialGradient id="b" cx="0" cy="0" r=".9"><stop offset="0" stop-color="#6a2c5b"/><stop offset="1" stop-color="#6a2c5b" stop-opacity="0"/></radialGradient></defs><rect width="160" height="90" fill="#1a0614"/><rect width="160" height="90" fill="url(#b)"/><rect width="160" height="90" fill="url(#a)"/><rect x="18" y="16" width="70" height="44" rx="5" fill="#fff" fill-opacity=".12"/></svg>`,
  );

let clipId = 10;
const clip = (kind: ClipKind, preview: string, ago: number, extra: Partial<ClipEntry> = {}): ClipEntry => ({
  id: ++clipId,
  kind,
  preview,
  chars: preview.length,
  pinned: false,
  sourceApp: null,
  firstCopiedAt: now - ago - HOUR,
  lastCopiedAt: now - ago,
  copyCount: 1,
  byteSize: preview.length,
  width: null,
  height: null,
  thumb: null,
  ...extra,
});

let clips: ClipEntry[] = [
  clip("text", "docker compose up -d --build", 20_000, { copyCount: 4 }),
  clip("image", "", 4 * MIN, { byteSize: 188_416, width: 1280, height: 720, thumb }),
  clip("files", "/home/me/Projects/quickdesk/release-notes.pdf\n/home/me/Projects/quickdesk/mockup.png\n/home/me/Projects/quickdesk/assets", 9 * MIN),
  clip("text", "Họp dời sang 15:30, phòng 4B", 12 * MIN, { copyCount: 3 }),
  clip("text", "https://github.com/TankGum/quickdesk", 25 * MIN),
  clip("text", "SELECT id, title FROM notes\nWHERE deleted_at IS NULL\nORDER BY updated_at DESC\nLIMIT 20;", 50 * MIN),
  clip("text", "ssh deploy@10.0.0.12", 3 * DAY, { pinned: true, copyCount: 27 }),
];

const clipStatus: ClipStatus = {
  backend: "x11",
  state: "running",
  detail: null,
  paused: false,
  pausedUntil: null,
  autoPaste: true,
};

function clipQuery(query: string, limit = 50, kind?: ClipKind) {
  const q = fold(query.trim());
  return clips
    .filter((c) => (!kind || c.kind === kind) && (!q || fold(c.preview).includes(q)))
    .sort((a, b) => Number(b.pinned) - Number(a.pinned) || b.lastCopiedAt - a.lastCopiedAt)
    .slice(0, limit);
}

// ---------- Ports ----------

let ports: PortEntry[] = [
  { port: 3000, addrs: ["127.0.0.1"], pid: 48211, process: "node", cmdline: "node node_modules/.bin/vite --port 3000", user: "me", container: null },
  {
    port: 5432,
    addrs: ["0.0.0.0", "::"],
    pid: null,
    process: null,
    cmdline: null,
    user: null,
    container: { id: "c0ffee", name: "postgres-dev", image: "postgres:16", privatePort: 5432 },
  },
  { port: 6379, addrs: ["127.0.0.1"], pid: 1187, process: "redis-server", cmdline: "/usr/bin/redis-server 127.0.0.1:6379", user: "redis", container: null },
  { port: 8080, addrs: ["::"], pid: 50277, process: "java", cmdline: "java -jar build/libs/api-0.0.1-SNAPSHOT.jar", user: "me", container: null },
  { port: 9229, addrs: ["127.0.0.1"], pid: 48215, process: "node", cmdline: "node --inspect server.js", user: "me", container: null },
];
const dead = new Set<number>();

// ---------- AI usage ----------

const usage: UsageSnapshot = {
  providers: [
    {
      provider: "claude",
      name: "Claude",
      plan: "Max",
      source: "live",
      asOf: now,
      error: null,
      windows: [
        { id: "five_hour", label: "5h", usedPercent: 42, resetsAt: now + 2 * HOUR + 10 * MIN, detail: null },
        { id: "weekly", label: "weekly", usedPercent: 18, resetsAt: now + 4 * DAY + 6 * HOUR, detail: null },
      ],
    },
    {
      provider: "codex",
      name: "Codex",
      plan: "Plus",
      source: "local",
      asOf: now - 12 * MIN,
      error: null,
      windows: [
        { id: "five_hour", label: "5h", usedPercent: 81, resetsAt: now + 38 * MIN, detail: null },
        { id: "weekly", label: "weekly", usedPercent: 27, resetsAt: now + 5 * DAY + 2 * HOUR, detail: null },
      ],
    },
    {
      provider: "antigravity",
      name: "Antigravity",
      plan: null,
      source: "local",
      asOf: now - 3 * DAY,
      error: null,
      windows: [{ id: "quota_hit", label: "quota reached", usedPercent: null, resetsAt: null, detail: null }],
    },
  ],
  updatedAt: now,
  headline: 81,
  ringShows: { provider: "codex", window: "five_hour" },
  ring: null,
  trayEnabled: true,
};

// ---------- Settings ----------

const hotkeys: HotkeyConfig = { notes: "Super+Alt+N", quickNote: "", clipboard: "Super+Alt+V", ports: "Super+Alt+P" };

const sync: SyncStatus = {
  state: "idle",
  lastSyncAt: now - 40_000,
  lastError: null,
  lastReport: { pulled: 1, pushed: 2, conflicts: 0, compacted: false },
  config: { endpoint: "https://<account>.r2.cloudflarestorage.com", bucket: "my-notes", region: "auto", accessKeyId: "••••••••", prefix: "" },
};

// ---------- Commands ----------

type Args = Record<string, any>; // eslint-disable-line @typescript-eslint/no-explicit-any

const changed = (event: string) => setTimeout(() => emit(event), 0);

const commands: Record<string, (a: Args) => unknown> = {
  app_get_language: () => ({ pref: "auto", resolved: params.get("lang") === "vi" ? "vi" : "en" }),
  app_set_language: (a) => ({ pref: a.language, resolved: a.language === "vi" ? "vi" : "en" }),
  app_onboarding: () => ({ done: true, offerUinput: false }),
  app_info: () => ({
    version: "0.2.0",
    os: "Ubuntu 24.04",
    wayland: true,
    desktop: "GNOME",
    hotkeyStrategy: "GnomeKeybinding",
    hotkeys,
    deviceId: "0192a7c4-demo",
    dataDir: "~/.local/share/io.github.tankgum.quickdesk",
  }),
  app_autostart_get: () => true,
  app_autostart_set: (a) => a.enabled,
  app_show: () => undefined,
  diag_focus_stats: () => ({ total: 0, focused: 0, latencyP50: null, latencyMax: null, recent: [] }),

  notes_list: () => sortNotes(notes),
  notes_search: (a) => sortNotes(notes.filter((n) => fold(`${n.title}\n${n.body}`).includes(fold(a.query)))),
  notes_create: (a) => {
    const n: Note = { id: `n${Date.now()}`, title: a.title ?? "", body: a.body, pinned: false, createdAt: Date.now(), updatedAt: Date.now(), conflictOf: null };
    notes.push(n);
    changed("notes://changed");
    return n;
  },
  notes_update: (a) => {
    const n = notes.find((x) => x.id === a.id)!;
    Object.assign(n, Object.fromEntries(Object.entries(a).filter(([k, v]) => k !== "id" && v !== undefined)), { updatedAt: Date.now() });
    changed("notes://changed");
    return n;
  },
  notes_delete: (a) => {
    trash = trash.concat(notes.filter((n) => n.id === a.id));
    notes = notes.filter((n) => n.id !== a.id);
    changed("notes://changed");
  },
  notes_restore: (a) => {
    const n = trash.find((x) => x.id === a.id)!;
    notes.push(n);
    changed("notes://changed");
    return n;
  },

  clip_list: (a) => clipQuery("", a.limit, a.kind),
  clip_search: (a) => clipQuery(a.query, a.limit, a.kind),
  clip_status: () => clipStatus,
  clip_copy: () => toPage("toast", { text: "Copied to clipboard" }),
  clip_paste: () => {
    toPage("hide", { label: "clip-popup" });
    toPage("toast", { text: "Pasted into the app you were using" });
  },
  clip_pin: (a) => {
    const c = clips.find((x) => x.id === a.id)!;
    c.pinned = a.pinned;
    changed("clipboard://changed");
    return c;
  },
  clip_delete: (a) => {
    clips = clips.filter((c) => c.id !== a.id);
    changed("clipboard://changed");
  },
  clip_clear: (a) => {
    const before = clips.length;
    clips = clips.filter((c) => a.keepPinned && c.pinned);
    changed("clipboard://changed");
    return before - clips.length;
  },
  clip_set_auto_paste: (a) => {
    clipStatus.autoPaste = a.enabled;
    changed("clipboard://changed");
  },
  clip_set_paused: (a) => {
    clipStatus.paused = a.paused;
    clipStatus.pausedUntil = a.paused && a.minutes ? Date.now() + a.minutes * MIN : null;
    changed("clipboard://changed");
  },
  clip_paste_info: () => ({ method: "auto", effective: "portal", uinputAvailable: true, ruleInstalled: false, canEnable: true, setupCommand: null }),
  clipboard_write: () => undefined,

  ports_scan: () => ports.filter((p) => p.pid === null || !dead.has(p.pid)),
  ports_kill: (a) => {
    dead.add(a.pid);
  },
  ports_is_alive: (a) => !dead.has(a.pid),
  ports_stop_container: (a) => {
    ports = ports.filter((p) => p.container?.id !== a.id);
  },
  ports_open: (a) => toPage("toast", { text: `Opening http://localhost:${a.port}` }),

  ai_usage_get: () => usage,
  ai_usage_refresh: () => ({ ...usage, updatedAt: Date.now() }),
  ai_set_tray: (a) => {
    usage.trayEnabled = a.enabled;
    changed("ai://usage");
  },
  ai_set_ring: (a) => {
    usage.ring = a.choice;
    if (a.choice) usage.ringShows = a.choice;
    changed("ai://usage");
  },

  sync_status: () => sync,
  sync_now: () => {
    sync.lastSyncAt = Date.now();
    emit("sync://status", sync);
  },

  hotkeys_get: () => ({ config: hotkeys, strategy: "GnomeKeybinding" }),
};

export async function invoke<T>(cmd: string, args: Args = {}): Promise<T> {
  const fn = commands[cmd];
  // Unknown commands (settings the demo cannot perform) just succeed.
  return (fn ? fn(args) : undefined) as T;
}
