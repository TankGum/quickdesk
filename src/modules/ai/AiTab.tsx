import { useCallback, useEffect, useState } from "react";

import { Key, t, useI18n } from "../../shared/i18n";
import { api, errorMessage, ProviderUsage, UsageSnapshot, UsageWindow, useBackendEvent } from "../../shared/ipc";
import { absoluteTime, clockTime, relativeTime } from "../../shared/time";

const STALE_MS = 60_000;

export function level(percent: number): "ok" | "warn" | "high" {
  return percent >= 85 ? "high" : percent >= 60 ? "warn" : "ok";
}

function duration(ms: number): string {
  const min = Math.max(0, Math.ceil(ms / 60_000));
  const d = Math.floor(min / 1440);
  const h = Math.floor((min % 1440) / 60);
  const m = min % 60;
  if (d > 0) return t("ai.duration.dh", { d, h });
  if (h > 0) return t("ai.duration.hm", { h, m });
  return t("ai.duration.min", { m });
}

/** Today: "16:20"; otherwise date and time. */
function when(ms: number): string {
  return new Date(ms).toDateString() === new Date().toDateString() ? clockTime(ms) : absoluteTime(ms);
}

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
      <label className="switch ai-tray">
        <input
          type="checkbox"
          checked={snap.trayEnabled}
          onChange={(e) => void api.aiSetTray(e.target.checked).catch((x) => setError(errorMessage(x)))}
        />
        {t("ai.tray")}
      </label>
      {error && <div className="banner error">{error}</div>}
      {snap.providers.length === 0 ? (
        <div className="empty">{t("ai.none")}</div>
      ) : (
        <div className="ai-cards">
          {snap.providers.map((p) => (
            <ProviderCard key={p.provider} p={p} />
          ))}
        </div>
      )}
      <p className="muted small">
        {snap.updatedAt && <>{t("ai.updated", { time: relativeTime(snap.updatedAt) })} · </>}
        {t("ai.footnote")}
      </p>
    </div>
  );
}

function ProviderCard({ p }: { p: ProviderUsage }) {
  const errKey = p.error && (`ai.err.${p.error}` as Key);
  const knownErr = errKey && ["ai.err.login_expired", "ai.err.no_data", "ai.err.no_percent"].includes(errKey);
  return (
    <section className="ai-card">
      <header>
        <h3>{p.name}</h3>
        {p.plan && <span className="badge">{p.plan}</span>}
        <span className="spacer" />
        <span className={`badge ${p.source}`}>
          {p.source === "live" ? t("ai.live") : p.asOf ? t("ai.local", { time: absoluteTime(p.asOf) }) : t("ai.local", { time: "?" })}
        </span>
      </header>
      {p.error && <div className={knownErr ? "muted small" : "banner error"}>{knownErr ? t(errKey as Key) : p.error}</div>}
      {p.windows.map((w) => (
        <WindowRow key={w.id} w={w} asOf={p.asOf} />
      ))}
    </section>
  );
}

function WindowRow({ w, asOf }: { w: UsageWindow; asOf: number | null }) {
  const key = `ai.window.${w.id}` as Key;
  const label = t(key) === key ? w.label : t(key);
  if (w.id === "quota_hit") {
    return (
      <div className="ai-window">
        <div className="ai-window-head">
          <span>{label}</span>
          <span className="muted">{asOf ? when(asOf) : "-"}</span>
        </div>
      </div>
    );
  }
  if (w.id === "extra_off") {
    return (
      <div className="ai-window">
        <div className="ai-window-head">
          <span>{label}</span>
          <span className="muted">{t("ai.off")}</span>
        </div>
      </div>
    );
  }
  const pct = w.usedPercent;
  return (
    <div className="ai-window">
      <div className="ai-window-head">
        <span>{label}</span>
        <span className={pct !== null ? `ai-pct ${level(pct)}` : "muted"}>
          {pct !== null ? `${Math.round(pct)}%` : ""}
          {w.detail && <span className="muted"> {w.detail}</span>}
        </span>
      </div>
      {pct !== null && (
        <div className="ai-bar">
          <div className={`ai-fill ${level(pct)}`} style={{ width: `${Math.min(100, Math.max(0, pct))}%` }} />
        </div>
      )}
      {w.resetsAt && <div className="muted small">{t("ai.resets", { time: when(w.resetsAt), left: duration(w.resetsAt - Date.now()) })}</div>}
    </div>
  );
}
