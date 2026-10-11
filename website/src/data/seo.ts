// Structured data (schema.org JSON-LD) for search engines: what QuickDesk is,
// and the home page's questions and answers. Built from the same strings the
// pages show, so the two never disagree.
import { home } from "./home";
import { hasMac, release } from "./release";
import { GITHUB, href, type Lang } from "../i18n";

const SITE = "https://quickdesk.click";

const text = (html: string) =>
  html
    .replace(/<[^>]+>/g, "")
    .replace(/&amp;/g, "&")
    .replace(/\s+/g, " ")
    .trim();

/** The app itself: free, for Linux and Windows, where to download it. */
export function softwareApp(lang: Lang) {
  return {
    "@context": "https://schema.org",
    "@type": "SoftwareApplication",
    name: "QuickDesk",
    description:
      lang === "vi"
        ? "Tiện ích desktop miễn phí, mã nguồn mở: ghi chú mã hoá đầu-cuối có đồng bộ, lịch sử clipboard, trình quản lý cổng, version ngôn ngữ lập trình và hạn mức AI, gọi ra bằng một phím tắt."
        : "A free, open-source desktop helper: end-to-end encrypted synced notes, clipboard history, a port manager, programming language versions and AI usage limits, each one shortcut away.",
    applicationCategory: "UtilitiesApplication",
    operatingSystem: hasMac ? "Linux, Windows 10, Windows 11, macOS 11 or later (Apple Silicon)" : "Linux, Windows 10, Windows 11",
    softwareVersion: release.version,
    dateModified: release.date,
    url: `${SITE}${href(lang, "/")}`,
    downloadUrl: `${SITE}${href(lang, "/download")}`,
    image: `${SITE}${lang === "vi" ? "/og-vi.png" : "/og.png"}`,
    inLanguage: ["en", "vi"],
    isAccessibleForFree: true,
    license: "https://opensource.org/licenses/MIT",
    offers: { "@type": "Offer", price: "0", priceCurrency: "USD" },
    sameAs: [GITHUB],
  };
}

/** The FAQ on the home page (questions t81, t83 … t99; answers the key after). */
export function faqPage(lang: Lang) {
  const pairs: [keyof typeof home, keyof typeof home][] = [
    ["t81", hasMac ? "t82-mac" : "t82"],
    ["t83", "t84"],
    ["t85", hasMac ? "t86-mac" : "t86"],
    ["t87", "t88"],
    ["t89", "t90"],
    ["t91", "t92"],
    ["t93", hasMac ? "t94-mac" : "t94"],
    ["t95", "t96"],
    ["t97", "t98"],
    ["t99", hasMac ? "t100-mac" : "t100"],
  ];
  return {
    "@context": "https://schema.org",
    "@type": "FAQPage",
    inLanguage: lang,
    mainEntity: pairs.map(([q, a]) => ({
      "@type": "Question",
      name: text(home[q][lang]),
      acceptedAnswer: { "@type": "Answer", text: text(home[a][lang]) },
    })),
  };
}
