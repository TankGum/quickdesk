// Which desktop the UI runs on. The webview's user agent says it (WebView2
// on Windows, WKWebView on macOS, WebKitGTK on Linux); the website demo
// follows the visitor's.
const ua = typeof navigator !== "undefined" ? navigator.userAgent : "";
export const isWindows = /Windows/.test(ua);
export const isMac = /Macintosh|Mac OS X/.test(ua);

/** macOS writes modifiers as symbols, in this order, with no "+": ⌃⌥⇧⌘N. */
const MAC_MODS: [string, string][] = [
  ["Ctrl", "⌃"],
  ["Alt", "⌥"],
  ["Shift", "⇧"],
  ["Super", "⌘"],
];

/** "Super+Alt+N" as people on this system call the keys ("Win+Alt+N" on Windows, "⌃⌥N" on a Mac). */
export function keyLabel(accelerator: string): string {
  if (isMac && accelerator) {
    const parts = accelerator.split("+");
    const key = parts[parts.length - 1];
    const mods = MAC_MODS.filter(([name]) => parts.slice(0, -1).includes(name)).map(([, sym]) => sym);
    return mods.join("") + key;
  }
  return isWindows ? accelerator.replace(/\bSuper\b/g, "Win") : accelerator;
}
