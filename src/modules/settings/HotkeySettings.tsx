import { useCallback, useEffect, useRef, useState } from "react";

import { api, errorMessage, HotkeyCheck, HotkeyConfig, HotkeyTarget } from "../../shared/ipc";

const ROWS: { target: HotkeyTarget; key: keyof HotkeyConfig; label: string; hint: string }[] = [
  { target: "notes", key: "notes", label: "Notes", hint: "Open the notes manager" },
  { target: "quick-note", key: "quickNote", label: "Quick note", hint: "Small popup to jot one note (optional)" },
  { target: "clipboard", key: "clipboard", label: "Clipboard", hint: "Clipboard history popup" },
  { target: "ports", key: "ports", label: "Ports", hint: "Port manager" },
];

const MODIFIER_CODES = /^(Shift|Control|Alt|Meta|OS|Super)(Left|Right)?$/;

/** KeyboardEvent → "Super+Alt+N", or null while only modifiers are held. */
export function comboFromEvent(e: KeyboardEvent, superHeld: boolean): string | null {
  const c = e.code;
  const key = /^Key[A-Z]$/.test(c)
    ? c.slice(3)
    : /^Digit\d$/.test(c)
      ? c.slice(5)
      : /^F([1-9]|1\d|2[0-4])$/.test(c)
        ? c
        : c === "Space"
          ? "Space"
          : null;
  if (!key) return null;
  const mods: string[] = [];
  if (superHeld || e.metaKey || e.getModifierState("Super") || e.getModifierState("OS")) mods.push("Super");
  if (e.ctrlKey) mods.push("Ctrl");
  if (e.altKey) mods.push("Alt");
  if (e.shiftKey) mods.push("Shift");
  return [...mods, key].join("+");
}

type Recording = { target: HotkeyTarget; check: HotkeyCheck | null };

