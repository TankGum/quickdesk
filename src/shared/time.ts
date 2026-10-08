import { locale, t } from "./i18n";

const UNITS: [Intl.RelativeTimeFormatUnit, number][] = [
  ["year", 365 * 24 * 3600e3],
  ["month", 30 * 24 * 3600e3],
  ["week", 7 * 24 * 3600e3],
  ["day", 24 * 3600e3],
  ["hour", 3600e3],
  ["minute", 60e3],
];

const formatters = new Map<string, Intl.RelativeTimeFormat>();

/** "5 minutes ago" / "5 phút trước", "yesterday", "just now", in the UI language. */
export function relativeTime(ms: number, now = Date.now()): string {
  const loc = locale();
  let rtf = formatters.get(loc);
  if (!rtf) {
    rtf = new Intl.RelativeTimeFormat(loc, { numeric: "auto" });
    formatters.set(loc, rtf);
  }
  const diff = ms - now;
  for (const [unit, size] of UNITS) {
    if (Math.abs(diff) >= size) return rtf.format(Math.round(diff / size), unit);
  }
  return t("time.justNow");
}

export function absoluteTime(ms: number): string {
  return new Date(ms).toLocaleString(locale());
}

export function clockTime(ms: number): string {
  return new Date(ms).toLocaleTimeString(locale(), { hour: "2-digit", minute: "2-digit" });
}
