import { useCallback, useEffect, useState } from "react";

import { Key, t, useI18n } from "../../shared/i18n";
import { api, errorMessage, PasteInfo, PasteMethod } from "../../shared/ipc";
import { Icon } from "../../shared/Icon";

const METHODS: { value: PasteMethod; label: Key; hint: Key }[] = [
  { value: "auto", label: "paste.auto", hint: "paste.auto.hint" },
  { value: "uinput", label: "paste.uinput", hint: "paste.uinput.hint" },
  { value: "portal", label: "paste.portal", hint: "paste.portal.hint" },
];

export function PasteSettings() {
  useI18n();
  const [info, setInfo] = useState<PasteInfo | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [showManual, setShowManual] = useState(false);

  const refresh = useCallback(() => {
    void api.clipPasteInfo().then(setInfo, (e) => setError(errorMessage(e)));
  }, []);
  useEffect(refresh, [refresh]);

  const run = async (fn: () => Promise<void>) => {
    setBusy(true);
    setError(null);
    try {
      await fn();
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
      refresh();
    }
  };

  if (!info) return null;

  return (
    <div className="paste-settings">
      <p className="muted small">
        {t("paste.intro")}
        <b>{info.effective === "uinput" ? t("paste.usingInstant") : t("paste.usingPortal")}</b>.
      </p>
      <div className="radio-list">
        {METHODS.map((m) => (
          <label key={m.value} className="radio">
            <input
              type="radio"
              name="paste-method"
              checked={info.method === m.value}
              onChange={() => void run(() => api.clipSetPasteMethod(m.value))}
            />
            <span>
              {t(m.label)}
              <small className="muted"> · {t(m.hint)}</small>
            </span>
          </label>
        ))}
      </div>

      {info.setupCommand && (
        <div className="setup-box">
          {info.uinputAvailable ? (
            <div className="row tight">
              <span className="ok-text">
                <Icon name="check" size={14} /> {t("paste.enabled")}
              </span>
              {info.ruleInstalled && (
                <button className="btn" disabled={busy} onClick={() => void run(api.clipUinputDisable)}>
                  {busy ? t("common.waitingPassword") : t("paste.disable")}
                </button>
              )}
            </div>
          ) : (
            <>
              <div>
                <b>{t("paste.off")}</b>
                {t("paste.offExplain")}
              </div>
              <div className="row tight">
                {info.canEnable && (
                  <button className="btn primary" disabled={busy} onClick={() => void run(api.clipUinputEnable)}>
                    {busy ? t("common.waitingPassword") : t("paste.enable")}
                  </button>
                )}
                <button className="btn" onClick={() => setShowManual((v) => !v)}>
                  {showManual ? t("paste.hide") : t("paste.manual")}
                </button>
              </div>
              {showManual && (
                <>
                  <pre className="setup-command">{info.setupCommand}</pre>
                  <div className="row tight">
                    <button className="btn" onClick={() => void api.clipboardWrite(info.setupCommand!)}>
                      {t("paste.copyCommand")}
                    </button>
                    <button className="btn" onClick={refresh}>
                      {t("paste.checkAgain")}
                    </button>
                  </div>
                </>
              )}
            </>
          )}
        </div>
      )}
      {error && <div className="banner error">{error}</div>}
    </div>
  );
}