export function HotkeySettings() {
  const [config, setConfig] = useState<HotkeyConfig | null>(null);
  const [strategy, setStrategy] = useState<string>("");
  const [recording, setRecording] = useState<Recording | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const superHeld = useRef(false);

  useEffect(() => {
    void api.hotkeysGet().then((h) => {
      setConfig(h.config);
      setStrategy(h.strategy);
    });
  }, []);

  const stop = useCallback(async (save?: { target: HotkeyTarget; accel: string }) => {
    setRecording(null);
    superHeld.current = false;
    try {
      if (save) {
        setConfig(await api.hotkeysSet(save.target, save.accel));
        setNotice(save.accel ? `Saved ${save.accel}` : "Hotkey removed");
      } else {
        await api.hotkeysResume();
      }
      setError(null);
    } catch (e) {
      setError(errorMessage(e));
      await api.hotkeysResume().catch(() => {});
    }
  }, []);

  const start = async (target: HotkeyTarget) => {
    setError(null);
    setNotice(null);
    // Unbind ours first so pressing the current combination reaches this window.
    await api.hotkeysSuspend();
    setRecording({ target, check: null });
  };

  // Capture keys while recording.
  useEffect(() => {
    if (!recording) return;
    const down = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (MODIFIER_CODES.test(e.code)) {
        if (/^(Meta|OS|Super)/.test(e.code)) superHeld.current = true;
        return;
      }
      if (e.key === "Escape" && !e.ctrlKey && !e.altKey && !e.metaKey && !superHeld.current) {
        void stop();
        return;
      }
      const combo = comboFromEvent(e, superHeld.current);
      if (!combo) return;
      void api.hotkeysCheck(recording.target, combo).then((check) => setRecording((r) => r && { ...r, check }));
    };
    const up = (e: KeyboardEvent) => {
      if (/^(Meta|OS|Super)/.test(e.code)) superHeld.current = false;
    };
    // Leaving the window while recording must not leave hotkeys unbound.
    const blur = () => void stop();
    window.addEventListener("keydown", down, true);
    window.addEventListener("keyup", up, true);
    window.addEventListener("blur", blur);
    return () => {
      window.removeEventListener("keydown", down, true);
      window.removeEventListener("keyup", up, true);
      window.removeEventListener("blur", blur);
    };
  }, [recording, stop]);

  // Unmounting mid-recording (tab switch) restores hotkeys too.
  const recordingRef = useRef(recording);
  recordingRef.current = recording;
  useEffect(
    () => () => {
      if (recordingRef.current) void api.hotkeysResume();
    },
    [],
  );

  useEffect(() => {
    if (!notice) return;
    const id = setTimeout(() => setNotice(null), 2500);
    return () => clearTimeout(id);
  }, [notice]);

  if (!config) return null;

  return (
    <div className="hotkeys">
      <table className="hotkey-table">
        <tbody>
          {ROWS.map((row) => {
            const value = config[row.key];
            const rec = recording?.target === row.target ? recording : null;
            return (
              <tr key={row.target} className={rec ? "recording" : ""}>
                <td className="hk-label">
                  <div>{row.label}</div>
                  <small className="muted">{row.hint}</small>
                </td>
                <td className="hk-value">
                  {rec ? (
                    <RecordingCell rec={rec} onSave={(accel) => void stop({ target: row.target, accel })} onCancel={() => void stop()} />
                  ) : value ? (
                    <kbd>{value}</kbd>
                  ) : (
                    <span className="muted">Not set</span>
                  )}
                </td>
                <td className="hk-actions">
                  {!rec && (
                    <>
                      <button className="btn" disabled={!!recording} onClick={() => void start(row.target)}>
                        Change
                      </button>
                      {value && (
                        <button className="btn" disabled={!!recording} onClick={() => void stop({ target: row.target, accel: "" })}>
                          Clear
                        </button>
                      )}
                    </>
                  )}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
      {error && <div className="banner error">{error}</div>}
      {notice && <div className="muted small">{notice}</div>}
      <div className="row">
        <button
          className="btn"
          disabled={!!recording}
          onClick={() =>
            void api.hotkeysReset().then(
              (c) => {
                setConfig(c);
                setNotice("Restored defaults");
              },
              (e) => setError(errorMessage(e)),
            )
          }
        >
          Restore defaults
        </button>
      </div>
      <p className="muted small">
        {strategy === "GnomeKeybinding"
          ? "Registered as GNOME custom shortcuts (Settings → Keyboard → Custom Shortcuts) and removed when QuickDesk quits. "
          : strategy === "Manual"
            ? "This desktop has no global-shortcut API: bind `quickdesk toggle notes|quick-note|clipboard|ports` in your desktop settings. "
            : ""}
        Vietnamese input methods (Unikey, Bamboo) swallow <kbd>Super</kbd>+<kbd>Shift</kbd>+letter while you type in a text field;
        combinations with <kbd>Alt</kbd> or <kbd>Ctrl</kbd> keep working there.
      </p>
    </div>
  );
}

function RecordingCell({ rec, onSave, onCancel }: { rec: Recording; onSave: (accel: string) => void; onCancel: () => void }) {
  const c = rec.check;
  if (!c) {
    return (
      <span className="recording-hint">
        Press the new shortcut… <span className="muted">(Esc to cancel)</span>
      </span>
    );
  }
  return (
    <div className="hk-check">
      <kbd>{c.normalized}</kbd>
      {c.error ? (
        <div className="error-text small">{c.error}. Try another combination, or Esc to cancel.</div>
      ) : (
        <>
          {c.conflicts.length > 0 && (
            <div className="warn-text small">Already used by {c.conflicts.join(", ")}; it may not reach QuickDesk.</div>
          )}
          {c.inputMethodWarning && (
            <div className="warn-text small">Without Alt/Ctrl this may not work while typing with Unikey/Bamboo.</div>
          )}
          <div className="row tight">
            <button className="btn primary" onClick={() => onSave(c.normalized)}>
              Save
            </button>
            <button className="btn" onClick={onCancel}>
              Cancel
            </button>
            <span className="muted small">or press another combination</span>
          </div>
        </>
      )}
    </div>
  );
}
