/** Keyboard hints for a window's footer: each item is keys plus what they do. */
export type Hint = readonly [keys: readonly string[], label: string];

export function Hints({ items, className = "" }: { items: (Hint | false | null)[]; className?: string }) {
  return (
    <div className={`hints ${className}`}>
      {items.filter(Boolean).map((h) => {
        const [keys, label] = h as Hint;
        return (
          <span key={label} className="hint">
            {keys.map((k) => (
              <kbd key={k}>{k}</kbd>
            ))}
            {label}
          </span>
        );
      })}
    </div>
  );
}
