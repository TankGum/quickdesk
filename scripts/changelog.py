#!/usr/bin/env python3
"""Release notes from CHANGELOG.md (English) and CHANGELOG.vi.md (Vietnamese).

    python3 scripts/changelog.py check 0.2.2     fail unless both files have the version
    python3 scripts/changelog.py notes 0.2.2     Markdown for GitHub Releases (both languages)

publish_r2.py imports `section` and `to_html` for update.json and the download page.
"""

import html
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FILES = {"en": ROOT / "CHANGELOG.md", "vi": ROOT / "CHANGELOG.vi.md"}


def section(lang: str, version: str) -> str | None:
    """Body of `## <version> …` (without the heading), or None."""
    text = FILES[lang].read_text()
    head = re.compile(rf"^## v?{re.escape(version)}(\s|$).*$", re.M)
    m = head.search(text)
    if not m:
        return None
    rest = text[m.end():]
    nxt = re.search(r"^## ", rest, re.M)
    body = (rest[: nxt.start()] if nxt else rest).strip()
    return body or None


def _inline(text: str) -> str:
    out = html.escape(text, quote=False)
    out = re.sub(r"`([^`]+)`", r"<code>\1</code>", out)
    # [text](https://…) links; only http(s), so notes cannot inject script URLs.
    return re.sub(r"\[([^\]]+)\]\((https?://[^)\s\"]+)\)", r'<a class="btn--link" href="\2">\1</a>', out)


def to_html(md: str) -> str:
    """The small Markdown subset the changelog uses: ### groups, - bullets,
    `code` and [links](https://…)."""
    parts, items = [], []

    def flush():
        if items:
            parts.append("<ul>" + "".join(f"<li>{i}</li>" for i in items) + "</ul>")
            items.clear()

    for line in md.splitlines():
        line = line.rstrip()
        if line.startswith("### "):
            flush()
            parts.append(f"<h4>{_inline(line[4:])}</h4>")
        elif line.startswith("- "):
            items.append(_inline(line[2:]))
        elif line.strip():
            flush()
            parts.append(f"<p>{_inline(line)}</p>")
    flush()
    return "".join(parts)


def main() -> None:
    if len(sys.argv) != 3 or sys.argv[1] not in ("check", "notes"):
        sys.exit(__doc__)
    cmd, version = sys.argv[1], sys.argv[2].lstrip("v")
    found = {lang: section(lang, version) for lang in FILES}
    missing = [FILES[lang].name for lang, body in found.items() if not body]
    if missing:
        sys.exit(f"no '## {version}' section in {', '.join(missing)}: write the release notes first")
    if cmd == "notes":
        print(found["en"])
        print("\n---\n\n**Tiếng Việt**\n")
        print(found["vi"])


if __name__ == "__main__":
    main()
