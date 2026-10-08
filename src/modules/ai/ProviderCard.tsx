import { t } from "../../shared/i18n";
import { ProviderUsage, RingChoice } from "../../shared/ipc";
import { absoluteTime } from "../../shared/time";
import { Gauge } from "./Gauge";
import { duration, errorText, sameChoice, when, windowLabel } from "./usage";

/** One AI tool: gauges for every limit with a percentage, then the rest. */
export function ProviderCard({ p, ringShows }: { p: ProviderUsage; ringShows: RingChoice | null }) {
  const gauges = p.windows.filter((w) => w.usedPercent !== null);
  const others = p.windows.filter((w) => w.usedPercent === null);
  const err = p.error ? errorText(p.error, p.asOf) : null;
  return (
    <section className="ai-card">
      <header>
        <h3>{p.name}</h3>
        {p.plan && <span className="badge">{p.plan}</span>}
        <span className="spacer" />
        {p.source === "live" ? (
          <span className="badge live" title={t("ai.live")}>
            <span className="live-dot" /> {t("ai.live")}
          </span>
        ) : (
          p.asOf && (
            <span className="badge" title={t("ai.local", { time: absoluteTime(p.asOf) })}>
              {when(p.asOf)}
            </span>
          )
        )}
      </header>
      {err && <div className={err.known ? "muted small" : "banner error"}>{err.text}</div>}
      {gauges.length > 0 && (
        <div className="gauges">
          {gauges.map((w) => (
            <div key={w.id} className={`gauge-cell ${sameChoice(ringShows, p.provider, w.id) ? "on-bar" : ""}`} title={sameChoice(ringShows, p.provider, w.id) ? t("ai.onTopBar") : undefined}>
              <Gauge percent={w.usedPercent} size={80} />
              <div className="gauge-label">{windowLabel(w)}</div>
              {w.resetsAt && <div className="gauge-sub">{t("ai.resetsIn", { left: duration(w.resetsAt - Date.now()) })}</div>}
              {w.resetsAt && <div className="gauge-sub muted">{when(w.resetsAt)}</div>}
              {w.detail && <div className="gauge-sub muted">{w.detail}</div>}
            </div>
          ))}
        </div>
      )}
      {others.map((w) => (
        <div key={w.id} className="ai-line">
          <span>{windowLabel(w)}</span>
          <span className="muted">{w.id === "quota_hit" ? (p.asOf ? when(p.asOf) : "–") : w.id === "extra_off" ? t("ai.off") : (w.detail ?? "–")}</span>
        </div>
      ))}
    </section>
  );
}
