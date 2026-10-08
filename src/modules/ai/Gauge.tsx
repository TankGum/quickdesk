import { level } from "./usage";

/** Circular gauge: an arc filling clockwise from the top, % in the middle. */
export function Gauge({ percent, size = 76, stroke = 8 }: { percent: number | null; size?: number; stroke?: number }) {
  const r = (size - stroke) / 2;
  const c = 2 * Math.PI * r;
  const p = percent === null ? 0 : Math.min(100, Math.max(0, percent));
  return (
    <svg className={`gauge ${percent === null ? "none" : level(p)}`} width={size} height={size} viewBox={`0 0 ${size} ${size}`} role="img" aria-label={percent === null ? "-" : `${Math.round(p)}%`}>
      <circle className="gauge-track" cx={size / 2} cy={size / 2} r={r} strokeWidth={stroke} fill="none" />
      <circle
        className="gauge-fill"
        cx={size / 2}
        cy={size / 2}
        r={r}
        strokeWidth={stroke}
        fill="none"
        strokeLinecap="round"
        strokeDasharray={`${(p / 100) * c} ${c}`}
        transform={`rotate(-90 ${size / 2} ${size / 2})`}
      />
      <text className="gauge-text" x="50%" y="50%" dominantBaseline="central" textAnchor="middle" fontSize={size * 0.24}>
        {percent === null ? "–" : `${Math.round(p)}%`}
      </text>
    </svg>
  );
}
