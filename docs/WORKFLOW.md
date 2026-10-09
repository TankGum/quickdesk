# Working on QuickDesk

How the pieces fit together, and what to do for each kind of change. One-time
infrastructure setup (bucket, secrets, signing key, Pages project) is in
[RELEASING.md](RELEASING.md).

## The moving parts

```
GitHub (TankGum/quickdesk)
 ├─ push to main ──────────► CI (.github/workflows/ci.yml): fmt, clippy, tests, typecheck,
 │                            app build, demo build, website build
 ├─ push to main ──────────► Cloudflare Pages builds website/ → https://quickdesk.click
 └─ push a tag vX.Y.Z ─────► Release (.github/workflows/release.yml):
                               1. checks tag = version and release notes exist
                               2. builds + signs .deb / .rpm / .AppImage
                               3. scripts/publish_r2.py → R2 bucket → https://dl.quickdesk.click
                                  (packages, .sig, SHA256SUMS, latest.json, update.json)
                               4. GitHub Release with the notes and packages
                               5. calls the Pages deploy hook → website shows the new version

Installed apps (0.2.2 and newer) read dl.quickdesk.click/update.json (fallback: r2.dev)
30 s after start and every 6 h, show "update available" with the notes, and install
only when the user clicks Update Now.
```

| Thing | Where | Notes |
|---|---|---|
| Website | `quickdesk.click`, Cloudflare Pages, source `website/` | Rebuilds on every push to `main` and after every release |
| Download host | `dl.quickdesk.click`, R2 bucket `quickdesk-downloads` | **Keep the `r2.dev` public URL on**: 0.2.2 only knows that address |
| Old links | `quickdesk.click/update.json`, `/latest.json`, `/releases/*` | Redirected to `dl.` by `website/public/_redirects` |
| Updater signing key | `~/.tauri/quickdesk.key` + GitHub secrets | Public half in `src-tauri/tauri.conf.json`. **Back it up**: losing it means installed apps can never update again |
| App identifier | `click.quickdesk` | Data in `~/.local/share/click.quickdesk`; older data dirs are migrated in `src-tauri/src/lib.rs` |

## 1. Changing the app

Code: `src/` (React UI), `src-tauri/` (shell, tray, updater, commands), `crates/`.

```sh
# work, then
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace && npm run typecheck
git commit … && git push origin main     # CI checks it; users get nothing yet
```

Pushing to `main` does **not** reach users. To ship, release (section 3).

While working, also:

- **New or changed Tauri command?** Give it an answer in `src/demo/core.ts`, or the
  website demo breaks (it runs the real UI on that sample data).
- **New UI text?** Add it to both languages in `src/shared/i18n.ts` (tray menu text:
  `src-tauri/src/i18n.rs`).
- **Changed behaviour** (defaults, limits, shortcuts, paths, anything sent over the
  network)? Update `website/src/data/docs.ts`, and **always** `website/src/data/privacy.ts`
  when data leaves the computer.

Note: the website demo is built from `main`, so it shows UI changes as soon as they are
pushed, before they are released.

## 2. Changing the website

Code: `website/` (Astro). Push to `main` and Cloudflare Pages deploys it in ~3 minutes;
no release needed. A branch or pull request gets its own preview URL.

| To change | Edit |
|---|---|
| Home page text (English and Vietnamese side by side) | `website/src/data/home.ts` |
| Home page markup | `website/src/components/Home.astro` |
| Download page | `website/src/components/DownloadPage.astro` |
| Docs | `website/src/data/docs.ts` |
| Privacy | `website/src/data/privacy.ts` |
| Changelog page | nothing: it reads `CHANGELOG.md` / `CHANGELOG.vi.md` |
| Nav, footer, page titles, languages | `website/src/layouts/Base.astro`, `website/src/i18n.ts`, `website/src/pages/` |
| Styles | home: `website/public/assets/css/style.css`; other pages: `website/src/styles/site.css` |
| Demo behaviour on the home page | `website/public/assets/js/main.js` (desktop, tray menus); sample data: `src/demo/core.ts` |

Every page exists in English (`/…`) and Vietnamese (`/vi/…`); `website/src/pages/vi/` holds
thin wrappers around the same components. Version, sizes and checksums come from
`dl.quickdesk.click/latest.json` at build time, so they are never edited by hand.

Run it locally (needs **Node 22**; the app itself works with Node 18):

```sh
npm run build:demo                      # repo root: the demo the home page embeds
cd website && npm install && npm run dev   # http://localhost:4321
```

## 3. Releasing

1. Write the notes for users in **both** `CHANGELOG.md` and `CHANGELOG.vi.md`:

   ```markdown
   ## 0.3.1 — 2026-10-20

   ### Fixed            (Vietnamese file: ### Sửa lỗi; also New/Mới, Improved/Cải thiện)
   - What the user notices, in plain words.
   ```

2. `npm run release 0.3.1 -- --push` bumps the version (tauri.conf.json, Cargo.toml,
   package.json and the lock files), commits "Release 0.3.1", tags `v0.3.1` and pushes.
   Without `-- --push` it stops before pushing; `-- --dry-run` only checks.
3. Watch the Release workflow in GitHub Actions (~15 minutes). If it fails before
   publishing, nothing was uploaded: fix and re-run.
4. Check `https://dl.quickdesk.click/update.json` shows the new version, and the website's
   `/download`.

The notes show up in the app (update banner and "what's new" after updating), on the
website, and on GitHub Releases. Versions only go up; the tag must match the version.

## Gotchas

- `!` in front of a command is for the Claude Code prompt only; in a normal terminal it
  negates the command (`! wget … && pkexec …` never runs the install).
- Super+Alt (not Super+Shift) is the default because IBus Unikey/Bamboo swallow
  Super+Shift+letter while typing.
- GNOME draws tray menus: they cannot be styled, and they close on every click.
- The Claude usage endpoint rate-limits: live checks are every 5 minutes at most, with
  back-off after a 429. QuickDesk only reads Claude Code's token; never store or refresh it.
- A `.deb`/`.rpm` update is installed with `pkexec`, so the user is asked for their password.
