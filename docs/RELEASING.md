# Releasing QuickDesk

Linux packages (`.deb`, `.rpm`, `.AppImage`) are built by GitHub Actions and
published to a Cloudflare R2 bucket, which also serves the download page.

## One-time setup

### 1. Cloudflare R2 bucket

1. Cloudflare dashboard → **R2** → **Create bucket**, e.g. `quickdesk-downloads`.
2. Bucket → **Settings** → **Public access**:
   - quick start: enable the **R2.dev subdomain** (gives `https://pub-<id>.r2.dev`), or
   - better: **Connect a custom domain** (e.g. `download.example.com`).
3. **R2** → **Manage R2 API Tokens** → **Create API token**:
   - permission **Object Read & Write**, limited to this one bucket;
   - note the **Access Key ID** and **Secret Access Key** (shown once);
   - the **Account ID** is shown on the R2 overview page.

### Custom domain (quickdesk.click)

The bucket is served at `https://quickdesk.click` (R2 → bucket → Settings →
Custom Domains). Releases use it for every link (`R2_PUBLIC_URL` in the Release
workflow), and the app's updater asks it first. **Keep the `r2.dev` public URL
enabled**: QuickDesk 0.2.2 only knows `https://pub-…r2.dev/update.json`, and
newer builds list it as a fallback in `plugins.updater.endpoints`.

### 2. GitHub secrets

Repository → **Settings** → **Secrets and variables** → **Actions** → **New repository secret**:

| Secret | Value |
|---|---|
| `R2_ACCOUNT_ID` | Cloudflare account ID |
| `R2_ACCESS_KEY_ID` | API token access key ID |
| `R2_SECRET_ACCESS_KEY` | API token secret |
| `R2_BUCKET` | bucket name, e.g. `quickdesk-downloads` |
| `TAURI_SIGNING_PRIVATE_KEY` | updater signing key (see step 3) |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | its password |

### 3. Updater signing key (once)

The app updates itself (Settings → Updates, and a banner when a new version
is out). Every package is signed, and the app installs only packages signed
by this key, so a compromised bucket cannot push malware.

```sh
npx tauri signer generate -w ~/.tauri/quickdesk.key   # asks for a password
```

- Put the contents of `~/.tauri/quickdesk.key` in the GitHub secret
  `TAURI_SIGNING_PRIVATE_KEY`, and its password in
  `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
- Put the contents of `~/.tauri/quickdesk.key.pub` in
  `plugins.updater.pubkey` in `src-tauri/tauri.conf.json`.
- **Back the private key up.** Without it, installed copies can never be
  updated again (they only trust this key).

The Release workflow builds with `createUpdaterArtifacts`, so each package gets
a `.sig`, and `scripts/publish_r2.py` writes `update.json` with one entry per
package type (`linux-x86_64-deb`, `-rpm`, `-appimage`): each install updates
with the same kind of package it came from. A .deb or .rpm is installed with
`pkexec dpkg -i` / `rpm -U`, which asks for the administrator password; the
app then restarts. The endpoint is `plugins.updater.endpoints`; keep the old
URL listed there if the bucket ever moves to a custom domain.

### 4. Download page

The page lives in `packaging/download/`: `index.html` plus `assets/` (CSS, JS,
images), which are uploaded next to it. Its demo is **the real UI**:
`npm run build:demo` builds the frontend with Tauri's API swapped for sample
data (`src/demo/`, see `vite.config.ts`) into `dist-demo/`, published as
`demo/` and shown in iframes. So any UI change ships to the page on the next
release, with nothing to copy by hand; when you add a command, give it a demo
answer in `src/demo/core.ts`. When publishing, these placeholders in
`index.html` are filled in and the result is uploaded as `index.html`:

`{{VERSION}}` `{{DATE}}` · `{{DEB_URL}}` `{{RPM_URL}}` `{{APPIMAGE_URL}}` (relative
links) · `{{DEB_SHA256}}` `{{RPM_SHA256}}` `{{APPIMAGE_SHA256}}` ·
`{{DEB_SIZE}}` `{{RPM_SIZE}}` `{{APPIMAGE_SIZE}}`.

A page can instead read `latest.json` at runtime (same data as JSON).

To check the page in a browser, render it without uploading anything (download
links and sizes are real only after a local `npx tauri build`):

```sh
python3 scripts/publish_r2.py --preview /tmp/qd-page
python3 -m http.server -d /tmp/qd-page 8000   # then open http://localhost:8000
```

## Each release

1. Write what changed for users in `CHANGELOG.md` **and** `CHANGELOG.vi.md`: a
   `## <version> — <date>` section with `### New` / `### Improved` / `### Fixed`
   (Vietnamese: `### Mới` / `### Cải thiện` / `### Sửa lỗi`). The release fails
   without it. These notes appear in the app's update banner, in "what's new"
   after updating, on the download page and on GitHub Releases.
2. Run the release command; it bumps the version in `tauri.conf.json`,
   `package.json` and `Cargo.toml` (and their lock files), checks both
   changelogs have the section, commits "Release <version>" and tags it:
   ```sh
   npm run release 0.2.3            # then: git push origin main v0.2.3
   npm run release 0.2.3 -- --push  # or let it push (starts the Release workflow)
   ```
   It refuses to run with other uncommitted changes, a version that is not
   newer, an existing tag, or missing release notes (`-- --dry-run` checks only).
3. Wait for the **Release** workflow (about 15 minutes).
4. The workflow checks the tag matches the version and that release notes exist, builds on Ubuntu
   22.04 (packages then also install on newer distributions), and uploads:
   ```
   releases/<version>/QuickDesk_<version>_amd64.deb
   releases/<version>/QuickDesk-<version>-1.x86_64.rpm
   releases/<version>/QuickDesk_<version>_amd64.AppImage
   releases/<version>/SHA256SUMS
   releases/<version>/*.sig      updater signatures
   latest.json
   update.json                   read by the app's updater
   index.html
   assets/...
   demo/...
   ```
   The packages are also attached to the workflow run as an artifact.

Testing the upload without R2: run any S3-compatible server locally and set
`R2_ENDPOINT` (e.g. `http://127.0.0.1:9000`) before `python3 scripts/publish_r2.py`.

## Not yet

- Windows / macOS builds (untested; no auto-paste, clipboard images/files on those platforms;
  unsigned builds trigger SmartScreen / Gatekeeper warnings).
- Auto-update inside the app.
