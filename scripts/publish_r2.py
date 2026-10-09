#!/usr/bin/env python3
"""Publish the built Linux packages to a Cloudflare R2 (S3-compatible) bucket.

Bucket layout:

    releases/<version>/QuickDesk_<version>_amd64.deb
    releases/<version>/QuickDesk-<version>-1.x86_64.rpm
    releases/<version>/QuickDesk_<version>_amd64.AppImage
    releases/<version>/SHA256SUMS
    latest.json            what the newest version is and where to get it
    update.json            signed manifest the app's updater reads (tauri-plugin-updater)
    index.html             the download page, if packaging/download/index.html exists
    assets/...             files the page uses (packaging/download/assets)
    demo/...               the real UI on sample data (`npm run build:demo` → dist-demo)

The download page template may use these placeholders:
    {{VERSION}} {{DATE}}
    {{DEB_URL}} {{RPM_URL}} {{APPIMAGE_URL}}         (relative to the bucket root)
    {{DEB_SHA256}} {{RPM_SHA256}} {{APPIMAGE_SHA256}}
    {{DEB_SIZE}} {{RPM_SIZE}} {{APPIMAGE_SIZE}}      (e.g. "9.2 MB")
    {{NOTES_HTML}}                                    this version's CHANGELOG.md section
    "{{NOTES_HTML_VI_JSON}}"                          CHANGELOG.vi.md's, as a JSON string

`python3 scripts/publish_r2.py --preview DIR` builds the demo and renders the
page into DIR instead of uploading; serve DIR over HTTP to check it (the demo
is an ES module app, which browsers do not load from file://). Without a local
`npx tauri build` the download links and sizes are placeholders.

Environment: R2_ACCOUNT_ID, R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY, R2_BUCKET, and
R2_PUBLIC_URL (the bucket's public base URL; update.json needs absolute links).
R2_ENDPOINT overrides the endpoint (e.g. a local S3 server for testing).
The packages must have been built with updater signatures (`<package>.sig`, from
TAURI_SIGNING_PRIVATE_KEY and `bundle.createUpdaterArtifacts`).
Requires the `aws` CLI (preinstalled on GitHub-hosted runners).
"""

import datetime as dt
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path

import changelog

ROOT = Path(__file__).resolve().parent.parent
BUNDLE = ROOT / "target" / "release" / "bundle"
TEMPLATE = ROOT / "packaging" / "download" / "index.html"
PAGE_ASSETS = TEMPLATE.parent / "assets"
DEMO = ROOT / "dist-demo"

CONTENT_TYPES = {
    ".deb": "application/vnd.debian.binary-package",
    ".rpm": "application/x-rpm",
    ".AppImage": "application/octet-stream",
    ".json": "application/json",
    ".sig": "text/plain; charset=utf-8",
    ".html": "text/html; charset=utf-8",
    ".css": "text/css; charset=utf-8",
    ".js": "text/javascript; charset=utf-8",
    ".svg": "image/svg+xml",
    ".png": "image/png",
    ".jpg": "image/jpeg",
    ".webp": "image/webp",
    ".woff2": "font/woff2",
    "": "text/plain; charset=utf-8",
}


def env(name: str) -> str:
    value = os.environ.get(name, "").strip()
    if not value:
        sys.exit(f"missing environment variable {name}")
    return value


def human_size(n: int) -> str:
    return f"{n / 1024 / 1024:.1f} MB"


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def page_files() -> list[tuple[Path, str]]:
    """(source, published path) for everything the page loads besides index.html."""
    files = []
    for base, prefix in ((PAGE_ASSETS, "assets"), (DEMO, "demo")):
        if base.is_dir():
            files += [(p, f"{prefix}/{p.relative_to(base).as_posix()}") for p in sorted(base.rglob("*")) if p.is_file() and not p.name.startswith(".")]
    return files


def notes(version: str) -> dict:
    """Release notes per language; empty only in --preview without a changelog entry."""
    return {lang: changelog.section(lang, version) or "" for lang in changelog.FILES}


def render_page(version: str, published: str, files: dict) -> str:
    page = TEMPLATE.read_text()
    n = notes(version)
    # Inside the page's JSON block of translations, so it must stay valid JSON.
    page = page.replace('"{{NOTES_HTML_VI_JSON}}"', json.dumps(changelog.to_html(n["vi"]), ensure_ascii=False))
    values = {"VERSION": version, "DATE": published[:10], "NOTES_HTML": changelog.to_html(n["en"])}
    for kind, f in files.items():
        k = kind.upper()
        values[f"{k}_URL"] = f["url"]
        values[f"{k}_SHA256"] = f["sha256"]
        values[f"{k}_SIZE"] = human_size(f["size"])
    for name, value in values.items():
        page = page.replace("{{" + name + "}}", value)
    return page


def updater_manifest(version: str, published: str, public: str, files: dict, sigs: dict) -> dict:
    """Static manifest in tauri-plugin-updater's format. `notes` is shown in
    the app; `notes_vi` is QuickDesk's own extra field for Vietnamese."""
    platform = {"deb": "linux-x86_64-deb", "rpm": "linux-x86_64-rpm", "appimage": "linux-x86_64-appimage"}
    n = notes(version)
    return {
        "version": version,
        "notes": n["en"],
        "notes_vi": n["vi"],
        "pub_date": published,
        "platforms": {
            platform[kind]: {"signature": sigs[kind].read_text().strip(), "url": f"{public}/{f['url']}"}
            for kind, f in files.items()
        },
    }


