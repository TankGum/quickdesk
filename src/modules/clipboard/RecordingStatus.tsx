import { useEffect, useRef, useState } from "react";

import { Key, t, useI18n } from "../../shared/i18n";
import { api, ClipStatus, errorMessage } from "../../shared/ipc";
import { clockTime } from "../../shared/time";
import { Icon } from "../../shared/Icon";

const PAUSE_OPTIONS: { label: Key; minutes?: number }[] = [
  { label: "rec.for15", minutes: 15 },
  { label: "rec.for60", minutes: 60 },
  { label: "rec.untilResume" },
];

function remaining(ms: number) {
  const min = Math.max(1, Math.round((ms - Date.now()) / 60_000));
  return min >= 60 ? t("rec.hours", { n: Math.round(min / 60) }) : t("rec.minutes", { n: min });
}

/**
 * Says in plain words whether copies are being saved, and lets the user pause
 * for a while (e.g. before copying a password) or resume.
 */
export function RecordingStatus({ status, compact = false, onError }: { status: ClipStatus; compact?: boolean; onError: (e: string) => void }) {
  useI18n();
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
      ? t("rec.resumesAt", { time: clockTime(status.pausedUntil), left: remaining(status.pausedUntil) })
      : t("rec.staysOff");
    return (
      <div className={`recording paused ${compact ? "compact" : ""}`}>
        <span className="recording-icon">
          <Icon name="pause" />
        </span>
        <div className="recording-text">
          <b>{t("rec.paused")}</b>
          {!compact && <> {t("rec.pausedExplain")}</>} {until}
        </div>
        <button className="btn primary" onClick={() => set(false)}>
          {t("rec.resume")}
        </button>
      </div>
    );
  }

  if (compact) return null;

  return (
    <div className="recording on" ref={box}>
      <span className="dot" />
      <div className="recording-text">
        <b>{t("rec.saving")}</b>
      </div>
      <div className="menu-anchor">
        <button className="btn" onClick={() => setMenu((m) => !m)} title={t("rec.pauseTitle")}>
          {t("rec.pause")}
          <Icon name="chevronDown" size={14} />
        </button>
        {menu && (
          <div className="menu">
            <div className="menu-hint">{t("rec.menuHint")}</div>
            {PAUSE_OPTIONS.map((o) => (
              <button key={o.label} className="menu-item" onClick={() => set(true, o.minutes)}>
                {t(o.label)}
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
