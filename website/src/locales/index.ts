// Every language of the site. English and Vietnamese keep their long content
// in src/data/ (home.ts, docs.ts, privacy.ts) and CHANGELOG*.md at the repo
// root; the other languages bring theirs in src/locales/<lang>/.
// Adding a language: copy a locale folder, translate it, add it here and to
// ALL_LANGS / LANG_INFO in src/i18n.ts.
import type { HomeKey } from "../data/home";
import type { Lang } from "../i18n";
import en, { type Strings } from "./en";
import vi from "./vi";
import ar from "./ar";
import bn from "./bn";
import es from "./es";
import fr from "./fr";
import hi from "./hi";
import ja from "./ja";
import ru from "./ru";
import pt from "./pt";
import id from "./id";
import zh from "./zh";

export interface Content {
  /** Every home page string, by its key in src/data/home.ts. */
  home: Record<HomeKey, string>;
  /** Docs sections by id (src/data/docs.ts). Links like /download are made local. */
  docs: Record<string, { title: string; html: string }>;
  privacy: { title: string; updated: string; html: string };
  /** The changelog in the same Markdown as CHANGELOG.md; releases missing here show in English. */
  changelog: string;
}

export interface Locale {
  strings: Strings;
  /** Long content; English and Vietnamese have theirs in src/data/. */
  content?: Content;
}

export const LOCALES: Partial<Record<Lang, Locale>> = {
  en: { strings: en },
  vi: { strings: vi },
  zh,
  ja,
  hi,
  es,
  fr,
  ar,
  bn,
  ru,
  pt,
  id,
};
