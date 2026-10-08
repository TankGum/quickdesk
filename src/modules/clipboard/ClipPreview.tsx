import { Icon, IconName } from "../../shared/Icon";
import { t } from "../../shared/i18n";
import { ClipEntry } from "../../shared/ipc";
import { oneLine } from "./useClipboard";

export function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(0)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
}

const fileName = (p: string) => p.split("/").filter(Boolean).pop() ?? p;
const dirName = (p: string) => p.slice(0, Math.max(0, p.lastIndexOf("/"))) || "/";

function fileIcon(p: string): IconName {
  const ext = p.split(".").pop()?.toLowerCase() ?? "";
  if (["png", "jpg", "jpeg", "gif", "webp", "svg"].includes(ext)) return "photo";
  if (["zip", "gz", "tar", "rar", "7z"].includes(ext)) return "archive";
  if (["mp4", "mov", "mkv", "webm"].includes(ext)) return "film";
  if (!p.split("/").pop()?.includes(".")) return "folder";
  return "doc";
}

/** Square tile in front of a popup row: ★ when pinned, else what kind it is. */
export function ClipIcon({ c }: { c: ClipEntry }) {
  const tile = (name: IconName, cls = "") => (
    <span className={`clip-icon ${cls}`}>
      <Icon name={name} />
    </span>
  );
  if (c.pinned) return tile("starFill", "pinned");
  if (c.kind === "image") return c.thumb ? <img className="clip-icon thumb" src={c.thumb} alt="" /> : tile("photo");
  if (c.kind === "files") return tile(fileIcon(c.preview.split("\n")[0] ?? ""));
  const text = c.preview.trim();
  if (/^https?:\/\/\S+$/.test(text)) return tile("link");
  return tile(COMMAND.test(text) && !text.includes("\n") ? "terminal" : "text");
}

const COMMAND = /^(\$ |sudo |ssh |git |docker |kubectl |npm |npx |pnpm |yarn |cargo |python3? |pip |make |curl |wget |apt |systemctl |cd |ls )/;

/** How an entry looks in a list. `compact`: one line (popup). */
export function ClipPreview({ c, compact = false }: { c: ClipEntry; compact?: boolean }) {
  if (c.kind === "image") {
    return (
      <div className={`clip-image ${compact ? "compact" : ""}`}>
        {/* The popup row already shows the thumbnail as its icon. */}
        {c.thumb && !compact && <img src={c.thumb} alt="" />}
        <span className="clip-image-meta">
          {t("clip.image", { w: c.width ?? "?", h: c.height ?? "?" })}
          <span className="muted"> · {formatBytes(c.byteSize)}</span>
        </span>
      </div>
    );
  }
  if (c.kind === "files") {
    const paths = c.preview.split("\n").filter(Boolean);
    const shown = compact ? paths.slice(0, 1) : paths.slice(0, 4);
    return (
      <div className={`clip-files ${compact ? "compact" : ""}`}>
        {shown.map((p) => (
          <div key={p} className="clip-file" title={p}>
            <span className="clip-file-icon">
              <Icon name={fileIcon(p)} size={14} />
            </span>
            <span className="clip-file-name">{fileName(p)}</span>
            {!compact && <span className="muted clip-file-dir">{dirName(p)}</span>}
          </div>
        ))}
        {paths.length > shown.length && <span className="muted small">{t("clip.moreFiles", { n: paths.length - shown.length })}</span>}
      </div>
    );
  }
  return compact ? (
    <span className="clip-text">{oneLine(c.preview)}</span>
  ) : (
    <pre className="clip-content">
      {c.preview}
      {c.chars > c.preview.length ? "…" : ""}
    </pre>
  );
}
