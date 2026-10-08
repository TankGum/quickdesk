import { useCallback, useEffect, useState } from "react";

import { api, errorMessage, PasteInfo, PasteMethod } from "../../shared/ipc";

const METHODS: { value: PasteMethod; label: string; hint: string }[] = [
  { value: "auto", label: "Automatic", hint: "Instant paste when enabled below, otherwise the desktop portal" },
  { value: "uinput", label: "Instant (virtual keyboard)", hint: "No prompts, no indicator" },
  { value: "portal", label: "Desktop portal", hint: "GNOME asks once and shows a remote-control indicator while pasting" },
];

export function PasteSettings() {
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
        Choosing a clipboard entry types it into the app you were using (Shift+Insert, which also works in terminals).
        Currently using: <b>{info.effective === "uinput" ? "instant (virtual keyboard)" : "desktop portal"}</b>.
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
              {m.label}
              <small className="muted"> · {m.hint}</small>
            </span>
          </label>
        ))}
      </div>

      {info.setupCommand && (
        <div className="setup-box">
          {info.uinputAvailable ? (
            <div className="row tight">
              <span>✓ Instant paste is enabled.</span>
              {info.ruleInstalled && (
                <button className="btn" disabled={busy} onClick={() => void run(api.clipUinputDisable)}>
                  {busy ? "Waiting for password…" : "Disable"}
                </button>
              )}
            </div>
          ) : (
            <>
              <div>
                <b>Instant paste is off.</b> Enabling it lets programs running as you create a virtual keyboard (the same
                permission Steam uses for controllers). You will be asked for your administrator password once.
              </div>
              <div className="row tight">
                {info.canEnable && (
                  <button className="btn primary" disabled={busy} onClick={() => void run(api.clipUinputEnable)}>
                    {busy ? "Waiting for password…" : "Enable instant paste"}
                  </button>
                )}
                <button className="btn" onClick={() => setShowManual((v) => !v)}>
                  {showManual ? "Hide" : "Set up manually"}
                </button>
              </div>
              {showManual && (
                <>
                  <pre className="setup-command">{info.setupCommand}</pre>
                  <div className="row tight">
                    <button className="btn" onClick={() => void api.clipboardWrite(info.setupCommand!)}>
                      Copy command
                    </button>
                    <button className="btn" onClick={refresh}>
                      Check again
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
