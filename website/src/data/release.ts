// The newest release, read from the download host's latest.json at build time.
// The Release workflow triggers a site rebuild after publishing, so this is
// always the version users get.

/** Where packages, signatures and the update manifest live (Cloudflare R2). */
export const DL_BASE = (import.meta.env.QD_DL_BASE as string | undefined) ?? "https://dl.quickdesk.click";
// Tried in order; the r2.dev address works before dl.quickdesk.click is set up.
const SOURCES = [DL_BASE, "https://pub-c87aab3360aa43119ac701dbe62a3688.r2.dev"];

export interface ReleaseFile {
  name: string;
  url: string;
  sha256: string;
  size: number;
}

export interface Release {
  version: string;
  date: string;
  /** Absolute download URLs, sizes and checksums per package type. */
  files: Record<"deb" | "rpm" | "appimage", ReleaseFile & { href: string; sizeText: string }> &
    /** From the first release with a Windows build on. */
    Partial<Record<"windows", ReleaseFile & { href: string; sizeText: string }>>;
  checksums: string;
}

const mb = (n: number) => `${(n / 1024 / 1024).toFixed(1)} MB`;

async function load(): Promise<Release> {
  for (const base of SOURCES) {
    try {
      const res = await fetch(`${base}/latest.json`);
      if (!res.ok) continue;
      const data = await res.json();
      const files = Object.fromEntries(
        Object.entries(data.files as Record<string, ReleaseFile>).map(([kind, f]) => [
          kind,
          { ...f, href: `${base}/${f.url}`, sizeText: mb(f.size) },
        ]),
      ) as Release["files"];
      return {
        version: data.version,
        date: String(data.publishedAt).slice(0, 10),
        files,
        checksums: `${base}/releases/${data.version}/SHA256SUMS`,
      };
    } catch {
      // Try the next source.
    }
  }
  throw new Error("latest.json is not reachable from any download host; cannot build the site");
}

export const release: Release = await load();
