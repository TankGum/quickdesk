// The site's languages. English lives at /, every other language under its
// own prefix (/vi/, /zh/, /ar/ …) with the same pages. Words come from
// src/locales/ (see en.ts); long content from src/data/ and src/locales/<lang>/.
import { hasMac, macBeta } from "./data/release";
import { LOCALES } from "./locales";
import type { Strings } from "./locales/en";

/** Every language the site can have, in the order the language menu lists them. */
export const ALL_LANGS = ["en", "vi", "zh", "ja", "hi", "es", "fr", "ar", "bn", "ru", "pt", "id"] as const;
export type Lang = (typeof ALL_LANGS)[number];
/** The languages the site is built in (those with a locale in src/locales/). */
export const LANGS: Lang[] = ALL_LANGS.filter((l) => l in LOCALES);

export interface LangInfo {
  /** The language's own name, for the language menu. */
  name: string;
  /** <html lang>, hreflang. */
  tag: string;
  /** og:locale */
  og: string;
  dir: "ltr" | "rtl";
}

export const LANG_INFO: Record<Lang, LangInfo> = {
  en: { name: "English", tag: "en", og: "en_US", dir: "ltr" },
  vi: { name: "Tiếng Việt", tag: "vi", og: "vi_VN", dir: "ltr" },
  zh: { name: "简体中文", tag: "zh-Hans", og: "zh_CN", dir: "ltr" },
  ja: { name: "日本語", tag: "ja", og: "ja_JP", dir: "ltr" },
  hi: { name: "हिन्दी", tag: "hi", og: "hi_IN", dir: "ltr" },
  es: { name: "Español", tag: "es", og: "es_ES", dir: "ltr" },
  fr: { name: "Français", tag: "fr", og: "fr_FR", dir: "ltr" },
  ar: { name: "العربية", tag: "ar", og: "ar_AR", dir: "rtl" },
  bn: { name: "বাংলা", tag: "bn", og: "bn_BD", dir: "ltr" },
  ru: { name: "Русский", tag: "ru", og: "ru_RU", dir: "ltr" },
  pt: { name: "Português", tag: "pt-BR", og: "pt_BR", dir: "ltr" },
  id: { name: "Bahasa Indonesia", tag: "id", og: "id_ID", dir: "ltr" },
};

/** Link to a page in the given language. */
export function href(lang: Lang, path: string): string {
  const p = path.startsWith("/") ? path : `/${path}`;
  if (lang === "en") return p;
  return p === "/" ? `/${lang}` : `/${lang}${p}`;
}

const PREFIX = new RegExp(`^/(?:${ALL_LANGS.filter((l) => l !== "en").join("|")})(?=/|$)`);

/** Strip the language prefix: the page path shared by every language. */
export function pagePath(pathname: string): string {
  // Static builds name pages docs.html / index.html; links use /docs and /.
  const clean = pathname.replace(/\.html$/, "").replace(/\/index$/, "/");
  const p = clean.replace(PREFIX, "") || "/";
  return p.length > 1 ? p.replace(/\/$/, "") : p;
}

/**
 * Point the site's own links in translated HTML at this language's pages:
 * href="/download#macos" becomes href="/fr/download#macos" on the French
 * site. Links that already carry a language, files and other sites are left.
 */
export function localLinks(html: string, lang: Lang): string {
  if (lang === "en") return html;
  return html.replace(
    /href="\/(download|docs|changelog|privacy|sitemap)?(#[^"]*)?"/g,
    (_m, page: string | undefined, hash: string | undefined) => `href="${href(lang, page ? `/${page}` : "/")}${hash ?? ""}"`,
  );
}

/** Fill {name} placeholders. */
export function fill(text: string, vars: Record<string, string | number>): string {
  return text.replace(/\{(\w+)\}/g, (m, k: string) => (k in vars ? String(vars[k]) : m));
}

/** The site's words in a language. */
export function strings(lang: Lang): Strings {
  return (LOCALES[lang] ?? LOCALES.en!).strings;
}

export const GITHUB = "https://github.com/TankGum/quickdesk";

/** The badge next to "macOS": "soon" before a Mac build, "beta" while in development. */
export function macBadge(lang: Lang): string | null {
  if (!hasMac) return strings(lang).ui.soon;
  return macBeta ? "beta" : null;
}
