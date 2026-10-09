import { useState } from "react";

import { useI18n } from "../../shared/i18n";
import { api, errorMessage } from "../../shared/ipc";
import { pickNotes, ReleaseNotes } from "../../shared/ReleaseNotes";
import { relativeTime } from "../../shared/time";
import { pendingVersion, useUpdate } from "./useUpdate";

/** Settings → Updates: version, last check, Check Now, automatic checks. */
export function UpdateSettings() {
  const { t } = useI18n();
  const [info, setInfo] = useUpdate();
  const [error, setError] = useState<string | null>(null);
  if (!info) return null;
  const s = info.status;
  const version = pendingVersion(info);

  const line = (() => {
    switch (s.state) {
      case "checking":
        return t("update.checking");
      case "upToDate":
        return t("update.upToDate", { time: relativeTime(s.checkedAt) });
      case "available":
        return t("update.available", { version: s.version });
      case "downloading":
        return t("update.downloading", { version: s.version, progress: s.total ? ` ${Math.round((s.downloaded / s.total) * 100)}%` : "" });
      case "installing":
        return t("update.installing", { version: s.version });
      case "error":
        return t("update.failed", { error: s.message });
      default:
        return t("update.notChecked");
    }
  })();

  return (
    <div className="update-settings">
      <p>
        {t("update.current", { version: info.currentVersion })} · <span className={s.state === "error" ? "error-text" : "muted"}>{line}</span>
      </p>
      <div className="row">
        {version && (s.state === "available" || s.state === "error") && (
          <button className="btn primary" onClick={() => void api.updateInstall().catch((e) => setError(errorMessage(e)))}>
            {t("update.installVersion", { version })}
          </button>
        )}
        <button className="btn" disabled={s.state === "checking" || s.state === "downloading" || s.state === "installing"} onClick={() => void api.updateCheck().then(setInfo, (e) => setError(errorMessage(e)))}>
          {t("update.checkNow")}
        </button>
      </div>
      {s.state === "available" && pickNotes(s.notes, s.notesVi) && <ReleaseNotes markdown={pickNotes(s.notes, s.notesVi)!} />}
      <label className="switch">
        <input type="checkbox" checked={info.autoCheck} onChange={(e) => void api.updateSetAuto(e.target.checked).then(setInfo, (x) => setError(errorMessage(x)))} />
        {t("update.auto")}
      </label>
      <p className="muted small">{t("update.explain")}</p>
      {error && <div className="banner error">{error}</div>}
    </div>
  );
}
