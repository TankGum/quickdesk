// sitemap.xml: every page in both languages, each listing its other-language
// twin (hreflang), dated by the newest release. robots.txt points here.
import type { APIRoute } from "astro";
import { release } from "../data/release";
import { href, type Lang } from "../i18n";

const PAGES = ["/", "/download", "/docs", "/changelog", "/privacy", "/sitemap"];
const LANGS: Lang[] = ["en", "vi"];

export const GET: APIRoute = ({ site }) => {
  const abs = (lang: Lang, page: string) => new URL(href(lang, page), site).toString();
  const urls = PAGES.flatMap((page) =>
    LANGS.map((lang) => {
      const alternates = [
        ...LANGS.map((l) => `    <xhtml:link rel="alternate" hreflang="${l}" href="${abs(l, page)}"/>`),
        `    <xhtml:link rel="alternate" hreflang="x-default" href="${abs("en", page)}"/>`,
      ].join("\n");
      return `  <url>\n    <loc>${abs(lang, page)}</loc>\n    <lastmod>${release.date}</lastmod>\n${alternates}\n  </url>`;
    }),
  );
  const xml = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">
${urls.join("\n")}
</urlset>
`;
  return new Response(xml, { headers: { "Content-Type": "application/xml; charset=utf-8" } });
};
