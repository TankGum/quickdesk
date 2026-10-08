import { useEffect, useRef, useState } from "react";

import { locale, useI18n } from "../../shared/i18n";
import { api, ClipKind, errorMessage } from "../../shared/ipc";
import { absoluteTime, relativeTime } from "../../shared/time";
import { ClipPreview } from "./ClipPreview";
import { KindFilter } from "./KindFilter";
import { RecordingStatus } from "./RecordingStatus";
import { useClipboard } from "./useClipboard";
import { Icon } from "../../shared/Icon";

export function ClipboardTab({ focusSignal }: { focusSignal: number }) {
  const { t } = useI18n();
  const [query, setQuery] = useState("");
  const [confirmClear, setConfirmClear] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const search = useRef<HTMLInputElement>(null);
  const [kind, setKind] = useState<ClipKind | undefined>(undefined);
  const { entries, status, error, setError } = useClipboard(query, 500, kind);

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
          placeholder={t("clip.searchTab")}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => e.key === "Escape" && setQuery("")}
        />
        {confirmClear ? (
          <>
            <button className="btn danger" onClick={() => void run(api.clipClear(true), t("clip.cleared")).then(() => setConfirmClear(false))}>
              {t("clip.clearConfirm")}
            </button>
            <button className="btn" onClick={() => setConfirmClear(false)}>
              {t("common.cancel")}
            </button>
          </>
        ) : (
          <button className="btn" onClick={() => setConfirmClear(true)}>
            {t("clip.clear")}
          </button>
        )}
      </div>
      <KindFilter value={kind} onChange={setKind} />
      {status && <RecordingStatus status={status} onError={setError} />}
      {status && (
        <label className="switch paste-pref" title={t("clip.pastePrefTitle")}>
          <input type="checkbox" checked={status.autoPaste} onChange={(e) => void run(api.clipSetAutoPaste(e.target.checked))} />
          <span>
            {t("clip.pastePref")}
            <small className="muted">{t("clip.pastePrefHint")}</small>
          </span>
        </label>
      )}
      {status && status.state !== "running" && status.state !== "starting" && (
        <div className="banner error">
          {t("clip.watcher", { state: `${status.state} (${status.backend})`, detail: status.detail ?? "" })}
        </div>
      )}
      {error && <div className="banner error">{error}</div>}
      {entries.length === 0 ? (
        <div className="empty">{query ? t("common.noMatches") : t("clip.emptyTab")}</div>
      ) : (
        <ul className="clip-history">
          {entries.map((c) => (
            <li key={c.id} className={c.pinned ? "pinned" : ""}>
              <button className={`icon pin ${c.pinned ? "on" : ""}`} title={c.pinned ? t("common.unpin") : t("clip.pinHint")} onClick={() => void run(api.clipPin(c.id, !c.pinned))}>
                <Icon name={c.pinned ? "starFill" : "star"} />
              </button>
              <div className="clip-main" onClick={() => void run(api.clipCopy(c.id), t("clip.copiedToClipboard"))} title={t("clip.clickToCopy")}>
                <ClipPreview c={c} />
                <div className="note-meta">
                  <span title={t("clip.firstCopied", { time: absoluteTime(c.firstCopiedAt) })}>{relativeTime(c.lastCopiedAt)}</span>
                  {c.copyCount > 1 && <span>{t("clip.copiedTimes", { n: c.copyCount })}</span>}
                  {c.kind === "text" && <span>{t("clip.chars", { n: c.chars.toLocaleString(locale()) })}</span>}
                  {c.kind === "files" && <span>· {t("clip.files", { n: c.preview.split("\n").filter(Boolean).length })}</span>}
                </div>
              </div>
              <button className="icon delete" title={t("common.delete")} onClick={() => void run(api.clipDelete(c.id))}>
                <Icon name="xmark" />
              </button>
            </li>
          ))}
        </ul>
      )}
      {notice && <div className="toast">{notice}</div>}
    </div>
  );
}
