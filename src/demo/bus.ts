// Demo build only: an in-page stand-in for Tauri's event system, plus the
// bridge to the landing page that embeds the demo windows in iframes.

type Handler = (event: { event: string; payload: unknown }) => void;

const handlers = new Map<string, Set<Handler>>();

export function on(event: string, fn: Handler): () => void {
  let set = handlers.get(event);
  if (!set) handlers.set(event, (set = new Set()));
  set.add(fn);
  return () => set!.delete(fn);
}

export function emit(event: string, payload?: unknown) {
  handlers.get(event)?.forEach((fn) => fn({ event, payload }));
}

export const params = new URLSearchParams(location.search);

// ?theme=dark|light forces a theme (the landing page is dark).
const theme = params.get("theme");
if (theme) document.documentElement.dataset.theme = theme;

// The app uses the system font, which on Ubuntu 24.04 is Ubuntu Sans. Load it
// so visitors on any OS see the app as it looks on Ubuntu.
const fonts = document.createElement("link");
fonts.rel = "stylesheet";
fonts.href = "https://fonts.googleapis.com/css2?family=Ubuntu+Sans:wght@400;500;600;700&family=Ubuntu+Sans+Mono:wght@400;500&display=swap";
document.head.appendChild(fonts);
document.documentElement.style.setProperty("--font", '"Ubuntu Sans", Ubuntu, system-ui, sans-serif');
document.documentElement.style.setProperty("--mono", '"Ubuntu Sans Mono", "Ubuntu Mono", ui-monospace, monospace');

/** Tell the embedding page something happened (hide, toast, …). */
export function toPage(type: string, data: Record<string, unknown> = {}) {
  if (window.parent !== window) window.parent.postMessage({ source: "quickdesk-demo", type, ...data }, "*");
}

if (window.parent !== window) {
  // The app focuses its search box whenever a window is shown. Inside the page
  // that would pull keyboard focus (and the scroll position) into the iframe,
  // so only allow it once the visitor is actually using this window.
  const focus = HTMLElement.prototype.focus;
  HTMLElement.prototype.focus = function (this: HTMLElement, opts?: FocusOptions) {
    if (document.hasFocus()) focus.call(this, { ...opts, preventScroll: true });
  };
  window.addEventListener("pointerdown", () => toPage("interact"), true);
}

// ?tab=… opens that tab, as if its hotkey had been pressed.
if (params.get("tab")) {
  setTimeout(() => emit("window://shown", { sentAtMs: Date.now(), shownAtMs: Date.now(), tab: params.get("tab") }), 150);
}

// The page shows a window: replay what Rust sends when a hotkey fires.
window.addEventListener("message", ({ data }) => {
  if (!data || data.source !== "quickdesk-page") return;
  if (data.type === "show") {
    const now = Date.now();
    emit("window://shown", { sentAtMs: now, shownAtMs: now, tab: data.tab ?? null });
  }
});
