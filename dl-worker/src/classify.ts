// What a request to dl.quickdesk.click is, for the download counts: which
// file (kind, system, version) and where it came from. Pure, so it can be
// tested without Cloudflare.

export type Kind = "install" | "update" | "check" | "latest" | "sums" | "signature" | "other";
export type Source = "website" | "terminal" | "powershell" | "updater" | "github" | "direct" | "bot" | "other";

export interface Download {
  kind: Kind;
  /** linux-deb, linux-rpm, linux-appimage, windows, macos-dmg, macos-app, or "" */
  system: string;
  /** "0.6.0", or "" for files outside a release */
  version: string;
  source: Source;
}

const PACKAGES: [RegExp, string][] = [
  [/_amd64\.deb$/, "linux-deb"],
  [/\.x86_64\.rpm$/, "linux-rpm"],
  [/_amd64\.AppImage$/, "linux-appimage"],
  [/-setup\.exe$/, "windows"],
  [/_aarch64\.dmg$/, "macos-dmg"],
  [/\.app\.tar\.gz$/, "macos-app"],
];

/** Who asked, from the User-Agent and Referer headers. */
export function source(userAgent: string, referer: string): Source {
  const ua = userAgent.toLowerCase();
  if (ua.startsWith("tauri-plugin-updater/")) return "updater";
  if (/bot|crawl|spider|slurp|preview|monitor|uptime/.test(ua)) return "bot";
  if (ua.startsWith("curl/") || ua.startsWith("wget/")) return "terminal";
  if (ua.includes("powershell")) return "powershell";
  let host = "";
  try {
    host = referer ? new URL(referer).hostname : "";
  } catch {
    /* not a URL: treat as none */
  }
  if (host === "quickdesk.click" || host.endsWith(".quickdesk.click")) return "website";
  if (host === "github.com" || host.endsWith(".github.com")) return "github";
  if (ua.startsWith("mozilla/")) return "direct";
  return "other";
}

/** The file asked for, and what that means given who asked. */
export function classify(path: string, userAgent: string, referer: string): Download {
  const from = source(userAgent, referer);
  if (path === "/update.json") return { kind: "check", system: "", version: "", source: from };
  if (path === "/latest.json") return { kind: "latest", system: "", version: "", source: from };
  const m = path.match(/^\/releases\/(\d+\.\d+\.\d+)\/([^/]+)$/);
  if (!m) return { kind: "other", system: "", version: "", source: from };
  const [, version, file] = m;
  if (file === "SHA256SUMS") return { kind: "sums", system: "", version, source: from };
  const signature = file.endsWith(".sig");
  const name = signature ? file.slice(0, -4) : file;
  const system = PACKAGES.find(([re]) => re.test(name))?.[1] ?? "";
  if (signature) return { kind: "signature", system, version, source: from };
  if (!system) return { kind: "other", system: "", version, source: from };
  // The app's updater downloads the same packages people install by hand.
  return { kind: from === "updater" ? "update" : "install", system, version, source: from };
}

/**
 * Whether a response is one download: a GET that got the file, counting a
 * resumed download (a Range that does not start at 0) only once.
 */
export function counts(method: string, status: number, range: string | null): boolean {
  if (method !== "GET") return false;
  if (status === 200) return true;
  return status === 206 && (!range || /^bytes=0-/.test(range));
}
