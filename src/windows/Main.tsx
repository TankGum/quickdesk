import { useCallback, useEffect, useRef, useState } from "react";

import { ClipboardTab } from "../modules/clipboard/ClipboardTab";
import { NotesTab } from "../modules/notes/NotesTab";
import { PortsTab } from "../modules/ports/PortsTab";
import { SyncFooter } from "../modules/sync/SyncFooter";
import { SyncSettings } from "../modules/sync/SyncSettings";
import { HotkeySettings } from "../modules/settings/HotkeySettings";
import { PasteSettings } from "../modules/settings/PasteSettings";
import { LangPref, setLanguage, useI18n } from "../shared/i18n";
import { api, AppInfo, currentWindow, FocusStats, hideWindow, useShown } from "../shared/ipc";
import { Welcome } from "./Welcome";

const TABS = ["notes", "clipboard", "ports", "settings"] as const;
type Tab = (typeof TABS)[number];

export function Main() {
  const { t } = useI18n();
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
        {TABS.map((name) => (
          <button key={name} className={name === tab ? "active" : ""} onClick={() => setTab(name)}>
            {t(`tab.${name}` as const)}
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

function LanguagePicker() {
  const { t, pref } = useI18n();
  const options: { value: LangPref; label: string }[] = [
    { value: "auto", label: t("settings.language.auto") },
    { value: "vi", label: "Tiếng Việt" },
    { value: "en", label: "English" },
  ];
  return (
    <div className="radio-list horizontal">
      {options.map((o) => (
        <label key={o.value} className="radio">
          <input type="radio" name="ui-language" checked={pref === o.value} onChange={() => void setLanguage(o.value)} />
          <span>{o.label}</span>
        </label>
      ))}
    </div>
  );
}

function Settings() {
  const { t } = useI18n();
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
      <h2>{t("settings.language")}</h2>
      <LanguagePicker />

      <h2>{t("settings.sync")}</h2>
      <SyncSettings />

      <h2>{t("settings.hotkeys")}</h2>
      <HotkeySettings />

      <h2>{t("settings.autoPaste")}</h2>
      <PasteSettings />

      <h2>{t("settings.focusTest")}</h2>
      {stats && stats.total > 0 ? (
        <>
          <p>
            {t("settings.focusTest.summary", {
              focused: stats.focused,
              total: stats.total,
              p50: stats.latencyP50 ?? "-",
              max: stats.latencyMax ?? "-",
            })}
          </p>
          <table>
            <thead>
              <tr>
                <th>{t("settings.focusTest.window")}</th>
                <th>{t("settings.focusTest.document")}</th>
                <th>{t("settings.focusTest.input")}</th>
                <th>{t("settings.focusTest.latency")}</th>
              </tr>
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
        <p className="muted">{t("settings.focusTest.empty")}</p>
      )}

      <h2>{t("settings.about")}</h2>
      <dl>
        <dt>{t("settings.version")}</dt>
        <dd>{info.version}</dd>
        <dt>{t("settings.session")}</dt>
        <dd>
          {info.os} · {info.wayland ? "Wayland" : "X11/native"} · {info.desktop || t("settings.unknown")}
        </dd>
        <dt>{t("settings.device")}</dt>
        <dd><code>{info.deviceId}</code></dd>
        <dt>{t("settings.data")}</dt>
        <dd><code>{info.dataDir}</code></dd>
      </dl>
      <button className="danger" onClick={() => void api.quit()}>
        {t("settings.quit")}
      </button>
    </div>
  );
}

