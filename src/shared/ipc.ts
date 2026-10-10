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

export type ClipKind = "text" | "image" | "files";

export interface ClipEntry {
  id: number;
  kind: ClipKind;
  /** Text: first 400 characters. Files: the paths, one per line. */
  preview: string;
  chars: number;
  pinned: boolean;
  sourceApp: string | null;
  firstCopiedAt: number;
  lastCopiedAt: number;
  copyCount: number;
  byteSize: number;
  width: number | null;
  height: number | null;
  /** data: URL of a small PNG, for images. */
  thumb: string | null;
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
  /** The QuickDesk Cloud sync account this device uses. */
  account: string | null;
  /** "bucket_removed": the old own-bucket setup was dropped (0.3.1). */
  notice: string | null;
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

export interface UsageWindow {
  id: string;
  label: string;
  usedPercent: number | null;
  resetsAt: number | null;
  detail: string | null;
}

export interface ProviderUsage {
  provider: string;
  name: string;
  plan: string | null;
  windows: UsageWindow[];
  source: "live" | "local";
  asOf: number | null;
  error: string | null;
}

export interface RingChoice {
  provider: string;
  window: string;
}

export interface UsageSnapshot {
  providers: ProviderUsage[];
  updatedAt: number | null;
  /** Percent the top-bar ring shows, and which limit that is. */
  headline: number | null;
  ringShows: RingChoice | null;
  /** The user's choice; null = automatic. */
  ring: RingChoice | null;
  trayEnabled: boolean;
}

/** One language on this computer (crates/qd-runtimes). */
export interface Runtime {
  id: string;
  name: string;
  /** nvm | rustup | uv; null: read-only (system or unsupported manager). */
  manager: string | null;
  managerVersion: string | null;
  /** What a new terminal runs. */
  active: { version: string; path: string; managed: boolean } | null;
  /** An InstalledVersion.id. */
  default: string | null;
  installed: InstalledVersion[];
  issue: RuntimeIssue | null;
  /** Versions are typed (rustup), not picked from a list. */
  freeInput: boolean;
  projectFile: string | null;
  /** No manager yet; this one ("mise") can be installed to manage it. */
  installManager: string | null;
}

export interface InstalledVersion {
  id: string;
  version: string;
  path: string;
  bytes: number;
  isDefault: boolean;
}

export interface RuntimeIssue {
  kind: "not_loaded" | "shadowed" | "other";
  expected: string;
  /** Lines that fix it in the shell's rc file (empty: no automatic fix). */
  fix: string[];
}

export interface AvailableVersion {
  id: string;
  version: string;
  tag: string | null;
  /** Release line (`22`, `3.12`); empty when the tool has none. */
  line: string;
  installed: boolean;
}

/** A newer release in the same line as an installed version. */
export interface RuntimeUpdate {
  lang: string;
  from: string;
  fromVersion: string;
  to: string;
  toVersion: string;
}

export interface AvailableList {
  versions: AvailableVersion[];
  fetchedAt: number;
  /** Offline: these are from fetchedAt. */
  stale: boolean;
}

export type RuntimeAction = "install" | "uninstall" | "set_default" | "install_manager" | "upgrade";

export interface RuntimeJob {
  lang: string;
  action: RuntimeAction;
  version: string;
  running: boolean;
  percent: number | null;
  line: string;
  error: string | null;
  cancelled: boolean;
  startedAt: number;
  /** upgrade: the version being replaced (it stays installed). */
  from: string | null;
}

/** "Apply right away in open terminals": QuickDesk's prompt hook. */
export interface AutoApplyState {
  on: boolean;
  file: string;
  line: string;
  script: string;
}

export interface ShellFix {
  file: string;
  lines: string[];
  current: string[];
}

/** Self-update state (src-tauri/src/updater.rs). */
export type UpdateStatus =
  | { state: "idle" }
  | { state: "checking" }
  | { state: "upToDate"; checkedAt: number }
  | { state: "available"; version: string; notes: string | null; notesVi: string | null }
  | { state: "downloading"; version: string; downloaded: number; total: number | null }
  | { state: "installing"; version: string }
  | { state: "error"; message: string; version: string | null };

export interface UpdateInfo {
  currentVersion: string;
  autoCheck: boolean;
  status: UpdateStatus;
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
  runtimesScan: (force: boolean) => invoke<Runtime[]>("runtimes_scan", { force }),
  runtimesAvailable: (lang: string) => invoke<AvailableList>("runtimes_available", { lang }),
  runtimesRun: (lang: string, action: RuntimeAction, version: string) => invoke<void>("runtimes_run", { lang, action, version }),
  runtimesCancel: () => invoke<void>("runtimes_cancel"),
  runtimesUpgrade: (lang: string, from: string, to: string) => invoke<void>("runtimes_upgrade", { lang, from, to }),
  runtimesUpdates: () => invoke<RuntimeUpdate[]>("runtimes_updates"),
  runtimesJob: () => invoke<RuntimeJob | null>("runtimes_job"),
  runtimesShellFix: (lang: string) => invoke<ShellFix>("runtimes_shell_fix", { lang }),
  runtimesShellApply: (lang: string) => invoke<{ file: string; backup: string | null }>("runtimes_shell_apply", { lang }),
  runtimesShellUndo: () => invoke<boolean>("runtimes_shell_undo"),
  runtimesPickFolder: () => invoke<string | null>("runtimes_pick_folder"),
  runtimesAutoApply: () => invoke<AutoApplyState>("runtimes_auto_apply"),
  runtimesSetAutoApply: (on: boolean) => invoke<{ file: string; backup: string | null }>("runtimes_set_auto_apply", { on }),
  runtimesProjectGet: (dir: string, lang: string) => invoke<{ file: string; content: string | null }>("runtimes_project_get", { dir, lang }),
  runtimesProjectSet: (dir: string, lang: string, version: string) => invoke<string>("runtimes_project_set", { dir, lang, version }),
  clipboardWrite: (text: string) => invoke<void>("clipboard_write", { text }),
  clipList: (limit?: number, kind?: ClipKind) => invoke<ClipEntry[]>("clip_list", { limit, kind }),
  clipSearch: (query: string, limit?: number, kind?: ClipKind) =>
    invoke<ClipEntry[]>("clip_search", { query, limit, kind }),
  clipCopy: (id: number) => invoke<void>("clip_copy", { id }),
  clipPaste: (id: number) => invoke<void>("clip_paste", { id }),
  clipPasteInfo: () => invoke<PasteInfo>("clip_paste_info"),
  clipSetPasteMethod: (method: PasteMethod) => invoke<void>("clip_set_paste_method", { method }),
  clipUinputEnable: () => invoke<void>("clip_uinput_enable"),
  clipUinputDisable: () => invoke<void>("clip_uinput_disable"),
  autostartGet: () => invoke<boolean>("app_autostart_get"),
  autostartSet: (enabled: boolean) => invoke<boolean>("app_autostart_set", { enabled }),
  onboarding: () => invoke<{ done: boolean; offerUinput: boolean }>("app_onboarding"),
  onboardingFinish: () => invoke<void>("app_onboarding_finish"),
  clipSetAutoPaste: (enabled: boolean) => invoke<void>("clip_set_auto_paste", { enabled }),
  clipPin: (id: number, pinned: boolean) => invoke<ClipEntry>("clip_pin", { id, pinned }),
  clipDelete: (id: number) => invoke<void>("clip_delete", { id }),
  clipClear: (keepPinned: boolean) => invoke<number>("clip_clear", { keepPinned }),
  clipStatus: () => invoke<ClipStatus>("clip_status"),
  clipSetPaused: (paused: boolean, minutes?: number) => invoke<void>("clip_set_paused", { paused, minutes }),
  syncStatus: () => invoke<SyncStatus>("sync_status"),
  syncEnable: (passphrase: string) => invoke<{ syncCode: string; recoveryKey: string }>("sync_enable", { passphrase }),
  syncJoin: (code: string, secret: string) => invoke<void>("sync_join", { code, secret }),
  syncCode: () => invoke<string>("sync_code"),
  syncUnlock: (secret: string) => invoke<void>("sync_unlock", { secret }),
  syncNow: () => invoke<void>("sync_now"),
  syncDisconnect: (deleteCloud: boolean) => invoke<void>("sync_disconnect", { deleteCloud }),
  aiUsage: () => invoke<UsageSnapshot>("ai_usage_get"),
  aiUsageRefresh: () => invoke<UsageSnapshot>("ai_usage_refresh"),
  aiSetTray: (enabled: boolean) => invoke<void>("ai_set_tray", { enabled }),
  aiSetRing: (choice: RingChoice | null) => invoke<void>("ai_set_ring", { choice }),
  focusReport: (report: FocusReport) => invoke<void>("diag_focus_report", { report }),
  focusStats: () => invoke<FocusStats>("diag_focus_stats"),
  updateStatus: () => invoke<UpdateInfo>("update_status"),
  updateCheck: () => invoke<UpdateInfo>("update_check"),
  updateInstall: () => invoke<void>("update_install"),
  updateSetAuto: (enabled: boolean) => invoke<UpdateInfo>("update_set_auto", { enabled }),
  quit: () => invoke<void>("app_quit"),
  show: (target: HotkeyTarget | "main" | "ai" | "usage") => invoke<void>("app_show", { target }),
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
