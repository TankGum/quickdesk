import { useEffect, useState } from "react";

import { api, errorMessage, HotkeyConfig } from "../shared/ipc";

/** First-run screen: shows the hotkeys and offers the one-time instant-paste opt-in. */
export function Welcome({ onDone }: { onDone: () => void }) {
  const [offerUinput, setOfferUinput] = useState(false);
  const [enableUinput, setEnableUinput] = useState(true);
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
    if (offerUinput && enableUinput) {
      try {
        await api.clipUinputEnable();
      } catch (e) {
        // Not fatal: auto-paste falls back to the desktop portal.
        setError(`${errorMessage(e)}. You can enable it later in Settings → Auto-paste.`);
        setBusy(false);
        setOfferUinput(false);
        return;
      }
    }
    await api.onboardingFinish();
    onDone();
  };

  return (
    <div className="welcome">
      <h1>Welcome to QuickDesk</h1>
      <p className="muted">Lives in the tray. These shortcuts work from any app:</p>
      {hotkeys && (
        <dl className="welcome-keys">
          <dt><kbd>{hotkeys.notes}</kbd></dt>
          <dd>Notes: jot something down, search everything you wrote</dd>
          <dt><kbd>{hotkeys.clipboard}</kbd></dt>
          <dd>Clipboard history: pick an entry and it is pasted where you were typing</dd>
          <dt><kbd>{hotkeys.ports}</kbd></dt>
          <dd>Ports: what is listening on 3000, 8000… and stop it</dd>
        </dl>
      )}
      {offerUinput && (
        <label className="welcome-option">
          <input type="checkbox" checked={enableUinput} onChange={(e) => setEnableUinput(e.target.checked)} />
          <span>
            <b>Enable instant paste</b> (recommended)
            <small className="muted">
              Pastes without a permission prompt or a remote-control indicator. Asks for your administrator password once and
              lets programs running as you create a virtual keyboard, the same permission Steam uses for controllers. You can
              change this later in Settings.
            </small>
          </span>
        </label>
      )}
      {error && <div className="banner error">{error}</div>}
      <button className="btn primary big" disabled={busy} onClick={() => void start()}>
        {busy ? "Waiting for password…" : "Get started"}
      </button>
      <p className="muted small">Change shortcuts any time in Settings → Hotkeys.</p>
    </div>
  );
}
