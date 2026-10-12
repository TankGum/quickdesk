// The site's own words in English: header and footer, page titles, and the
// text of each component. Every other language has the same shape
// (`Strings`), so TypeScript reports anything missing. {v} (version), {date},
// {beta} … are filled in by the page. Long content (home page sections, docs,
// privacy, changelog) lives in src/data/ (English, Vietnamese) and in
// src/locales/<lang>/ for the other languages.
const en = {
  ui: {
    home: "Home",
    download: "Download",
    docs: "Docs",
    changelog: "Changelog",
    privacy: "Privacy",
    github: "GitHub",
    language: "Language",
    footer: "QuickDesk · MIT License.",
    tagline: "Notes, clipboard history, ports and AI limits. One shortcut away.",
    systems: "Systems",
    soon: "soon",
    product: "Product",
    project: "Project",
    issues: "Report a problem",
    sitemap: "Site map",
    skip: "Skip to content",
    menu: "Menu",
    close: "Close",
    faq: "FAQ",
    theme: "Switch between light and dark",
    themeShort: "Light / dark",
  },
  pages: {
    home: {
      title: "QuickDesk — Notes, clipboard & ports for Linux, Windows and macOS",
      description:
        "QuickDesk is a free, open-source desktop helper for Linux, Windows and macOS: end-to-end encrypted notes, clipboard history, a live port manager and your AI usage limits, summoned with one shortcut.",
    },
    download: {
      title: "Download QuickDesk for Linux, Windows and macOS (free)",
      description:
        "Download QuickDesk for Windows 10 and 11, macOS on Apple Silicon, Ubuntu, Debian, Fedora and other Linux distributions: the Windows installer, .dmg, .deb, .rpm and AppImage, with checksums.",
    },
    docs: {
      title: "QuickDesk docs: install, shortcuts, sync, clipboard, ports",
      description:
        "How to install QuickDesk on Linux, Windows and macOS, use its shortcuts, sync notes, clipboard history, ports, language versions, AI usage and updates.",
    },
    changelog: {
      title: "QuickDesk changelog: what’s new in each release",
      description: "What is new, improved and fixed in each QuickDesk release.",
    },
    privacy: {
      title: "Privacy · QuickDesk",
      description: "QuickDesk has no accounts, analytics or telemetry. What stays on your computer and what is sent where.",
    },
    sitemap: {
      title: "Site map · QuickDesk",
      description:
        "Every page of quickdesk.click: download for Linux, Windows and macOS, the docs section by section, every release in the changelog, and privacy.",
    },
  },
  seo: {
    app: "A free, open-source desktop helper: end-to-end encrypted synced notes, clipboard history, a port manager, programming language versions and AI usage limits, each one shortcut away.",
    ogAlt: "QuickDesk: your whole desk, one shortcut away.",
  },
  // The hero's download button, per system.
  hero: {
    pick: "Your system",
    linux: "Download for Linux",
    windows: "Download for Windows",
    macos: "Download for macOS{beta}",
    macSoon: "macOS: coming soon",
  },
  download: {
    title: "Download QuickDesk {v}",
    leadMac: "For Linux and Windows (x86_64) and macOS (Apple Silicon{beta}) · released {date}. Free and open source (MIT).",
    leadSoon: "For Linux and Windows on x86_64, macOS coming soon · released {date}. Free and open source (MIT).",
    pick: "Your system",
    devTitle: "The Mac version is still in development",
    devText:
      "QuickDesk for macOS works, but it is new: not everything has been tried on every Mac yet, and auto-paste and the shortcuts are still being refined. If something does not work, please tell us.",
    devLink: "Report a problem on GitHub",
    deb: ["Ubuntu, Debian, Linux Mint, Pop!_OS", "Ubuntu 22.04 and newer, Debian 12 and newer."],
    rpm: ["Fedora and other RPM-based distros", "Fedora 36 and newer, openSUSE Tumbleweed."],
    appimage: ["Any other distribution", "Portable: no install needed. Needs FUSE 2 (libfuse2)."],
    windows: ["Windows 10 and 11", "64-bit. Installs for your user only: no administrator rights needed."],
    macos: ["macOS 11 or later", "Apple Silicon (M1 and newer). Open the disk image and drag QuickDesk to Applications."],
    oneMac:
      "In Terminal. Downloads QuickDesk.app, checks its SHA-256, puts it in Applications and opens it. Installed this way, macOS does not warn about it.",
    afterMac:
      "QuickDesk lives in the menu bar and updates itself from now on. For auto-paste, allow it under Privacy & Security → Accessibility when macOS asks.",
    gatekeeper: "Downloaded in a browser, macOS blocks the first launch: QuickDesk is not notarized by Apple yet.",
    gatekeeperHow: "How to open it",
    openTitle: "Opening QuickDesk the first time",
    openLead:
      "QuickDesk is not notarized by Apple yet (that needs a paid Apple Developer ID). When the .dmg was downloaded in a browser, macOS stops the first launch with “Apple cannot check it for malicious software”. Nothing is wrong with the app: do one of these once. Installed with the command above, you can skip this.",
    openA: "In System Settings",
    openASteps: [
      "Drag QuickDesk to Applications and open it. When macOS warns, click Done.",
      "Open System Settings → Privacy & Security and scroll down to Security.",
      "Next to “QuickDesk was blocked to protect your Mac”, click Open Anyway, confirm with your password or Touch ID, then click Open.",
    ],
    openOld: "On macOS 14 or earlier, Control-click QuickDesk in Applications and choose Open, then Open again.",
    openB: "Or in Terminal",
    openBLead: "After dragging QuickDesk to Applications, run this, then open QuickDesk as usual:",
    get: "Download",
    sha: "SHA-256",
    all: "All checksums (SHA256SUMS)",
    notes: "What’s new in {v}",
    allChanges: "All changes",
    afterLinux: "Then open QuickDesk from your app menu. It lives in the top bar and updates itself from now on.",
    afterWindows: "Then open QuickDesk from the Start menu. It lives in the system tray and updates itself from now on.",
    copy: "Copy",
    oneTitle: "Install with one command",
    oneLinux:
      "In a terminal. Picks the right package for your distribution (.deb, .rpm or AppImage), checks its SHA-256 and installs it. It asks for your password only for the install step.",
    oneWindows: "In PowerShell. Downloads the installer, checks its SHA-256 and installs QuickDesk for your user, without administrator rights.",
    oneScript: "Read the script",
    orPick: "Or pick a package",
    smartscreen: "Windows may say it protected your PC: the installer is not signed yet. Choose More info → Run anyway.",
    macTitle: "QuickDesk for macOS is on the way",
    macLead: "Notes, clipboard history, ports and AI limits on the Mac. It will be announced on the changelog and in GitHub releases.",
    macWatch: "Watch the releases on GitHub",
  },
  // The home page's call to action, and the install its terminal plays.
  cta: {
    title: "Ready when you are.",
    lead: "Install it once and everything is a shortcut away: your notes, what you copied, the ports in use and your AI limits.",
    download: "Download",
    changes: "What’s new",
    meta: "Version {v} · {date} · Free and open source (MIT)",
    downloading: "Downloading",
    verified: "SHA-256 verified",
    installed: "QuickDesk {v} installed",
    running: "QuickDesk is running",
    open: "Open it with",
  },
  intro: {
    tagline: "Your whole desk. One shortcut away.",
    skip: "Skip",
  },
  docs: {
    title: "Documentation",
    lead: "Installing, shortcuts, sync, clipboard, ports, AI usage and updates.",
    contents: "Contents",
    groups: ["Get started", "Features", "Help"],
    sync: "Sync",
    clipboard: "Clipboard",
    versions: "Runtimes",
    stuck: "Still stuck?",
    ask: "Ask on GitHub",
  },
  sitemap: {
    title: "Site map",
    lead: "Every page and section of quickdesk.click.",
    home: "Home",
    demo: "Try the shortcuts",
    getIt: "Get QuickDesk",
    oneLine: "Install scripts",
    other: "Elsewhere",
    machine: "sitemap.xml (for search engines)",
    privacyNote: "No accounts, analytics or telemetry: what stays on your computer and what is sent where.",
  },
  changelog: {
    title: "Changelog",
    lead: "What is new, improved and fixed in each QuickDesk release.",
    latest: "Latest",
    source: "Source: CHANGELOG.md on GitHub",
    english: "Not translated yet; shown in English.",
  },
  notFound: {
    title: "Page not found · QuickDesk",
    description: "This page does not exist.",
    heading: "This page does not exist.",
  },
  // What public/assets/js/main.js writes itself (the demo's toasts and notes).
  demo: {
    refreshed: "Usage refreshed",
    paused: "Clipboard history paused",
    resumed: "Clipboard history resumed",
    quit: "Quit — in the real app, QuickDesk would close now",
    encrypting: "Encrypting…",
    synced: "Encrypted & synced",
    copied: "Copied",
    pressCopy: "Press Ctrl+C",
    copy: "Copy",
    note: "Standup — Tuesday\n• Shipped clipboard images\n• Reviewing the release flow\n• No blockers",
  },
};

export type Strings = typeof en;
export default en;
