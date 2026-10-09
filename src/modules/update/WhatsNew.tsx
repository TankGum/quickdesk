import { useEffect, useState } from "react";

import { Icon } from "../../shared/Icon";
import { useI18n } from "../../shared/i18n";
import { api } from "../../shared/ipc";
import { bundledNotes, ReleaseNotes } from "../../shared/ReleaseNotes";

const SEEN_KEY = "qd.whatsNewSeen";

function seen(): string | null {
  try {
    return localStorage.getItem(SEEN_KEY);
  } catch {
    return null;
  }
}
/** Remember that `version`'s notes need no announcement (also called after first-run setup). */
export function markSeen(version: string) {
  try {
    localStorage.setItem(SEEN_KEY, version);
  } catch {
    /* storage unavailable: the notice may show again, which is harmless */
  }
}

/** Once after an update: "Updated to X" with that version's notes. */
export function WhatsNew() {
  const { t } = useI18n();
  const [version, setVersion] = useState<string | null>(null);
  const [open, setOpen] = useState(false);

  useEffect(() => {
    void Promise.all([api.updateStatus(), api.onboarding()]).then(([info, onboarding]) => {
      const current = info.currentVersion;
      // A fresh install has nothing to announce (Welcome marks it seen); only upgrades do.
      if (!onboarding.done) return markSeen(current);
      if (seen() !== current && bundledNotes(current)) setVersion(current);
    });
  }, []);

  if (!version) return null;
  const close = () => {
    markSeen(version);
    setVersion(null);
  };
  const notes = bundledNotes(version);

  return (
    <div className="update-banner whatsnew-banner" role="status">
      <div className="update-row">
        <Icon name="check" size={15} />
        <span className="update-text">{t("update.updated", { version })}</span>
        <button className="btn" onClick={() => setOpen((o) => !o)}>
          {t("update.whatsNew")}
          <Icon name="chevronDown" size={14} className={open ? "flip" : ""} />
        </button>
        <button className="icon" title={t("common.close")} onClick={close}>
          <Icon name="xmark" />
        </button>
      </div>
      {open && notes && <ReleaseNotes markdown={notes} />}
    </div>
  );
}
