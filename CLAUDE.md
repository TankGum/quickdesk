# Notes for AI assistants working on QuickDesk

Read [docs/WORKFLOW.md](docs/WORKFLOW.md) first: what to do for app changes, website
changes and releases, and where everything lives. Infrastructure setup is in
[docs/RELEASING.md](docs/RELEASING.md); the product design in [docs/SPEC.md](docs/SPEC.md).

Rules that are easy to break:

- The maintainer writes in Vietnamese; reply in Vietnamese (technical terms in English are fine).
- Do not push, tag or release without being asked. Pushing to `main` deploys the website;
  pushing a tag publishes a release to every user.
- Checks before committing: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace`, `npm run typecheck`; website changes: `npm run build:demo` then
  `npm run build --prefix website` (Node 22).
- Every new Tauri command needs a demo answer in `src/demo/core.ts`; every UI string needs
  English and Vietnamese (`src/shared/i18n.ts`, tray: `src-tauri/src/i18n.rs`).
- Behaviour that changes what leaves the computer must be reflected in
  `website/src/data/privacy.ts`.
- Claude Code's OAuth token (`~/.claude/.credentials.json`) may only be read and sent to
  `api.anthropic.com`; never print, store or refresh it.
- Sync goes through QuickDesk Cloud (`sync-server/`, `crates/qd-sync/src/cloud.rs`); notes stay
  end-to-end encrypted on the client. Deploying the Worker affects every installed app.
- QuickDesk ships for Linux and Windows. Platform code sits behind `cfg(target_os = "linux")` /
  `cfg(windows)`; the UI checks `isWindows` (`src/shared/platform.ts`). Releases build both, and
  publish only when both built.
- Never read `~/.tauri/quickdesk.key` (the updater's private signing key); the `.pub` file is fine.
- Release notes go in both `CHANGELOG.md` and `CHANGELOG.vi.md` (a `## Unreleased` section is fine until the
  version is known; the release script dates it); then `npm run release <version>`.
