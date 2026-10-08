/** Markdown → readable plain text, for list previews and headings. */
export function markdownToText(md: string): string {
  return md
    .replace(/^```[^\n]*\n?/gm, "")
    .replace(/^\s{0,3}#{1,6}\s+/gm, "")
    .replace(/^\s*>\s?/gm, "")
    .replace(/^(\s*)[-*+]\s+\[[xX]\]\s+/gm, "$1☑ ")
    .replace(/^(\s*)[-*+]\s+\[ \]\s+/gm, "$1☐ ")
    .replace(/^(\s*)[-*+]\s+/gm, "$1• ")
    .replace(/(\*\*|__|~~|`)/g, "")
    .replace(/(^|[^\\])[*_](?=\S)([^*_\n]+?)[*_](?!\w)/g, "$1$2")
    .replace(/\\([\\`*_{}[\]()#+\-.!~>|])/g, "$1");
}
