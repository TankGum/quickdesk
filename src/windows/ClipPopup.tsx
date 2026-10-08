import { useEffect, useRef, useState } from "react";

import { oneLine, useClipboard } from "../modules/clipboard/useClipboard";
import { api, errorMessage, hideWindow, useShown } from "../shared/ipc";
import { relativeTime } from "../shared/time";

const POPUP_LIMIT = 50;

export function ClipPopup() {
  const input = useRef<HTMLInputElement>(null);
  const list = useRef<HTMLUListElement>(null);
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState(0);
  const { entries, status, error, setError } = useClipboard(query, POPUP_LIMIT);

  useShown(() => {
    setQuery("");
    setSelected(0);
  }, () => input.current);

  useEffect(() => setSelected(0), [query]);
  useEffect(() => {
    list.current?.children[selected]?.scrollIntoView({ block: "nearest" });
  }, [selected]);

  const pick = async (index: number) => {
    const entry = entries[index];
    if (!entry) return;
    try {
      await api.clipCopy(entry.id);
      await hideWindow();
    } catch (e) {
      setError(errorMessage(e));
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
        void pick(selected);
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
    (status?.paused
      ? "History is paused (tray menu → Pause clipboard history)"
      : status && status.state !== "running" && status.state !== "starting"
        ? `Clipboard watcher ${status.state}: ${status.detail ?? ""}`
        : null);

  return (
    <div className="popup clip" onKeyDown={onKeyDown}>
      <input
        ref={input}
        className="popup-search"
        placeholder="🔍 Search clipboard..."
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        autoFocus
      />
      {banner && <div className="clip-banner">{banner}</div>}
      {entries.length === 0 ? (
        <div className="empty">{query ? "No matches." : "Nothing copied yet."}</div>
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
      <div className="popup-hint">↑↓ select · Enter copy · Ctrl+P pin · Del delete · Esc close</div>
    </div>
  );
}
