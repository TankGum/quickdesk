import { useCallback, useEffect, useRef, useState } from "react";

import { ClipboardTab } from "../modules/clipboard/ClipboardTab";
import { NotesTab } from "../modules/notes/NotesTab";
import { PortsTab } from "../modules/ports/PortsTab";
import { SyncFooter } from "../modules/sync/SyncFooter";
import { SyncSettings } from "../modules/sync/SyncSettings";
import { HotkeySettings } from "../modules/settings/HotkeySettings";
import { PasteSettings } from "../modules/settings/PasteSettings";
import { api, AppInfo, currentWindow, FocusStats, hideWindow, useShown } from "../shared/ipc";
import { Welcome } from "./Welcome";

const TABS = ["notes", "clipboard", "ports", "settings"] as const;
type Tab = (typeof TABS)[number];

export function Main() {
  const [tab, setTab] = useState<Tab>("notes");
  const [onboarded, setOnboarded] = useState<boolean | null>(null);
  useEffect(() => {
    void api.onboarding().then((o) => setOnboarded(o.done));
  }, []);
  // Bumped on every show so the active tab can re-focus its input.
  const [shownAt, setShownAt] = useState(0);

  useShown((s) => {
    if (s.tab && (TABS as readonly string[]).includes(s.tab)) setTab(s.tab as Tab);
    setShownAt(s.shownAtMs);
  });

  // Hotkey pressed while this window is focused: same tab → hide, else switch.
  const tabRef = useRef(tab);
  tabRef.current = tab;
  useEffect(() => {
    const unlisten = currentWindow.listen<string>("window://toggle-tab", ({ payload }) => {
      if (payload === tabRef.current) {
        void hideWindow();
      } else if ((TABS as readonly string[]).includes(payload)) {
        setTab(payload as Tab);
        setShownAt(Date.now());
      }
    });
    return () => {
      void unlisten.then((f) => f());
    };
  }, []);

  if (onboarded === false) {
    return <Welcome onDone={() => setOnboarded(true)} />;
  }

  return (
    <div className="main">
      <nav className="tabs">
        {TABS.map((t) => (
          <button key={t} className={t === tab ? "active" : ""} onClick={() => setTab(t)}>
            {t[0].toUpperCase() + t.slice(1)}
          </button>
        ))}
      </nav>
      <section className="content">
        {tab === "notes" && <NotesTab focusSignal={shownAt} />}
        {tab === "settings" && <Settings />}
        {tab === "ports" && <PortsTab focusSignal={shownAt} />}
        {tab === "clipboard" && <ClipboardTab focusSignal={shownAt} />}
      </section>
      <SyncFooter onOpenSettings={() => setTab("settings")} />
    </div>
  );
}

function Settings() {
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [stats, setStats] = useState<FocusStats | null>(null);

  const refresh = useCallback(() => {
    void api.appInfo().then(setInfo);
    void api.focusStats().then(setStats);
  }, []);

  useEffect(() => {
    refresh();
    const id = setInterval(refresh, 2000);
    return () => clearInterval(id);
  }, [refresh]);

  if (!info) return null;
  return (
    <div className="settings">
      <h2>Sync</h2>
      <SyncSettings />

      <h2>Hotkeys</h2>
      <HotkeySettings />

      <h2>Auto-paste</h2>
      <PasteSettings />

      <h2>Popup focus test</h2>
      {stats && stats.total > 0 ? (
        <>
          <p>
            Focused <b>{stats.focused}/{stats.total}</b> · latency p50 {stats.latencyP50} ms · max{" "}
            {stats.latencyMax} ms
          </p>
          <table>
            <thead>
              <tr><th>Window</th><th>Document</th><th>Input</th><th>Latency</th></tr>
            </thead>
            <tbody>
              {stats.recent.map((r, i) => (
                <tr key={i}>
                  <td>{r.label}</td>
                  <td>{r.documentFocused ? "✓" : "✗"}</td>
                  <td>{r.inputFocused ? "✓" : "✗"}</td>
                  <td>{(r.reportedAtMs ?? 0) - r.sentAtMs} ms</td>
                </tr>
              ))}
            </tbody>
          </table>
        </>
      ) : (
        <p className="muted">Press a popup hotkey from another app to record a sample.</p>
      )}

      <h2>About</h2>
      <dl>
        <dt>Version</dt>
        <dd>{info.version}</dd>
        <dt>Session</dt>
        <dd>{info.os} · {info.wayland ? "Wayland" : "X11/native"} · {info.desktop || "unknown"}</dd>
        <dt>Device</dt>
        <dd><code>{info.deviceId}</code></dd>
        <dt>Data</dt>
        <dd><code>{info.dataDir}</code></dd>
      </dl>
      <button className="danger" onClick={() => void api.quit()}>Quit QuickDesk</button>
    </div>
  );
}

