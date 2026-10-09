import { useEffect, useState } from "react";

import { lang, setLanguage, useI18n } from "../shared/i18n";
import { markSeen } from "../modules/update/WhatsNew";
import { api, errorMessage, HotkeyConfig } from "../shared/ipc";

/** First-run screen: shows the hotkeys and offers the one-time instant-paste opt-in. */
export function Welcome({ onDone }: { onDone: () => void }) {
  const { t, pref } = useI18n();
  const [offerUinput, setOfferUinput] = useState(false);
  const [enableUinput, setEnableUinput] = useState(true);
  const [autostart, setAutostart] = useState(true);
  const [hotkeys, setHotkeys] = useState<HotkeyConfig | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void api.onboarding().then((o) => setOfferUinput(o.offerUinput));
    void api.hotkeysGet().then((h) => setHotkeys(h.config));
  }, []);

  const start = async () => {
    setBusy(true);
    setError(null);
    // Best effort: a failure here should not block getting started.
    await api.autostartSet(autostart).catch(() => {});
    if (offerUinput && enableUinput) {
      try {
        await api.clipUinputEnable();
      } catch (e) {
        // Not fatal: auto-paste falls back to the desktop portal.
        setError(t("welcome.instantFailed", { error: errorMessage(e) }));
        setBusy(false);
        setOfferUinput(false);
        return;
      }
    }
    await api.onboardingFinish();
    // A new install, not an update: no "what's new" for this version.
    const { currentVersion } = await api.updateStatus();
    markSeen(currentVersion);
    onDone();
  };

  return (
    <div className="welcome">
      <div className="radio-list horizontal welcome-lang">
        {(
          [
            ["vi", "Tiếng Việt"],
            ["en", "English"],
          ] as const
        ).map(([value, label]) => (
          <label key={value} className="radio">
            <input
              type="radio"
              name="welcome-lang"
              checked={pref === value || (pref === "auto" && lang() === value)}
              onChange={() => void setLanguage(value)}
            />
            <span>{label}</span>
          </label>
        ))}
      </div>
      <h1>{t("welcome.title")}</h1>
      <p className="muted">{t("welcome.intro")}</p>
      {hotkeys && (
        <dl className="welcome-keys">
          <dt><kbd>{hotkeys.notes}</kbd></dt>
          <dd>{t("welcome.notes")}</dd>
          <dt><kbd>{hotkeys.clipboard}</kbd></dt>
          <dd>{t("welcome.clipboard")}</dd>
          <dt><kbd>{hotkeys.ports}</kbd></dt>
          <dd>{t("welcome.ports")}</dd>
        </dl>
      )}
      <label className="welcome-option">
        <input type="checkbox" checked={autostart} onChange={(e) => setAutostart(e.target.checked)} />
        <span>
          <b>{t("welcome.autostart")}</b> {t("welcome.recommended")}
          <small className="muted">{t("welcome.autostartHint")}</small>
        </span>
      </label>
      {offerUinput && (
        <label className="welcome-option">
          <input type="checkbox" checked={enableUinput} onChange={(e) => setEnableUinput(e.target.checked)} />
          <span>
            <b>{t("welcome.instant")}</b> {t("welcome.recommended")}
            <small className="muted">{t("welcome.instantHint")}</small>
          </span>
        </label>
      )}
      {error && <div className="banner error">{error}</div>}
      <button className="btn primary big" disabled={busy} onClick={() => void start()}>
        {busy ? t("common.waitingPassword") : t("welcome.start")}
      </button>
      <p className="muted small">{t("welcome.footer")}</p>
    </div>
  );
}
