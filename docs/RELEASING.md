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

### 2. GitHub secrets

Repository → **Settings** → **Secrets and variables** → **Actions** → **New repository secret**:

| Secret | Value |
|---|---|
| `R2_ACCOUNT_ID` | Cloudflare account ID |
| `R2_ACCESS_KEY_ID` | API token access key ID |
| `R2_SECRET_ACCESS_KEY` | API token secret |
| `R2_BUCKET` | bucket name, e.g. `quickdesk-downloads` |

### 3. Download page (optional)

Put the page at `packaging/download/index.html`. When publishing, these
placeholders are filled in and the result is uploaded as `index.html`:

`{{VERSION}}` `{{DATE}}` · `{{DEB_URL}}` `{{RPM_URL}}` `{{APPIMAGE_URL}}` (relative
links) · `{{DEB_SHA256}}` `{{RPM_SHA256}}` `{{APPIMAGE_SHA256}}` ·
`{{DEB_SIZE}}` `{{RPM_SIZE}}` `{{APPIMAGE_SIZE}}`.

A page can instead read `latest.json` at runtime (same data as JSON).

## Each release

1. Bump `version` in `src-tauri/tauri.conf.json`, `Cargo.toml` (workspace) and `package.json`.
2. Commit, then tag and push:
   ```sh
   git tag v0.2.0
   git push origin main v0.2.0
   ```
3. The **Release** workflow checks the tag matches the version, builds on Ubuntu
   22.04 (packages then also install on newer distributions), and uploads:
   ```
   releases/<version>/QuickDesk_<version>_amd64.deb
   releases/<version>/QuickDesk-<version>-1.x86_64.rpm
   releases/<version>/QuickDesk_<version>_amd64.AppImage
   releases/<version>/SHA256SUMS
   latest.json
   index.html
   ```
   The packages are also attached to the workflow run as an artifact.

Testing the upload without R2: run any S3-compatible server locally and set
`R2_ENDPOINT` (e.g. `http://127.0.0.1:9000`) before `python3 scripts/publish_r2.py`.

## Not yet

- Windows / macOS builds (untested; no auto-paste, clipboard images/files on those platforms;
  unsigned builds trigger SmartScreen / Gatekeeper warnings).
- Auto-update inside the app.
