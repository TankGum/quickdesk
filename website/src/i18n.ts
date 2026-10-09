// Two languages, one route tree each: English at /, Vietnamese at /vi/.
export type Lang = "en" | "vi";

/** Link to a page in the given language. */
export function href(lang: Lang, path: string): string {
  const p = path.startsWith("/") ? path : `/${path}`;
  return lang === "vi" ? (p === "/" ? "/vi" : `/vi${p}`) : p;
}

/** The same page in the other language (for the EN / VI switch and hreflang). */
export function otherLang(lang: Lang): Lang {
  return lang === "vi" ? "en" : "vi";
}

/** Strip the /vi prefix: the page path shared by both languages. */
export function pagePath(pathname: string): string {
  const p = pathname.replace(/^\/vi(?=\/|$)/, "") || "/";
  return p.length > 1 ? p.replace(/\/$/, "") : p;
}

export const ui = {
  en: {
    home: "Home",
    download: "Download",
    docs: "Docs",
    changelog: "Changelog",
    privacy: "Privacy",
    github: "GitHub",
    language: "Language",
    footer: "QuickDesk · MIT License. Made for Linux, with care.",
    skip: "Skip to content",
  },
  vi: {
    home: "Trang chủ",
    download: "Tải về",
    docs: "Tài liệu",
    changelog: "Thay đổi",
    privacy: "Quyền riêng tư",
    github: "GitHub",
    language: "Ngôn ngữ",
    footer: "QuickDesk · Giấy phép MIT. Làm cho Linux, bằng cả tấm lòng.",
    skip: "Bỏ qua tới nội dung",
  },
} as const;

export const GITHUB = "https://github.com/TankGum/quickdesk";
