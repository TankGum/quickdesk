// Formatting helpers shared by the AI tab and the usage popup.
import { Key, t } from "../../shared/i18n";
import { RingChoice, UsageWindow } from "../../shared/ipc";
import { absoluteTime, clockTime } from "../../shared/time";

export const STALE_MS = 60_000;

export type Level = "ok" | "warn" | "high";

export function level(percent: number): Level {
  return percent >= 85 ? "high" : percent >= 60 ? "warn" : "ok";
}

export function duration(ms: number): string {
  const min = Math.max(0, Math.ceil(ms / 60_000));
  const d = Math.floor(min / 1440);
  const h = Math.floor((min % 1440) / 60);
  const m = min % 60;
  if (d > 0) return t("ai.duration.dh", { d, h });
  if (h > 0) return t("ai.duration.hm", { h, m });
  return t("ai.duration.min", { m });
}

/** Today: "16:20"; otherwise date and time. */
export function when(ms: number): string {
  return new Date(ms).toDateString() === new Date().toDateString() ? clockTime(ms) : absoluteTime(ms);
}

export function windowLabel(w: UsageWindow): string {
  const key = `ai.window.${w.id}` as Key;
  return t(key) === key ? w.label : t(key);
}

export const sameChoice = (a: RingChoice | null, provider: string, window: string) =>
  !!a && a.provider === provider && a.window === window;

const KNOWN_ERRORS = ["login_expired", "no_data", "no_percent"];

/**
 * Friendly text for a provider error code, or the raw message. Transient
 * errors (rate limit, offline) mention how old the numbers still shown are.
 */
export function errorText(code: string, asOf: number | null): { text: string; known: boolean } {
  if (code === "rate_limited" || code === "offline") {
    if (asOf) return { text: t(`ai.err.${code}` as Key, { time: when(asOf) }), known: true };
    return { text: t(code === "offline" ? "ai.err.offline_empty" : "ai.err.rate_limited_empty"), known: true };
  }
  return KNOWN_ERRORS.includes(code) ? { text: t(`ai.err.${code}` as Key), known: true } : { text: code, known: false };
}
