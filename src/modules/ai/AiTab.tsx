import { useCallback, useEffect, useState } from "react";

import { t, useI18n } from "../../shared/i18n";
import { api, errorMessage, UsageSnapshot, useBackendEvent } from "../../shared/ipc";
import { ProviderCard } from "./ProviderCard";
import { STALE_MS, windowLabel } from "./usage";

export function AiTab({ focusSignal }: { focusSignal: number }) {
  useI18n();
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

  // Show what we have immediately, refresh if it is getting old.
  useEffect(() => {
    void api.aiUsage().then((s) => {
      setSnap(s);
      if (!s.updatedAt || Date.now() - s.updatedAt > STALE_MS) void refresh();
    });
  }, [focusSignal, refresh]);

  useBackendEvent(
    "ai://usage",
    useCallback(() => void api.aiUsage().then(setSnap), []),
  );

  // Keep "in 2 h 5 min" current.
  useEffect(() => {
    const id = setInterval(() => tick((n) => n + 1), 30_000);
    return () => clearInterval(id);
  }, []);

  if (!snap) return null;

  return (
    <div className="ai">
      <div className="toolbar">
        <span className="muted ai-intro">{t("ai.intro")}</span>
        <button className="btn" disabled={busy} onClick={() => void refresh()}>
          {busy ? t("ai.refreshing") : t("ai.refresh")}
        </button>
      </div>
      <div className="ai-tray">
        <label className="switch">
          <input
            type="checkbox"
            checked={snap.trayEnabled}
            onChange={(e) => void api.aiSetTray(e.target.checked).catch((x) => setError(errorMessage(x)))}
          />
          {t("ai.tray")}
        </label>
        {snap.trayEnabled && (
          <label className="ai-ring-pick">
            <span className="muted">{t("ai.ring")}:</span>
            <select
              value={snap.ring ? `${snap.ring.provider}|${snap.ring.window}` : ""}
              onChange={(e) => {
                const [provider, window] = e.target.value.split("|");
                void api.aiSetRing(e.target.value ? { provider, window } : null).catch((x) => setError(errorMessage(x)));
              }}
            >
              <option value="">{t("ai.ringAuto")}</option>
              {snap.providers.flatMap((p) =>
                p.windows
                  .filter((w) => w.usedPercent !== null)
                  .map((w) => (
                    <option key={`${p.provider}|${w.id}`} value={`${p.provider}|${w.id}`}>
                      {p.name} · {windowLabel(w)} ({Math.round(w.usedPercent!)}%)
                    </option>
                  )),
              )}
            </select>
          </label>
        )}
      </div>
      {error && <div className="banner error">{error}</div>}
      {snap.providers.length === 0 ? (
        <div className="empty">{t("ai.none")}</div>
      ) : (
        <div className="ai-cards">
          {snap.providers.map((p) => (
            <ProviderCard key={p.provider} p={p} ringShows={snap.trayEnabled ? snap.ringShows : null} />
          ))}
        </div>
      )}
    </div>
  );
}
