import { useCallback, useEffect, useState } from "react";

import { ProviderCard } from "../modules/ai/ProviderCard";
import { STALE_MS } from "../modules/ai/usage";
import { useI18n } from "../shared/i18n";
import { api, errorMessage, hideWindow, UsageSnapshot, useBackendEvent, useEscapeToHide, useShown } from "../shared/ipc";
import { relativeTime } from "../shared/time";

/** Opened from the top-bar ring: the usage of every AI tool at a glance. */
export function UsagePopup() {
  const { t } = useI18n();
  const [snap, setSnap] = useState<UsageSnapshot | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [, tick] = useState(0);

  const refresh = useCallback(async () => {
    setBusy(true);
    try {
      setSnap(await api.aiUsageRefresh());
      setError(null);
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }, []);

  const load = useCallback(() => {
    void api.aiUsage().then((s) => {
      setSnap(s);
      if (!s.updatedAt || Date.now() - s.updatedAt > STALE_MS) void refresh();
    });
  }, [refresh]);

  useEffect(load, [load]);
  useShown(load);
  useEscapeToHide();
  useBackendEvent("ai://usage", useCallback(() => void api.aiUsage().then(setSnap), []));
  useEffect(() => {
    const id = setInterval(() => tick((n) => n + 1), 30_000);
    return () => clearInterval(id);
  }, []);

  return (
    <div className="popup usage-popup">
      <header className="usage-head">
        <h2>{t("ai.title")}</h2>
        <span className="muted small">{snap?.updatedAt ? relativeTime(snap.updatedAt) : ""}</span>
        <span className="spacer" />
        <button className={`icon ${busy ? "spin" : ""}`} title={t("ai.refresh")} disabled={busy} onClick={() => void refresh()}>
          ↻
        </button>
        <button className="icon" title={t("notes.close")} onClick={() => void hideWindow()}>
          ✕
        </button>
      </header>
      {error && <div className="banner error">{error}</div>}
      <div className="usage-body">
        {snap && snap.providers.length === 0 && <div className="empty">{t("ai.none")}</div>}
        {snap?.providers.map((p) => (
          <ProviderCard key={p.provider} p={p} ringShows={snap.trayEnabled ? snap.ringShows : null} compact />
        ))}
      </div>
      <footer className="usage-foot">
        <button className="btn" onClick={() => void api.show("ai")}>
          {t("ai.openFull")}
        </button>
      </footer>
    </div>
  );
}
