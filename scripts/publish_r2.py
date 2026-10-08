#!/usr/bin/env python3
"""Publish the built Linux packages to a Cloudflare R2 (S3-compatible) bucket.

Bucket layout:

    releases/<version>/QuickDesk_<version>_amd64.deb
    releases/<version>/QuickDesk-<version>-1.x86_64.rpm
    releases/<version>/QuickDesk_<version>_amd64.AppImage
    releases/<version>/SHA256SUMS
    latest.json            what the newest version is and where to get it
    index.html             the download page, if packaging/download/index.html exists
    assets/...             files the page uses (packaging/download/assets)

The download page template may use these placeholders:
    {{VERSION}} {{DATE}}
    {{DEB_URL}} {{RPM_URL}} {{APPIMAGE_URL}}         (relative to the bucket root)
    {{DEB_SHA256}} {{RPM_SHA256}} {{APPIMAGE_SHA256}}
    {{DEB_SIZE}} {{RPM_SIZE}} {{APPIMAGE_SIZE}}      (e.g. "9.2 MB")

`python3 scripts/publish_r2.py --preview DIR` renders the page into DIR with the
current build's values instead of uploading, to check it in a browser.

Environment: R2_ACCOUNT_ID, R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY, R2_BUCKET.
R2_ENDPOINT overrides the endpoint (e.g. a local S3 server for testing).
Requires the `aws` CLI (preinstalled on GitHub-hosted runners).
"""

import datetime as dt
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BUNDLE = ROOT / "target" / "release" / "bundle"
TEMPLATE = ROOT / "packaging" / "download" / "index.html"
PAGE_ASSETS = TEMPLATE.parent / "assets"

CONTENT_TYPES = {
    ".deb": "application/vnd.debian.binary-package",
    ".rpm": "application/x-rpm",
    ".AppImage": "application/octet-stream",
    ".json": "application/json",
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


def page_assets() -> list[Path]:
    if not PAGE_ASSETS.is_dir():
        return []
    return sorted(p for p in PAGE_ASSETS.rglob("*") if p.is_file() and not p.name.startswith("."))


def render_page(version: str, published: str, files: dict) -> str:
    page = TEMPLATE.read_text()
    values = {"VERSION": version, "DATE": published[:10]}
    for kind, f in files.items():
        k = kind.upper()
        values[f"{k}_URL"] = f["url"]
        values[f"{k}_SHA256"] = f["sha256"]
        values[f"{k}_SIZE"] = human_size(f["size"])
    for name, value in values.items():
        page = page.replace("{{" + name + "}}", value)
    return page


def main() -> None:
    version = json.loads((ROOT / "src-tauri" / "tauri.conf.json").read_text())["version"]
    expected = {
        "deb": BUNDLE / "deb" / f"QuickDesk_{version}_amd64.deb",
        "rpm": BUNDLE / "rpm" / f"QuickDesk-{version}-1.x86_64.rpm",
        "appimage": BUNDLE / "appimage" / f"QuickDesk_{version}_amd64.AppImage",
    }
    missing = [str(p) for p in expected.values() if not p.is_file()]
    if missing:
        sys.exit("build output not found:\n  " + "\n  ".join(missing))

    prefix = f"releases/{version}"
    files = {}
    for kind, path in expected.items():
        files[kind] = {
            "name": path.name,
            "url": f"{prefix}/{path.name}",
            "sha256": sha256(path),
            "size": path.stat().st_size,
        }
    published = dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat()

    if len(sys.argv) == 3 and sys.argv[1] == "--preview":
        out = Path(sys.argv[2]).resolve()
        out.mkdir(parents=True, exist_ok=True)
        (out / "index.html").write_text(render_page(version, published, files))
        for asset in page_assets():
            dest = out / asset.relative_to(TEMPLATE.parent)
            dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_bytes(asset.read_bytes())
        print(f"preview written to {out / 'index.html'}")
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

    # Versioned files never change: cache them for a long time.
    immutable = "public, max-age=31536000, immutable"
    for kind, path in expected.items():
        upload(path, files[kind]["url"], immutable)
    upload(sums, f"{prefix}/SHA256SUMS", immutable)

    latest = out / "latest.json"
    latest.write_text(json.dumps({"version": version, "publishedAt": published, "files": files}, indent=2) + "\n")
    # Pointers to "the newest" must not be cached for long.
    fresh = "public, max-age=60"
    upload(latest, "latest.json", fresh)

    if TEMPLATE.is_file():
        # Assets keep fixed names across releases, so they get a short cache too.
        for asset in page_assets():
            upload(asset, asset.relative_to(TEMPLATE.parent).as_posix(), "public, max-age=300")
        index = out / "index.html"
        index.write_text(render_page(version, published, files))
        upload(index, "index.html", fresh)
    else:
        print(f"no download page template at {TEMPLATE.relative_to(ROOT)}; skipped index.html")

    print(f"published QuickDesk {version}")


if __name__ == "__main__":
    main()
