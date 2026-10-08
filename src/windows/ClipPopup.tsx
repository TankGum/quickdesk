import { useEffect, useRef, useState } from "react";

import { RecordingStatus } from "../modules/clipboard/RecordingStatus";
import { oneLine, useClipboard } from "../modules/clipboard/useClipboard";
import { useI18n } from "../shared/i18n";
import { api, errorMessage, hideWindow, useShown } from "../shared/ipc";
import { relativeTime } from "../shared/time";

const POPUP_LIMIT = 50;

export function ClipPopup() {
  const { t } = useI18n();
  const input = useRef<HTMLInputElement>(null);
  const list = useRef<HTMLUListElement>(null);
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState(0);
  const { entries, status, error, setError } = useClipboard(query, POPUP_LIMIT);

  useShown(() => {
    setQuery("");
    setSelected(0);
  }, () => input.current);

  // A stale error from an earlier attempt should not greet the next open.
  useEffect(() => {
    if (!error) return;
    const id = setTimeout(() => setError(null), 8000);
    return () => clearTimeout(id);
  }, [error, setError]);

  useEffect(() => setSelected(0), [query]);
  useEffect(() => {
    list.current?.children[selected]?.scrollIntoView({ block: "nearest" });
  }, [selected]);

  const pick = async (index: number, copyOnly = false) => {
    const entry = entries[index];
    if (!entry) return;
    try {
      if (status?.autoPaste && !copyOnly) {
        // Backend hides the popup, then types the text into the previous app.
        await api.clipPaste(entry.id);
      } else {
        await api.clipCopy(entry.id);
        await hideWindow();
      }
    } catch (e) {
      const pasteFailed = typeof e === "object" && e !== null && (e as { code?: string }).code === "paste_failed";
      setError(
        pasteFailed ? t("clip.pasteFailed", { error: errorMessage(e) }) : errorMessage(e),
      );
      if (pasteFailed) void api.show("clipboard");
    }
  };

  const onKeyDown = (e: React.KeyboardEvent) => {
    const entry = entries[selected];
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        setSelected((s) => Math.min(s + 1, entries.length - 1));
        break;
      case "ArrowUp":
        e.preventDefault();
        setSelected((s) => Math.max(s - 1, 0));
        break;
      case "Enter":
        e.preventDefault();
        void pick(selected, e.ctrlKey || e.metaKey);
        break;
      case "Escape":
        e.preventDefault();
        if (query) setQuery("");
        else void hideWindow();
        break;
      case "Delete":
        if (entry && !query) {
          e.preventDefault();
          void api.clipDelete(entry.id).catch((x) => setError(errorMessage(x)));
        }
        break;
      default:
        if (entry && (e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "p") {
          e.preventDefault();
          void api.clipPin(entry.id, !entry.pinned).catch((x) => setError(errorMessage(x)));
        }
    }
  };

  const banner =
    error ??
    (status && status.state !== "running" && status.state !== "starting"
      ? t("clip.watcher", { state: status.state, detail: status.detail ?? "" })
      : null);

  return (
    <div className="popup clip" onKeyDown={onKeyDown}>
      <input
        ref={input}
        className="popup-search"
        placeholder={t("clip.searchPopup")}
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        autoFocus
      />
      {status?.paused && <RecordingStatus status={status} compact onError={setError} />}
      {banner && <div className="clip-banner">{banner}</div>}
      {entries.length === 0 ? (
        <div className="empty">{query ? t("common.noMatches") : t("clip.empty")}</div>
      ) : (
        <ul ref={list} className="clip-list">
          {entries.map((c, i) => (
            <li
              key={c.id}
              className={i === selected ? "selected" : ""}
              onMouseEnter={() => setSelected(i)}
              onClick={() => void pick(i)}
            >
              <span className="clip-text">{oneLine(c.preview)}</span>
              <span className="clip-meta">
                {c.pinned && "★ "}
                {relativeTime(c.lastCopiedAt)}
              </span>
            </li>
          ))}
        </ul>
      )}
      <div className="popup-hint">
        {[
          t("clip.hint.select"),
          status?.autoPaste ? t("clip.hint.paste") : t("clip.hint.copy"),
          status?.autoPaste ? t("clip.hint.copyOnly") : null,
          t("clip.hint.rest"),
        ]
          .filter(Boolean)
          .join(" · ")}
      </div>
    </div>
  );
}
