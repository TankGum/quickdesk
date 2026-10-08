# QuickDesk

Developer toolbox in the tray: **Quick Notes** (E2E-encrypted sync through your own S3/R2 bucket),
**Clipboard history** (local only) and **Port Manager** (incl. Docker containers).
Tauri 2 + Rust + React. Design: [`docs/SPEC.md`](docs/SPEC.md).

| Hotkey | Opens |
|---|---|
| `Super+Alt+N` | Notes manager (cursor in "New note"; Ctrl+F searches) |
| `Super+Alt+V` | Clipboard popup (↑↓, Enter copies) |
| `Super+Alt+P` | Port Manager |
| *(unset)* | Quick-note popup; also in the tray menu |

All hotkeys can be changed in Settings → Hotkeys. Super+Alt is the default because Vietnamese
input methods (IBus Unikey/Bamboo) swallow Super+Shift+letter while a text field is focused.

On GNOME Wayland the hotkeys are registered as GNOME custom shortcuts running
`quickdesk toggle <notes|quick-note|clipboard|ports>` and are removed again on quit.

## Develop

Ubuntu/Debian packages:

```sh
sudo apt install build-essential libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev \
  librsvg2-dev libxdo-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev libdbus-1-dev
```

Then, with Rust (rustup) and Node ≥ 18:

```sh
npm install
npx tauri dev          # run
npx tauri build        # .deb / AppImage
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
crates/qd-sync       encryption, S3 transport, sync engine
src-tauri            app shell: windows, tray, hotkeys, commands, background workers
src                  React UI (popups + main window)
spikes/wayland       Wayland clipboard/hotkey spike and findings
```
