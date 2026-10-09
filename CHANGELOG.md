# Changelog

What changed in each QuickDesk release. The Vietnamese version is
[CHANGELOG.vi.md](CHANGELOG.vi.md); keep both in step.

Each release is a `## <version> — <date>` section with up to three groups:
`### New`, `### Improved`, `### Fixed`. The release workflow refuses to publish
a version that has no section here, and shows the section in the app, on the
download page and on GitHub Releases.

## 0.2.2 — 2026-10-09

### New
- QuickDesk now updates itself. It checks for new versions in the background, tells you what changed and installs only when you choose to. Every update is verified against QuickDesk's signing key before it is installed.
- Settings → Updates: see your version, check now, or turn automatic checks off.
- After an update, QuickDesk shows what is new, once.
- The download page is available in Vietnamese and links to the source code on GitHub.

### Improved
- One icon everywhere: the dock, the app menu, the windows and the tray now share QuickDesk's icon.

## 0.2.1 — 2026-10-08

### New
- A new look, shared by the app and its download page: a colour for each section (notes, clipboard, ports, AI), keyboard hints in popups and a macOS-style tab bar.
- Line icons in place of emoji throughout the app.
- Tray menus in a cleaner macOS style: short labels, a check mark for "Save Clipboard History", and AI limits grouped per tool.

### Improved
- The download page demo is the real app running on sample data.

## 0.2.0 — 2026-10-08

### New
- Packages for Ubuntu/Debian (.deb), Fedora (.rpm) and other distributions (AppImage), published on the QuickDesk download site.

### Improved
- Your data moves automatically to QuickDesk's new data folder (`~/.local/share/io.github.tankgum.quickdesk`).
