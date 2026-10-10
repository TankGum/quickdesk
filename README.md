# QuickDesk

Developer toolbox in the tray: **Quick Notes** (E2E-encrypted sync through QuickDesk Cloud),
**Clipboard history** (local only), **Port Manager** (incl. Docker containers), **language
Runtimes** (versions through nvm, rustup, uv, mise) and **AI usage limits** (a ring in the top bar). Website: [quickdesk.click](https://quickdesk.click).
Tauri 2 + Rust + React. Design: [`docs/SPEC.md`](docs/SPEC.md).

| Hotkey | Opens |
|---|---|
| `Super+Alt+N` | Notes manager (cursor in "New note"; Ctrl+F searches) |
| `Super+Alt+V` | Clipboard popup (↑↓, Enter pastes into the previous app, Ctrl+Enter copies only) |
| `Super+Alt+P` | Port Manager |
| *(unset)* | Quick-note popup; also in the tray menu |

Auto-paste types Shift+Insert into the app you were using (works in browsers, editors and
terminals). Two backends on Linux:
- **Instant** (virtual keyboard via `/dev/uinput`): no prompts, no indicator. Opt-in on the
  welcome screen or in Settings → Auto-paste; asks for the admin password once to install a udev
  rule (the same permission `steam-devices` grants). Uninstalling the package removes it.
- **Desktop portal** (fallback): GNOME asks once and shows a remote-control indicator while pasting.

Turn auto-paste off in the Clipboard tab to just copy.

All hotkeys can be changed in Settings → Hotkeys. Super+Alt is the default because Vietnamese
input methods (IBus Unikey/Bamboo) swallow Super+Shift+letter while a text field is focused.

On GNOME Wayland the hotkeys are registered as GNOME custom shortcuts running
`quickdesk toggle <notes|quick-note|clipboard|ports>` and are removed again on quit.

## Install (Linux)

Download from [quickdesk.click](https://quickdesk.click/download) · [Docs](https://quickdesk.click/docs):

| Distribution | File | Install |
|---|---|---|
| Ubuntu / Debian | `.deb` | `sudo apt install ./QuickDesk_<version>_amd64.deb` |
| Fedora / openSUSE | `.rpm` | `sudo dnf install ./QuickDesk-<version>-1.x86_64.rpm` |
| Any other | `.AppImage` | `chmod +x QuickDesk_<version>_amd64.AppImage && ./QuickDesk_<version>_amd64.AppImage` |

Data lives in `~/.local/share/click.quickdesk/` (logs in `logs/` there). QuickDesk updates
itself (Settings → Updates).

## Working on QuickDesk

- **[`docs/WORKFLOW.md`](docs/WORKFLOW.md)**: how app changes, website changes and releases
  flow (CI, Cloudflare Pages, the download host, the in-app updater) and which file to edit.
- [`docs/RELEASING.md`](docs/RELEASING.md): one-time setup (R2 bucket, domains, secrets,
  signing key, Pages project).
- Ship a release: notes in `CHANGELOG.md` + `CHANGELOG.vi.md`, then `npm run release <version> -- --push`.

## Develop

Ubuntu/Debian packages:

```sh
sudo apt install build-essential libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev \
  librsvg2-dev libxdo-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev libdbus-1-dev
```

Then, with Rust (rustup) and Node ≥ 18 (the website in `website/` needs Node 22):

```sh
npm install
npx tauri dev          # run
npx tauri build        # .deb / .rpm / AppImage
npm run build:demo && cd website && npm install && npm run dev   # website, http://localhost:4321
```

## Test

```sh
cargo test --workspace         # unit + simulation tests
npm run typecheck
# Live tests (opt-in):
cargo test -p qd-platform --test x11_clipboard -- --ignored   # needs an X11/Xwayland display
QD_S3_ENDPOINT=... QD_S3_BUCKET=... QD_S3_REGION=... QD_S3_KEY=... QD_S3_SECRET=... \
  cargo test -p qd-sync --test s3_live -- --ignored            # any S3-compatible endpoint
```

## Layout

```
crates/qd-core       SQLite, migrations, settings, ids, hybrid logical clock
crates/qd-platform   session detection, GNOME keybindings, clipboard watchers
crates/qd-notes      notes storage, FTS, sync merge rules
crates/qd-clipboard  clipboard history storage and search
crates/qd-ports      listening ports, process/container lookup, kill
crates/qd-sync       encryption, QuickDesk Cloud transport, sync engine
crates/qd-ai-usage   AI usage limits (Claude Code, Codex, Antigravity)
crates/qd-runtimes   language versions through nvm, rustup, uv and mise; PATH fixes
sync-server          QuickDesk Cloud (Cloudflare Worker + R2)
src-tauri            app shell: windows, tray, hotkeys, updater, commands, background workers
src                  React UI (popups + main window)
src/demo             sample-data backend: the same UI runs in the website's demo
website              quickdesk.click (Astro, Cloudflare Pages), English + Vietnamese
scripts              release tools: release.py, changelog.py, publish_r2.py
docs                 WORKFLOW.md, RELEASING.md, SPEC.md
spikes/wayland       Wayland clipboard/hotkey spike and findings
```

## License

[MIT](LICENSE)
