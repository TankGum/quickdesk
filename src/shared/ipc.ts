// Hand-written IPC bindings. Keep in sync with src-tauri/src/commands/*.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect } from "react";

export interface HotkeyConfig {
  notes: string;
  /** Empty string = no hotkey. */
  quickNote: string;
  clipboard: string;
  ports: string;
}

export type HotkeyTarget = "notes" | "quick-note" | "clipboard" | "ports";

export interface HotkeyCheck {
  normalized: string;
  error: string | null;
  conflicts: string[];
  inputMethodWarning: boolean;
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

export interface Note {
  id: string;
  /** May be empty: the UI then shows the first line of the body. */
  title: string;
  body: string;
  pinned: boolean;
  createdAt: number;
  updatedAt: number;
  conflictOf: string | null;
}

export interface Container {
  id: string;
  name: string;
  image: string;
  privatePort: number;
}

export interface PortEntry {
  port: number;
  addrs: string[];
  pid: number | null;
  process: string | null;
  cmdline: string | null;
  user: string | null;
  container: Container | null;
}

export interface ClipEntry {
  id: number;
  preview: string;
  chars: number;
  pinned: boolean;
  sourceApp: string | null;
  firstCopiedAt: number;
  lastCopiedAt: number;
  copyCount: number;
}

export interface ClipStatus {
  backend: string;
  state: "starting" | "running" | "retrying" | "unavailable";
  detail: string | null;
  paused: boolean;
  /** Unix ms when a timed pause ends; null while paused = until resumed. */
  pausedUntil: number | null;
  autoPaste: boolean;
}

export interface S3Config {
  endpoint: string;
  bucket: string;
  region: string;
  accessKeyId: string;
  prefix: string;
}

export interface SyncReport {
  pulled: number;
  pushed: number;
  conflicts: number;
  compacted: boolean;
}

export interface SyncStatus {
  state: "disabled" | "locked" | "idle" | "syncing" | "offline" | "error";
  lastSyncAt: number | null;
  lastError: string | null;
  lastReport: SyncReport | null;
  config: S3Config | null;
}

export type PasteMethod = "auto" | "uinput" | "portal";

export interface PasteInfo {
  method: PasteMethod;
  effective: PasteMethod;
  uinputAvailable: boolean;
  ruleInstalled: boolean;
  canEnable: boolean;
  setupCommand: string | null;
}

/** Shape of every rejected command (see `CmdError` in Rust). */
export interface CmdError {
  code: string;
  message: string;
}

export function errorMessage(e: unknown): string {
  if (typeof e === "object" && e !== null && "message" in e) return String((e as CmdError).message);
  return String(e);
}

export const api = {
  appInfo: () => invoke<AppInfo>("app_info"),
  notesCreate: (body: string, title?: string) => invoke<Note>("notes_create", { body, title }),
  notesUpdate: (id: string, patch: { title?: string; body?: string; pinned?: boolean }) =>
    invoke<Note>("notes_update", { id, ...patch }),
  notesDelete: (id: string) => invoke<void>("notes_delete", { id }),
  notesRestore: (id: string) => invoke<Note>("notes_restore", { id }),
  notesList: () => invoke<Note[]>("notes_list"),
  notesSearch: (query: string) => invoke<Note[]>("notes_search", { query }),
  portsScan: () => invoke<PortEntry[]>("ports_scan"),
  portsKill: (pid: number, force: boolean) => invoke<void>("ports_kill", { pid, force }),
  portsIsAlive: (pid: number) => invoke<boolean>("ports_is_alive", { pid }),
  portsStopContainer: (id: string) => invoke<void>("ports_stop_container", { id }),
  portsOpen: (port: number) => invoke<void>("ports_open", { port }),
  clipboardWrite: (text: string) => invoke<void>("clipboard_write", { text }),
  clipList: (limit?: number) => invoke<ClipEntry[]>("clip_list", { limit }),
  clipSearch: (query: string, limit?: number) => invoke<ClipEntry[]>("clip_search", { query, limit }),
  clipCopy: (id: number) => invoke<void>("clip_copy", { id }),
  clipPaste: (id: number) => invoke<void>("clip_paste", { id }),
  clipPasteInfo: () => invoke<PasteInfo>("clip_paste_info"),
  clipSetPasteMethod: (method: PasteMethod) => invoke<void>("clip_set_paste_method", { method }),
  clipUinputEnable: () => invoke<void>("clip_uinput_enable"),
  clipUinputDisable: () => invoke<void>("clip_uinput_disable"),
  onboarding: () => invoke<{ done: boolean; offerUinput: boolean }>("app_onboarding"),
  onboardingFinish: () => invoke<void>("app_onboarding_finish"),
  clipSetAutoPaste: (enabled: boolean) => invoke<void>("clip_set_auto_paste", { enabled }),
  clipPin: (id: number, pinned: boolean) => invoke<ClipEntry>("clip_pin", { id, pinned }),
  clipDelete: (id: number) => invoke<void>("clip_delete", { id }),
  clipClear: (keepPinned: boolean) => invoke<number>("clip_clear", { keepPinned }),
  clipStatus: () => invoke<ClipStatus>("clip_status"),
  clipSetPaused: (paused: boolean, minutes?: number) => invoke<void>("clip_set_paused", { paused, minutes }),
  syncStatus: () => invoke<SyncStatus>("sync_status"),
  syncConnect: (config: S3Config, secret: string) => invoke<{ initialized: boolean }>("sync_connect", { config, secret }),
  syncCreate: (passphrase: string) => invoke<string>("sync_create", { passphrase }),
  syncUnlock: (secret: string) => invoke<void>("sync_unlock", { secret }),
  syncNow: () => invoke<void>("sync_now"),
  syncDisconnect: () => invoke<void>("sync_disconnect"),
  focusReport: (report: FocusReport) => invoke<void>("diag_focus_report", { report }),
  focusStats: () => invoke<FocusStats>("diag_focus_stats"),
  quit: () => invoke<void>("app_quit"),
  show: (target: HotkeyTarget | "main") => invoke<void>("app_show", { target }),
  hotkeysGet: () => invoke<{ config: HotkeyConfig; strategy: AppInfo["hotkeyStrategy"] }>("hotkeys_get"),
  hotkeysCheck: (target: HotkeyTarget, accel: string) => invoke<HotkeyCheck>("hotkeys_check", { target, accel }),
  hotkeysSet: (target: HotkeyTarget, accel: string) => invoke<HotkeyConfig>("hotkeys_set", { target, accel }),
  hotkeysReset: () => invoke<HotkeyConfig>("hotkeys_reset"),
  hotkeysSuspend: () => invoke<void>("hotkeys_suspend"),
  hotkeysResume: () => invoke<void>("hotkeys_resume"),
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

/** Re-run `fn` whenever the backend emits `event` (to any window). */
export function useBackendEvent(event: string, fn: () => void) {
  useEffect(() => {
    const unlisten = listen(event, () => fn());
    return () => {
      void unlisten.then((f) => f());
    };
  }, [event, fn]);
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
