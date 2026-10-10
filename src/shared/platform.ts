// Which desktop the UI runs on. The webview's user agent says it (WebView2
// on Windows, WebKitGTK on Linux); the website demo follows the visitor's.
export const isWindows = typeof navigator !== "undefined" && /Windows/.test(navigator.userAgent);

/** "Super+Alt+N" as people on this system call the keys ("Win+Alt+N" on Windows). */
export function keyLabel(accelerator: string): string {
  return isWindows ? accelerator.replace(/\bSuper\b/g, "Win") : accelerator;
}
