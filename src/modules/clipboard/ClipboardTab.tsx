import { useEffect, useRef, useState } from "react";

import { api, errorMessage } from "../../shared/ipc";
import { absoluteTime, relativeTime } from "../../shared/time";
import { RecordingStatus } from "./RecordingStatus";
import { useClipboard } from "./useClipboard";

export function ClipboardTab({ focusSignal }: { focusSignal: number }) {
  const [query, setQuery] = useState("");
  const [confirmClear, setConfirmClear] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const search = useRef<HTMLInputElement>(null);
  const { entries, status, error, setError } = useClipboard(query, 500);

  useEffect(() => {
    search.current?.focus();
  }, [focusSignal]);

  useEffect(() => {
    if (!notice) return;
    const id = setTimeout(() => setNotice(null), 2000);
    return () => clearTimeout(id);
  }, [notice]);

  const run = (p: Promise<unknown>, done?: string) =>
    p.then(() => done && setNotice(done)).catch((e) => setError(errorMessage(e)));

  return (
    <div className="clipboard">
      <div className="toolbar">
        <input
          ref={search}
          className="search"
          placeholder="Search clipboard history… (substring, accents optional)"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => e.key === "Escape" && setQuery("")}
        />
        {confirmClear ? (
          <>
            <button className="btn danger" onClick={() => void run(api.clipClear(true), "History cleared").then(() => setConfirmClear(false))}>
              Clear (keep pinned)
            </button>
            <button className="btn" onClick={() => setConfirmClear(false)}>
              Cancel
            </button>
          </>
        ) : (
          <button className="btn" onClick={() => setConfirmClear(true)}>
            Clear…
          </button>
        )}
      </div>
      {status && <RecordingStatus status={status} onError={setError} />}
      {status && (
        <label className="switch paste-pref" title="Uses Settings → Auto-paste">
          <input type="checkbox" checked={status.autoPaste} onChange={(e) => void run(api.clipSetAutoPaste(e.target.checked))} />
          <span>
            Paste right away when I pick an entry in the popup
            <small className="muted"> (off: it is only copied, then press Ctrl+V yourself)</small>
          </span>
        </label>
      )}
      {status && status.state !== "running" && status.state !== "starting" && (
        <div className="banner error">
          Clipboard watcher {status.state} ({status.backend}){status.detail ? `: ${status.detail}` : ""}
        </div>
      )}
      {error && <div className="banner error">{error}</div>}
      {entries.length === 0 ? (
        <div className="empty">{query ? "No matches." : "Nothing copied yet. Copy something anywhere and it shows up here."}</div>
      ) : (
        <ul className="clip-history">
          {entries.map((c) => (
            <li key={c.id} className={c.pinned ? "pinned" : ""}>
              <button className={`icon pin ${c.pinned ? "on" : ""}`} title={c.pinned ? "Unpin" : "Pin (never pruned)"} onClick={() => void run(api.clipPin(c.id, !c.pinned))}>
                {c.pinned ? "★" : "☆"}
              </button>
              <div className="clip-main" onClick={() => void run(api.clipCopy(c.id), "Copied to clipboard")} title="Click to copy">
                <pre className="clip-content">{c.preview}{c.chars > c.preview.length ? "…" : ""}</pre>
                <div className="note-meta">
                  <span title={`First copied ${absoluteTime(c.firstCopiedAt)}`}>{relativeTime(c.lastCopiedAt)}</span>
                  {c.copyCount > 1 && <span>· copied {c.copyCount}×</span>}
                  <span>· {c.chars.toLocaleString()} chars</span>
                </div>
              </div>
              <button className="icon delete" title="Delete" onClick={() => void run(api.clipDelete(c.id))}>
                ✕
              </button>
            </li>
          ))}
        </ul>
      )}
      {notice && <div className="toast">{notice}</div>}
    </div>
  );
}
