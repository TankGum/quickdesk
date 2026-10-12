# Releasing QuickDesk

Notes sync through QuickDesk Cloud (`sync-server/`, a Worker on
`sync.quickdesk.click`). Linux packages (`.deb`, `.rpm`, `.AppImage`), the Windows installer
(NSIS `.exe`) and the macOS app (Apple Silicon `.dmg` and `.app.tar.gz`) are built by GitHub
Actions and published to a Cloudflare R2 bucket served at
`https://dl.quickdesk.click`.
The website `https://quickdesk.click` (`website/`) is a separate Astro site on
Cloudflare Pages that reads the newest release from the bucket's `latest.json`.

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

### Download host (dl.quickdesk.click)

The bucket is served at `https://dl.quickdesk.click` (R2 → bucket → Settings →
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
| `CF_PAGES_DEPLOY_HOOK` | the website's deploy hook URL (see step 4) |
| `MACOS_CERTIFICATE` | macOS code signing certificate, base64 (see below step 3) |
| `MACOS_CERTIFICATE_PASSWORD` | its password |

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
package type (`linux-x86_64-deb`, `-rpm`, `-appimage`, `windows-x86_64-nsis`,
`darwin-aarch64-app`): each install updates with the same kind of package it came from. A .deb or .rpm is installed with
`pkexec dpkg -i` / `rpm -U`, which asks for the administrator password; the
app then restarts. The endpoint is `plugins.updater.endpoints`; keep the old
URL listed there if the bucket ever moves to a custom domain.

### macOS signing certificate (once)

QuickDesk has no Apple Developer ID, so the Mac app is not notarized. Signed
ad hoc, macOS knows it only by the hash of each build: after every update it
asks for the login password again for each Keychain item QuickDesk reads, and
the Accessibility permission (needed for auto-paste) stops working until it is
turned off and on. Signed with a fixed certificate, macOS knows it by that
certificate, and "Always Allow" and the permission carry over to updates.
Gatekeeper treats both the same ("Open Anyway" for a downloaded .dmg).

```sh
scripts/make_macos_cert.sh   # writes ~/.tauri/quickdesk-macos.p12 (+ .password)
```

- Put the base64 it copies in the GitHub secret `MACOS_CERTIFICATE`, and the
  contents of `~/.tauri/quickdesk-macos.p12.password` in `MACOS_CERTIFICATE_PASSWORD`.
- **Back both files up.** A new certificate is a new app to macOS: every Mac
  asks once more.
- `scripts/macos_signing.sh` puts it on the build machine (release and macOS
  preview). Without the secrets the build is signed ad hoc, with a warning.
- To sign local builds the same way, import the .p12 into the login keychain and
  build with `APPLE_SIGNING_IDENTITY=<its SHA-1>` (the certificate is not
  trusted by the system, so codesign does not find it by name).

### 4. Sync server (QuickDesk Cloud)

`sync-server/` is a Cloudflare Worker on `https://sync.quickdesk.click` in front
of its own R2 bucket. It stores only what the app uploads, which is already
end-to-end encrypted; accounts are anonymous (random id + token = the sync
code). The app's endpoint is `qd_sync::cloud::DEFAULT_ENDPOINT`; `QD_SYNC_URL`
overrides it for testing.

Once (Node 22):

```sh
cd sync-server && npm install
npx wrangler login                        # opens the browser
npx wrangler r2 bucket create quickdesk-sync
npx wrangler deploy                       # also creates the sync.quickdesk.click record
curl https://sync.quickdesk.click/        # {"service":"QuickDesk Cloud sync","api":"v1"}
```

Redeploy with `npx wrangler deploy` after changing `sync-server/`. Locally:
`npx wrangler dev` (http://127.0.0.1:8787), then
`QD_SYNC_URL=http://127.0.0.1:8787 cargo test -p qd-sync --test cloud_live -- --ignored`.
Limits per account (object size, total size, object count) are at the top of
`sync-server/src/index.ts`; account creation is rate-limited per IP.

### 5. Website (Cloudflare Pages)

The site lives in `website/` (Astro): home (with the demo), `/download`,
`/docs`, `/changelog`, `/privacy`, each also under `/vi/` in Vietnamese. It
reads the newest release from `https://dl.quickdesk.click/latest.json` and the
changelog from `CHANGELOG*.md` at build time. The home page demo is **the real
UI**: `npm run build:demo` builds the frontend with Tauri's API swapped for
sample data (`src/demo/`, see `vite.config.ts`), and the site embeds it. When
you add a command, give it a demo answer in `src/demo/core.ts`.

Set up once in Cloudflare → **Workers & Pages** → **Create** → **Pages** →
**Connect to Git** → `TankGum/quickdesk`:

| Setting | Value |
|---|---|
| Production branch | `main` |
| Build command | `npm ci && npm run build:demo && npm ci --prefix website && npm run build --prefix website` |
| Build output directory | `website/dist` |
| Environment variable | `NODE_VERSION` = `22` |

Then **Custom domains** → add `quickdesk.click`, and **Settings → Builds →
Deploy hooks** → create one for `main` and store its URL as the
`CF_PAGES_DEPLOY_HOOK` secret: the Release workflow calls it so the site shows
a new version as soon as it is published. Every push to `main` also rebuilds
the site, and pull requests get preview URLs.

Run it locally (Node 22): `npm run build:demo && cd website && npm install && npm run dev`.

### 6. Download counts (dl-worker)

`dl-worker/` is a Cloudflare Worker (`quickdesk-dl`) on the route
`dl.quickdesk.click/*`. It passes every request through to the R2 bucket
unchanged (a Worker's `fetch()` to its own host goes to the origin, and any
error falls through to it too), then adds one to a daily total in the D1
database `quickdesk-downloads`, table `downloads`: day, kind (install, update,
`update.json` check…), system, version, source (website, terminal = curl/wget
e.g. `install.sh`, PowerShell = `install.ps1`, the app's updater, GitHub, direct
link, bot), country, n. No IP address, identifier or per-request row is stored;
`/privacy` says so. The classification is `dl-worker/src/classify.ts` (tested
with `npm test`). The old `r2.dev` URL, which QuickDesk 0.2.2's updater falls
back to, is not counted.

**Every download and update goes through it.** After changing it (Node 22):

```sh
cd dl-worker && npm install
npx wrangler login              # once
npm run migrate                 # if migrations/ changed
npm run deploy
```

then check that the host still serves exactly what the bucket holds:
`curl -fsSL https://dl.quickdesk.click/update.json | sha256sum` (and a package)
before and after, `curl -fsS -r 1000-1999 <package URL> | wc -c` (1000: ranges
work) and a few `curl -fsSI` requests (200). Right after a deploy the route can
answer 500 for a few seconds.

In Workers & Pages → `quickdesk-dl` → **Settings → Domains & Routes**, keep the
route's request limit failure mode on **Fail open**, so requests go straight to
the bucket if the free plan's daily Worker requests ever run out. Undo: remove
the route there; requests go straight to the bucket again.

**Reading the counts**

- Dashboard: **Storage & Databases → D1 → quickdesk-downloads → Console**, e.g.
  ```sql
  -- installs per version and system
  SELECT version, system, SUM(n) AS installs FROM downloads
  WHERE kind = 'install' GROUP BY version, system ORDER BY installs DESC;
  -- installs per day and source, last 30 days
  SELECT day, source, SUM(n) AS installs FROM downloads
  WHERE kind = 'install' AND day >= date('now', '-30 days') GROUP BY day, source ORDER BY day;
  -- installs per country
  SELECT country, SUM(n) AS installs FROM downloads WHERE kind = 'install' GROUP BY country ORDER BY installs DESC;
  -- update checks per day (about four per running app)
  SELECT day, SUM(n) AS checks FROM downloads WHERE kind = 'check' GROUP BY day ORDER BY day;
  ```
- Terminal: `npm run stats` (last 30 days: installs, updates, update checks, by
  system, source, version, country and day), `npm run stats -- --days 7`,
  `--json` for the raw rows. It uses wrangler's login; no token needed.

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
4. The workflow checks the tag matches the version and that release notes exist, builds the Linux
   packages on Ubuntu 22.04 (they then also install on newer distributions), the Windows
   installer on `windows-latest` and the macOS app on `macos-14` (Apple Silicon, signed ad hoc),
   and once **all three** built, uploads:
   ```
   releases/<version>/QuickDesk_<version>_amd64.deb
   releases/<version>/QuickDesk-<version>-1.x86_64.rpm
   releases/<version>/QuickDesk_<version>_amd64.AppImage
   releases/<version>/QuickDesk_<version>_x64-setup.exe   Windows (also its update package)
   releases/<version>/QuickDesk_<version>_aarch64.dmg     macOS disk image (website download)
   releases/<version>/QuickDesk_<version>_aarch64.app.tar.gz  macOS update package; install.sh uses it
   releases/<version>/SHA256SUMS
   releases/<version>/*.sig      updater signatures
   latest.json                   read by the website
   update.json                   read by the app's updater
   ```
   then creates the GitHub Release and rebuilds the website.
   The packages are also attached to the workflow run as an artifact.

Testing the upload without R2: run any S3-compatible server locally and set
`R2_ENDPOINT` (e.g. `http://127.0.0.1:9000`) before `python3 scripts/publish_r2.py`.

## Not yet

- Code signing for Windows: the installer is unsigned, so SmartScreen warns ("More info → Run
  anyway"). The updater's own signatures still protect updates.
- Code signing for macOS: the app is signed ad hoc (no Apple Developer ID), so a .dmg downloaded
  in a browser needs "Open Anyway" (install.sh avoids it: curl does not quarantine), and macOS
  may ask for Accessibility and the Keychain again after each update. With a Developer ID,
  set `bundle.macOS.signingIdentity` and notarize in the macos job.
- Intel Macs: only `aarch64-apple-darwin` is built.
