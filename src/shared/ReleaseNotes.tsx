import enLog from "../../CHANGELOG.md?raw";
import viLog from "../../CHANGELOG.vi.md?raw";
import { lang } from "./i18n";

/** A version's section of the changelog bundled into the app, in the UI language. */
export function bundledNotes(version: string): string | null {
  const text = lang() === "vi" ? viLog : enLog;
  const lines = text.split("\n");
  const start = lines.findIndex((l) => new RegExp(`^## v?${version.replace(/\./g, "\\.")}(\\s|$)`).test(l));
  if (start < 0) return null;
  const end = lines.findIndex((l, i) => i > start && l.startsWith("## "));
  const body = lines.slice(start + 1, end < 0 ? undefined : end).join("\n").trim();
  return body || null;
}

/** Pick the notes in the UI language from update.json (`notes` / `notes_vi`). */
export function pickNotes(notes: string | null, notesVi: string | null): string | null {
  return (lang() === "vi" ? notesVi || notes : notes || notesVi) || null;
}

function inline(text: string) {
  return text.split(/(`[^`]+`)/).map((part, i) => (part.startsWith("`") && part.endsWith("`") ? <code key={i}>{part.slice(1, -1)}</code> : part));
}

/** Renders the changelog's Markdown subset (### groups, - bullets) without HTML injection. */
export function ReleaseNotes({ markdown }: { markdown: string }) {
  const blocks: JSX.Element[] = [];
  let items: string[] = [];
  const flush = () => {
    if (items.length) blocks.push(<ul key={blocks.length}>{items.map((t, i) => <li key={i}>{inline(t)}</li>)}</ul>);
    items = [];
  };
  for (const raw of markdown.split("\n")) {
    const line = raw.trimEnd();
    if (line.startsWith("### ")) {
      flush();
      blocks.push(<h4 key={blocks.length}>{line.slice(4)}</h4>);
    } else if (line.startsWith("- ")) {
      items.push(line.slice(2));
    } else if (line.trim()) {
      flush();
      blocks.push(<p key={blocks.length}>{inline(line)}</p>);
    }
  }
  flush();
  return <div className="release-notes">{blocks}</div>;
}