def main() -> None:
    version = json.loads((ROOT / "src-tauri" / "tauri.conf.json").read_text())["version"]
    expected = {
        "deb": BUNDLE / "deb" / f"QuickDesk_{version}_amd64.deb",
        "rpm": BUNDLE / "rpm" / f"QuickDesk-{version}-1.x86_64.rpm",
        "appimage": BUNDLE / "appimage" / f"QuickDesk_{version}_amd64.AppImage",
    }
    preview = len(sys.argv) == 3 and sys.argv[1] == "--preview"
    if not preview and not all(notes(version).values()):
        sys.exit(f"no release notes for {version}: add a '## {version}' section to CHANGELOG.md and CHANGELOG.vi.md")
    missing = [str(p) for p in expected.values() if not p.is_file()]
    if missing and not preview:
        sys.exit("build output not found:\n  " + "\n  ".join(missing))
    if TEMPLATE.is_file() and not (DEMO / "index.html").is_file() and not preview:
        sys.exit("dist-demo not found: run `npm run build:demo` first")

    prefix = f"releases/{version}"
    files = {}
    for kind, path in expected.items():
        files[kind] = {
            "name": path.name,
            "url": f"{prefix}/{path.name}",
            "sha256": sha256(path) if path.is_file() else "",
            "size": path.stat().st_size if path.is_file() else 0,
        }
    published = dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat()

    if preview:
        subprocess.run(["npm", "run", "--silent", "build:demo"], cwd=ROOT, check=True)
        out = Path(sys.argv[2]).resolve()
        out.mkdir(parents=True, exist_ok=True)
        (out / "index.html").write_text(render_page(version, published, files))
        for src, dest in page_files():
            target = out / dest
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(src.read_bytes())
        print(f"preview written to {out}; view it with: python3 -m http.server -d {out} 8000")
        return

    bucket = env("R2_BUCKET")
    endpoint = os.environ.get("R2_ENDPOINT") or f"https://{env('R2_ACCOUNT_ID')}.r2.cloudflarestorage.com"
    aws_env = {
        **os.environ,
        "AWS_ACCESS_KEY_ID": env("R2_ACCESS_KEY_ID"),
        "AWS_SECRET_ACCESS_KEY": env("R2_SECRET_ACCESS_KEY"),
        "AWS_DEFAULT_REGION": "auto",
        # R2 rejects the CRC checksums newer aws CLIs send by default.
        "AWS_REQUEST_CHECKSUM_CALCULATION": "when_required",
        "AWS_RESPONSE_CHECKSUM_VALIDATION": "when_required",
    }

    def upload(path: Path, key: str, cache: str) -> None:
        ctype = CONTENT_TYPES.get(path.suffix, "application/octet-stream")
        cmd = [
            "aws", "s3", "cp", str(path), f"s3://{bucket}/{key}",
            "--endpoint-url", endpoint, "--content-type", ctype, "--cache-control", cache, "--only-show-errors",
        ]
        print(f"upload {key}")
        subprocess.run(cmd, check=True, env=aws_env)

    out = ROOT / "target" / "release" / "publish"
    out.mkdir(parents=True, exist_ok=True)
    sums = out / "SHA256SUMS"
    sums.write_text("".join(f"{f['sha256']}  {f['name']}\n" for f in files.values()))

    # Updater signatures sit next to each package.
    sigs = {kind: path.with_name(path.name + ".sig") for kind, path in expected.items()}
    unsigned = [str(p) for p in sigs.values() if not p.is_file()]
    if unsigned:
        sys.exit("updater signatures not found (build with TAURI_SIGNING_PRIVATE_KEY and createUpdaterArtifacts):\n  " + "\n  ".join(unsigned))
    public = env("R2_PUBLIC_URL").rstrip("/")

    # Versioned files never change: cache them for a long time.
    immutable = "public, max-age=31536000, immutable"
    for kind, path in expected.items():
        upload(path, files[kind]["url"], immutable)
        upload(sigs[kind], files[kind]["url"] + ".sig", immutable)
    upload(sums, f"{prefix}/SHA256SUMS", immutable)

    latest = out / "latest.json"
    latest.write_text(json.dumps({"version": version, "publishedAt": published, "notes": notes(version), "files": files}, indent=2) + "\n")
    # Pointers to "the newest" must not be cached for long.
    fresh = "public, max-age=60"
    upload(latest, "latest.json", fresh)

    # The app picks `linux-x86_64-<bundle type>`, so each install updates with
    # the same kind of package it was installed from.
    update = out / "update.json"
    update.write_text(json.dumps(updater_manifest(version, published, public, files, sigs), indent=2) + "\n")
    upload(update, "update.json", fresh)

    if TEMPLATE.is_file():
        # Assets keep fixed names across releases, so they get a short cache too.
        for src, dest in page_files():
            upload(src, dest, "public, max-age=300")
        index = out / "index.html"
        index.write_text(render_page(version, published, files))
        upload(index, "index.html", fresh)
    else:
        print(f"no download page template at {TEMPLATE.relative_to(ROOT)}; skipped index.html")

    print(f"published QuickDesk {version}")


if __name__ == "__main__":
    main()
