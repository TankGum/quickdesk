import { useCallback, useEffect, useRef, useState } from "react";

import { Key, t, useI18n } from "../../shared/i18n";
import { api, errorMessage, HotkeyCheck, HotkeyConfig, HotkeyTarget } from "../../shared/ipc";

const ROWS: { target: HotkeyTarget; key: keyof HotkeyConfig; label: Key; hint: Key }[] = [
  { target: "notes", key: "notes", label: "hk.notes", hint: "hk.notes.hint" },
  { target: "quick-note", key: "quickNote", label: "hk.quickNote", hint: "hk.quickNote.hint" },
  { target: "clipboard", key: "clipboard", label: "hk.clipboard", hint: "hk.clipboard.hint" },
  { target: "ports", key: "ports", label: "hk.ports", hint: "hk.ports.hint" },
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
  useI18n();
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
        setNotice(save.accel ? t("hk.saved", { accel: save.accel }) : t("hk.removed"));
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
                  <div>{t(row.label)}</div>
                  <small className="muted">{t(row.hint)}</small>
                </td>
                <td className="hk-value">
                  {rec ? (
                    <RecordingCell rec={rec} onSave={(accel) => void stop({ target: row.target, accel })} onCancel={() => void stop()} />
                  ) : value ? (
                    <kbd>{value}</kbd>
                  ) : (
                    <span className="muted">{t("hk.notSet")}</span>
                  )}
                </td>
                <td className="hk-actions">
                  {!rec && (
                    <>
                      <button className="btn" disabled={!!recording} onClick={() => void start(row.target)}>
                        {t("hk.change")}
                      </button>
                      {value && (
                        <button className="btn" disabled={!!recording} onClick={() => void stop({ target: row.target, accel: "" })}>
                          {t("hk.clear")}
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
                setNotice(t("hk.restored"));
              },
              (e) => setError(errorMessage(e)),
            )
          }
        >
          {t("hk.reset")}
        </button>
      </div>
      <p className="muted small">
        {strategy === "GnomeKeybinding" ? t("hk.gnome") : strategy === "Manual" ? t("hk.manual") : ""}
        {t("hk.ime")}
      </p>
    </div>
  );
}

function RecordingCell({ rec, onSave, onCancel }: { rec: Recording; onSave: (accel: string) => void; onCancel: () => void }) {
  const c = rec.check;
  if (!c) {
    return (
      <span className="recording-hint">
        {t("hk.press")} <span className="muted">{t("hk.escCancel")}</span>
      </span>
    );
  }
  return (
    <div className="hk-check">
      <kbd>{c.normalized}</kbd>
      {c.error ? (
        <div className="error-text small">{t("hk.tryAnother", { error: c.error })}</div>
      ) : (
        <>
          {c.conflicts.length > 0 && (
            <div className="warn-text small">{t("hk.conflict", { who: c.conflicts.join(", ") })}</div>
          )}
          {c.inputMethodWarning && (
            <div className="warn-text small">{t("hk.imeWarning")}</div>
          )}
          <div className="row tight">
            <button className="btn primary" onClick={() => onSave(c.normalized)}>
              {t("hk.save")}
            </button>
            <button className="btn" onClick={onCancel}>
              {t("common.cancel")}
            </button>
            <span className="muted small">{t("hk.orAnother")}</span>
          </div>
        </>
      )}
    </div>
  );
}
