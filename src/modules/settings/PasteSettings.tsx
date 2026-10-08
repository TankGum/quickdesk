import { useCallback, useEffect, useState } from "react";

import { api, errorMessage, PasteInfo, PasteMethod } from "../../shared/ipc";

const METHODS: { value: PasteMethod; label: string; hint: string }[] = [
  { value: "auto", label: "Automatic", hint: "Virtual keyboard when set up, otherwise the desktop portal" },
  { value: "uinput", label: "Virtual keyboard", hint: "No prompts, no indicator; needs the one-time setup below" },
  { value: "portal", label: "Desktop portal", hint: "Works out of the box; GNOME asks once and shows a remote-control indicator while pasting" },
];

export function PasteSettings() {
  const [info, setInfo] = useState<PasteInfo | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const refresh = useCallback(() => {
    void api.clipPasteInfo().then(setInfo, (e) => setError(errorMessage(e)));
  }, []);
  useEffect(refresh, [refresh]);

  if (!info) return null;
  const choose = (m: PasteMethod) => void api.clipSetPasteMethod(m).then(refresh, (e) => setError(errorMessage(e)));

  return (
    <div className="paste-settings">
      <p className="muted small">
        Choosing a clipboard entry types it into the app you were using (Shift+Insert, which also works in terminals).
        Currently using: <b>{info.effective === "uinput" ? "virtual keyboard" : "desktop portal"}</b>.
      </p>
      <div className="radio-list">
        {METHODS.map((m) => (
          <label key={m.value} className="radio">
            <input type="radio" name="paste-method" checked={info.method === m.value} onChange={() => choose(m.value)} />
            <span>
              {m.label}
              <small className="muted"> · {m.hint}</small>
            </span>
          </label>
        ))}
      </div>
      {info.setupCommand && !info.uinputAvailable && info.method !== "portal" && (
        <div className="setup-box">
          <div>
            <b>One-time setup for prompt-free paste.</b> Run this in a terminal (asks for your password once). It lets
            programs running as you create a virtual keyboard, the same rule Steam installs for controllers.
          </div>
          <pre className="setup-command">{info.setupCommand}</pre>
          <div className="row tight">
            <button
              className="btn"
              onClick={() => void api.clipboardWrite(info.setupCommand!).then(() => setCopied(true))}
            >
              {copied ? "Copied ✓" : "Copy command"}
            </button>
            <button className="btn" onClick={refresh}>
              Check again
            </button>
          </div>
        </div>
      )}
      {info.method === "uinput" && !info.uinputAvailable && (
        <div className="banner error">Virtual keyboard is not available yet: run the setup command above.</div>
      )}
      {info.uinputAvailable && <div className="muted small">✓ Virtual keyboard ready.</div>}
      {error && <div className="banner error">{error}</div>}
    </div>
  );
}
