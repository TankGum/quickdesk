import { useState } from "react";

import { Icon } from "../../shared/Icon";
import { useI18n } from "../../shared/i18n";
import { api, errorMessage } from "../../shared/ipc";
import { pendingVersion, useUpdate } from "./useUpdate";

/** Top-of-window notice when a new version is ready; installs only on request. */
export function UpdateBanner() {
  const { t } = useI18n();
  const [info] = useUpdate();
  const [dismissed, setDismissed] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const version = pendingVersion(info);
  const s = info?.status;
  if (!s || !version || dismissed === version) return null;

  const install = () => {
    setError(null);
    void api.updateInstall().catch((e) => setError(errorMessage(e)));
  };

  let text: string;
  let actions = true;
  if (s.state === "downloading") {
    const pct = s.total ? Math.round((s.downloaded / s.total) * 100) : null;
    text = t("update.downloading", { version, progress: pct === null ? "" : ` ${pct}%` });
    actions = false;
  } else if (s.state === "installing") {
    text = t("update.installing", { version });
    actions = false;
  } else if (s.state === "error") {
    text = t("update.failed", { error: error ?? s.message });
  } else {
    text = t("update.available", { version });
  }

  return (
    <div className={`update-banner ${s.state}`} role="status">
      <Icon name="refresh" size={15} />
      <span className="update-text">{text}</span>
      {actions && (
        <>
          <button className="btn primary" onClick={install}>
            {s.state === "error" ? t("update.retry") : t("update.now")}
          </button>
          <button className="btn" onClick={() => setDismissed(version)}>
            {t("update.later")}
          </button>
        </>
      )}
    </div>
  );
}
