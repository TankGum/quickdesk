import { t } from "../../shared/i18n";
import { ClipKind } from "../../shared/ipc";

const KINDS: (ClipKind | undefined)[] = [undefined, "text", "image", "files"];
const ICON: Record<string, string> = { all: "", text: "📝 ", image: "🖼️ ", files: "📄 " };

/** All / Text / Images / Files chips. */
export function KindFilter({ value, onChange, compact = false }: { value?: ClipKind; onChange: (k?: ClipKind) => void; compact?: boolean }) {
  return (
    <div className={`kind-filter ${compact ? "compact" : ""}`} role="tablist">
      {KINDS.map((k) => (
        <button
          key={k ?? "all"}
          role="tab"
          aria-selected={value === k}
          className={`chip ${value === k ? "on" : ""}`}
          // Keep focus in the search box (popup keyboard navigation).
          onMouseDown={(e) => e.preventDefault()}
          onClick={() => onChange(k)}
        >
          {ICON[k ?? "all"]}
          {t(`clip.kind.${k ?? "all"}` as const)}
        </button>
      ))}
    </div>
  );
}
