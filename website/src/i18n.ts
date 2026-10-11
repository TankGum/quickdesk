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
  // Static builds name pages docs.html / index.html; links use /docs and /.
  const clean = pathname.replace(/\.html$/, "").replace(/\/index$/, "/");
  const p = clean.replace(/^\/vi(?=\/|$)/, "") || "/";
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
    footer: "QuickDesk · MIT License.",
    tagline: "Notes, clipboard history, ports and AI limits. One shortcut away.",
    systems: "Systems",
    soon: "soon",
    product: "Product",
    project: "Project",
    issues: "Report a problem",
    sitemap: "Site map",
    skip: "Skip to content",
    menu: "Menu",
    close: "Close",
    faq: "FAQ",
    theme: "Switch between light and dark",
    themeShort: "Light / dark",
  },
  vi: {
    home: "Trang chủ",
    download: "Tải về",
    docs: "Tài liệu",
    changelog: "Thay đổi",
    privacy: "Quyền riêng tư",
    github: "GitHub",
    language: "Ngôn ngữ",
    footer: "QuickDesk · Giấy phép MIT.",
    tagline: "Ghi chú, lịch sử clipboard, cổng và hạn mức AI. Chỉ cách một phím tắt.",
    systems: "Hệ điều hành",
    soon: "sắp có",
    product: "Sản phẩm",
    project: "Dự án",
    issues: "Báo lỗi",
    sitemap: "Bản đồ trang web",
    skip: "Bỏ qua tới nội dung",
    menu: "Menu",
    close: "Đóng",
    faq: "FAQ",
    theme: "Đổi giao diện sáng / tối",
    themeShort: "Sáng / tối",
  },
} as const;

export const GITHUB = "https://github.com/TankGum/quickdesk";
