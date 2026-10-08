import { useCallback, useEffect, useState } from "react";

import { ClipboardTab } from "../modules/clipboard/ClipboardTab";
import { NotesTab } from "../modules/notes/NotesTab";
import { PortsTab } from "../modules/ports/PortsTab";
import { api, AppInfo, FocusStats, useShown } from "../shared/ipc";

const TABS = ["notes", "clipboard", "ports", "settings"] as const;
type Tab = (typeof TABS)[number];

export function Main() {
  const [tab, setTab] = useState<Tab>("notes");
  // Bumped on every show so the active tab can re-focus its input.
  const [shownAt, setShownAt] = useState(0);

  useShown((s) => {
    if (s.tab && (TABS as readonly string[]).includes(s.tab)) setTab(s.tab as Tab);
    setShownAt(s.shownAtMs);
  });

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
      <h2>Hotkeys</h2>
      <dl>
        <dt>Quick note</dt>
        <dd><kbd>{info.hotkeys.notes}</kbd></dd>
        <dt>Clipboard</dt>
        <dd><kbd>{info.hotkeys.clipboard}</kbd></dd>
        <dt>Ports</dt>
        <dd><kbd>{info.hotkeys.ports}</kbd></dd>
        <dt>Mechanism</dt>
        <dd>{strategyLabel[info.hotkeyStrategy]}</dd>
      </dl>

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

const strategyLabel: Record<AppInfo["hotkeyStrategy"], string> = {
  Plugin: "OS global shortcut",
  GnomeKeybinding: "GNOME custom shortcuts (Settings → Keyboard)",
  Manual: "Not available — bind `quickdesk toggle <notes|clipboard|ports>` manually",
};
