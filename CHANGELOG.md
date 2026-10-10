# Changelog

What changed in each QuickDesk release. The Vietnamese version is
[CHANGELOG.vi.md](CHANGELOG.vi.md); keep both in step.

Each release is a `## <version> — <date>` section with up to three groups:
`### New`, `### Improved`, `### Fixed`. The release workflow refuses to publish
a version that has no section here, and shows the section in the app, on the
download page and on GitHub Releases.

## 0.4.0 — 2026-10-10

### New
- A Runtimes tab for the versions of your programming languages: see which Node.js, Python, Rust, Go… a new terminal really runs and where it comes from, install and uninstall versions, pick the default and pin a version for a project. QuickDesk works through nvm, uv, rustup and mise, and never touches system packages.
- When a newer release of an installed line is out (say Node 22.24.1 for 22.23.3), the version shows an update button; the old one stays until you remove it.
- Languages without a version manager (Go from apt, for example) can get mise in one click, downloaded from its GitHub releases and checked against their SHA-256 sums.
- “Apply right away in open terminals”: changes reach terminals that are already open at their next prompt, and Node follows the nearest `.nvmrc` when you `cd`. When something in PATH gets in the way, the tab says why and offers the exact lines to add to your shell file, with a backup.

### Improved
- The download page shows the full command to download and install from a terminal.
- The version list can show the newest release of each line, LTS releases only, or everything.

## 0.3.1 — 2026-10-09

### New
- Sync is now built in: turn it on in Settings → Sync and join your other computers with a sync code. No account, email or storage setup. Notes are still encrypted on your computer before they leave it; QuickDesk Cloud only stores ciphertext.

### Fixed
- Notes, clipboard history and settings from earlier versions are back. Updating to 0.2.0 and to 0.3.0 started with an empty folder instead of moving your data; QuickDesk now brings it over by itself, and keeps the old folders as a backup.
- Opening Settings no longer turns the coloured dots of the other tabs grey.

### Improved
- Syncing through your own S3 or R2 bucket is no longer offered. If you used it, sync is turned off once with a note in Settings → Sync; your notes stay on your computer.
- The AI usage menu in the top bar shows only the tool you track, with a green, yellow or red bar for each limit and the reset time under it. Pick another tool under Track.

## 0.3.0 — 2026-10-09

### New
- QuickDesk has its own address: [quickdesk.click](https://quickdesk.click). Downloads and updates come from there.
- A real website: download page that suggests the right package, documentation, changelog and privacy pages, in English and Vietnamese.

### Improved
- Your data now lives in `~/.local/share/click.quickdesk`. QuickDesk moves it there by itself the first time it starts; notes, clipboard history and settings stay as they are.
- Restarting after an update is more reliable.

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
