// Hand-written IPC bindings for M1. Replaced by tauri-specta output once
// module commands arrive (M2).
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect } from "react";

export interface HotkeyConfig {
  notes: string;
  clipboard: string;
  ports: string;
}

export interface AppInfo {
  version: string;
  os: string;
  wayland: boolean;
  desktop: string;
  hotkeyStrategy: "Plugin" | "GnomeKeybinding" | "Manual";
  hotkeys: HotkeyConfig;
  deviceId: string;
  dataDir: string;
}

export interface FocusReport {
  label: string;
  documentFocused: boolean;
  inputFocused: boolean;
  sentAtMs: number;
  shownAtMs: number;
  reportedAtMs?: number;
}

export interface FocusStats {
  total: number;
  focused: number;
  latencyP50: number | null;
  latencyMax: number | null;
  recent: FocusReport[];
}

export interface Shown {
  sentAtMs: number;
  shownAtMs: number;
  tab: string | null;
}

export const api = {
  appInfo: () => invoke<AppInfo>("app_info"),
  focusReport: (report: FocusReport) => invoke<void>("diag_focus_report", { report }),
  focusStats: () => invoke<FocusStats>("diag_focus_stats"),
  quit: () => invoke<void>("app_quit"),
};

export const currentWindow = getCurrentWebviewWindow();

export function hideWindow() {
  return currentWindow.hide();
}

/**
 * Runs `onShown` every time Rust brings this window up, then reports whether
 * keyboard focus really landed in `input` (the M1 focus acceptance test).
 */
export function useShown(onShown: (s: Shown) => void, input?: () => HTMLElement | null) {
  useEffect(() => {
    const unlisten = currentWindow.listen<Shown>("window://shown", ({ payload }) => {
      onShown(payload);
      // Give the compositor a frame to deliver focus before measuring.
      requestAnimationFrame(() => {
        const el = input?.() ?? null;
        el?.focus();
        setTimeout(() => {
          void api.focusReport({
            label: currentWindow.label,
            documentFocused: document.hasFocus(),
            inputFocused: el !== null && document.activeElement === el,
            sentAtMs: payload.sentAtMs,
            shownAtMs: payload.shownAtMs,
          });
        }, 50);
      });
    });
    return () => {
      void unlisten.then((f) => f());
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
}

export function useEscapeToHide() {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") void hideWindow();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
}
