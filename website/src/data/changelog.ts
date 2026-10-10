// CHANGELOG.md / CHANGELOG.vi.md (repo root), parsed at build time for the
// changelog page. Same Markdown subset as scripts/changelog.py.
import en from "../../../CHANGELOG.md?raw";
import vi from "../../../CHANGELOG.vi.md?raw";
import type { Lang } from "../i18n";

const SOURCES: Record<Lang, string> = { en, vi };

export interface Entry {
  version: string;
  date: string;
  html: string;
}

const esc = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

export function inlineHtml(text: string): string {
  return esc(text)
    .replace(/`([^`]+)`/g, "<code>$1</code>")
    .replace(/\[([^\]]+)\]\((https?:\/\/[^)\s"]+)\)/g, '<a class="btn--link" href="$2">$1</a>');
}

export function toHtml(md: string): string {
  const out: string[] = [];
  let items: string[] = [];
  const flush = () => {
    if (items.length) out.push(`<ul>${items.map((i) => `<li>${i}</li>`).join("")}</ul>`);
    items = [];
  };
  for (const raw of md.split("\n")) {
    const line = raw.trimEnd();
    if (line.startsWith("### ")) {
      flush();
      out.push(`<h4>${inlineHtml(line.slice(4))}</h4>`);
    } else if (line.startsWith("- ")) {
      items.push(inlineHtml(line.slice(2)));
    } else if (line.trim()) {
      flush();
      out.push(`<p>${inlineHtml(line)}</p>`);
    }
  }
  flush();
  return out.join("");
}

/** Every release, newest first. `## Unreleased` (notes for the next release) is left out. */
export function changelog(lang: Lang): Entry[] {
  const text = SOURCES[lang];
  const parts = text.split(/^## /m).slice(1).filter((p) => !/^Unreleased\s*$/m.test(p.split("\n")[0]));
  return parts.map((part) => {
    const [head, ...body] = part.split("\n");
    const m = head.match(/^v?(\S+)\s*(?:—|-)?\s*(.*)$/);
    return { version: m?.[1] ?? head.trim(), date: (m?.[2] ?? "").trim(), html: toHtml(body.join("\n")) };
  });
}
