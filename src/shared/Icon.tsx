/**
 * Line icons in the spirit of Apple's SF Symbols: 24-unit grid, 1.75 stroke,
 * round caps, drawn in currentColor. Used instead of emoji, which render
 * differently on every system and clash with the UI.
 */
const PATHS = {
  star: <path d="M12 3.8l2.5 5.1 5.6.8-4 4 1 5.6-5.1-2.7-5 2.7 1-5.6-4.1-4 5.6-.8z" />,
  starFill: <path d="M12 3.8l2.5 5.1 5.6.8-4 4 1 5.6-5.1-2.7-5 2.7 1-5.6-4.1-4 5.6-.8z" fill="currentColor" />,
  trash: (
    <>
      <path d="M4.5 6.5h15M9.5 6.5V5c0-.8.6-1.5 1.4-1.5h2.2c.8 0 1.4.7 1.4 1.5v1.5" />
      <path d="M6.5 6.5l.8 12.1c.1 1 .9 1.9 2 1.9h5.4c1.1 0 1.9-.9 2-1.9l.8-12.1M10 10.5v6M14 10.5v6" />
    </>
  ),
  xmark: <path d="M6.5 6.5l11 11M17.5 6.5l-11 11" />,
  plus: <path d="M12 5.5v13M5.5 12h13" />,
  text: <path d="M4.5 6h15M4.5 10h10M4.5 14h15M4.5 18h10" />,
  photo: (
    <>
      <rect x="3.5" y="5" width="17" height="14" rx="2.5" />
      <circle cx="9" cy="10" r="1.6" />
      <path d="M4 17.5l4.5-4.5 3.5 3.5 2.5-2.5 5.5 5" />
    </>
  ),
  doc: <path d="M7 3.5h6.5L18 8v11c0 .8-.7 1.5-1.5 1.5h-9c-.8 0-1.5-.7-1.5-1.5V5c0-.8.7-1.5 1.5-1.5zM13.5 3.5V8H18" />,
  folder: <path d="M3.5 7.5c0-1.1.9-2 2-2h3.8l2 2h7.2c1.1 0 2 .9 2 2v8c0 1.1-.9 2-2 2h-13c-1.1 0-2-.9-2-2z" />,
  archive: (
    <>
      <rect x="3.5" y="4.5" width="17" height="4" rx="1" />
      <path d="M5 8.5v9c0 1.1.9 2 2 2h10c1.1 0 2-.9 2-2v-9M10 12.5h4" />
    </>
  ),
  film: (
    <>
      <rect x="3.5" y="5" width="17" height="14" rx="2.5" />
      <path d="M7.5 5v14M16.5 5v14M3.5 9.5h4M3.5 14.5h4M16.5 9.5h4M16.5 14.5h4" />
    </>
  ),
  link: <path d="M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1-1" />,
  terminal: (
    <>
      <rect x="3" y="4.5" width="18" height="15" rx="2.5" />
      <path d="M7 9.5l3 2.5-3 2.5M12.5 15H17" />
    </>
  ),
  refresh: <path d="M19 12a7 7 0 1 1-2.05-4.95M19.5 4.5v4h-4" />,
  box: <path d="M12 3.5l7.5 4v9l-7.5 4-7.5-4v-9zM4.5 7.5l7.5 4 7.5-4M12 11.5v9" />,
  pause: <path d="M9 6.5v11M15 6.5v11" />,
  chevronDown: <path d="M7 10l5 5 5-5" />,
  checklist: <path d="M4 7.5l1.5 1.5L8.5 6M4 15.5L5.5 17l3-3M11.5 7.5H20M11.5 15.5H20" />,
  quote: <path d="M10 7.5c-3 1-4.5 3-4.5 6V16h4v-4H6.7M19 7.5c-3 1-4.5 3-4.5 6V16h4v-4h-2.8" />,
  listBullet: (
    <>
      <path d="M9 7h11M9 12h11M9 17h11" />
      <circle cx="5" cy="7" r=".9" fill="currentColor" />
      <circle cx="5" cy="12" r=".9" fill="currentColor" />
      <circle cx="5" cy="17" r=".9" fill="currentColor" />
    </>
  ),
  listNumber: <path d="M10 7h10M10 12h10M10 17h10M4.5 5.5h1v4M4 9.5h2.5M4 14h2.2c.6 0 .9.7.5 1.1L4 18.5h2.8" />,
  code: <path d="M9 8l-4 4 4 4M15 8l4 4-4 4" />,
  codeBlock: (
    <path d="M9 5c-2 0-2.5 1-2.5 2.5v2c0 1-.6 2-1.8 2.5 1.2.5 1.8 1.5 1.8 2.5v2C6.5 18 7 19 9 19M15 5c2 0 2.5 1 2.5 2.5v2c0 1 .6 2 1.8 2.5-1.2.5-1.8 1.5-1.8 2.5v2c0 1.5-.5 2.5-2.5 2.5" />
  ),
  warning: <path d="M12 4.5l8.5 15h-17zM12 10v4M12 17v.01" />,
  check: <path d="M5 12.5l4.5 4.5L19 7.5" />,
} as const;

export type IconName = keyof typeof PATHS;

export function Icon({ name, size = 16, className = "" }: { name: IconName; size?: number; className?: string }) {
  return (
    <svg
      className={`icon-svg ${className}`}
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.75}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {PATHS[name]}
    </svg>
  );
}
