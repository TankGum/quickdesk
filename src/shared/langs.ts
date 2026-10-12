// The UI languages, the same as the website's (website/src/i18n.ts). No
// imports here: the demo's event bus reads this before the app loads.

export const LANGS = ["en", "vi", "zh", "ja", "hi", "es", "fr", "ar", "bn", "ru", "pt", "id"] as const;
export type Lang = (typeof LANGS)[number];

/** Each language in its own words, for the language pickers. */
export const LANG_NAMES: Record<Lang, string> = {
  en: "English",
  vi: "Tiếng Việt",
  zh: "简体中文",
  ja: "日本語",
  hi: "हिन्दी",
  es: "Español",
  fr: "Français",
  ar: "العربية",
  bn: "বাংলা",
  ru: "Русский",
  pt: "Português",
  id: "Bahasa Indonesia",
};

export function isLang(value: unknown): value is Lang {
  return (LANGS as readonly unknown[]).includes(value);
}
