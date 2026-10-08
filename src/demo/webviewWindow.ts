// Demo build only: replaces `@tauri-apps/api/webviewWindow`.
import { on, params, toPage } from "./bus";

const label = params.get("window") ?? "main";

const demoWindow = {
  label,
  hide: () => {
    toPage("hide", { label });
    return Promise.resolve();
  },
  show: () => Promise.resolve(),
  isVisible: () => Promise.resolve(true),
  listen<T>(event: string, fn: (e: { event: string; payload: T }) => void): Promise<() => void> {
    return Promise.resolve(on(event, fn as (e: { event: string; payload: unknown }) => void));
  },
};

export function getCurrentWebviewWindow() {
  return demoWindow;
}
