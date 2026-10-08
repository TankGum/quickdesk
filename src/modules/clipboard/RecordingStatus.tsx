import { useEffect, useRef, useState } from "react";

import { api, ClipStatus, errorMessage } from "../../shared/ipc";

const PAUSE_OPTIONS: { label: string; minutes?: number }[] = [
  { label: "For 15 minutes", minutes: 15 },
  { label: "For 1 hour", minutes: 60 },
  { label: "Until I turn it back on" },
];

function clock(ms: number) {
  return new Date(ms).toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });
}

function remaining(ms: number) {
  const min = Math.max(1, Math.round((ms - Date.now()) / 60_000));
  return min >= 60 ? `${Math.round(min / 60)} h` : `${min} min`;
}

/**
 * Says in plain words whether copies are being saved, and lets the user pause
 * for a while (e.g. before copying a password) or resume.
 */
export function RecordingStatus({ status, compact = false, onError }: { status: ClipStatus; compact?: boolean; onError: (e: string) => void }) {
  const [menu, setMenu] = useState(false);
  const [, tick] = useState(0);
  const box = useRef<HTMLDivElement>(null);

  // Keep "resumes in N min" current.
  useEffect(() => {
    if (!status.pausedUntil) return;
    const id = setInterval(() => tick((n) => n + 1), 30_000);
    return () => clearInterval(id);
  }, [status.pausedUntil]);

  useEffect(() => {
    if (!menu) return;
    const close = (e: MouseEvent) => !box.current?.contains(e.target as Node) && setMenu(false);
    window.addEventListener("mousedown", close);
    return () => window.removeEventListener("mousedown", close);
  }, [menu]);

  const set = (paused: boolean, minutes?: number) => {
    setMenu(false);
    void api.clipSetPaused(paused, minutes).catch((e) => onError(errorMessage(e)));
  };

  if (status.paused) {
    const until = status.pausedUntil
      ? `Turns back on at ${clock(status.pausedUntil)} (in ${remaining(status.pausedUntil)}).`
      : "Stays off until you turn it back on.";
    return (
      <div className={`recording paused ${compact ? "compact" : ""}`}>
        <span className="recording-icon">⏸</span>
        <div className="recording-text">
          <b>Not saving copies right now.</b>
          {!compact && <> Copy and paste still work; new copies just aren't added to the history below.</>} {until}
        </div>
        <button className="btn primary" onClick={() => set(false)}>
          Turn back on
        </button>
      </div>
    );
  }

  if (compact) return null;

  return (
    <div className="recording on" ref={box}>
      <span className="dot" />
      <div className="recording-text">
        <b>Saving everything you copy</b>
        <span className="muted"> · only on this device, never synced</span>
      </div>
      <div className="menu-anchor">
        <button className="btn" onClick={() => setMenu((m) => !m)} title="Stop saving copies for a while, e.g. before copying a password">
          Pause saving ▾
        </button>
        {menu && (
          <div className="menu">
            <div className="menu-hint">
              Stop saving new copies, e.g. before copying a password, token or customer data. Copy and paste keep working.
            </div>
            {PAUSE_OPTIONS.map((o) => (
              <button key={o.label} className="menu-item" onClick={() => set(true, o.minutes)}>
                {o.label}
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
