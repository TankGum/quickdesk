#!/usr/bin/env python3
"""Publish the built Linux and Windows packages to the download host
(Cloudflare R2, served at https://dl.quickdesk.click).

Bucket layout:

    releases/<version>/QuickDesk_<version>_amd64.deb      (+ .sig)
    releases/<version>/QuickDesk-<version>-1.x86_64.rpm   (+ .sig)
    releases/<version>/QuickDesk_<version>_amd64.AppImage (+ .sig)
    releases/<version>/QuickDesk_<version>_x64-setup.exe  (+ .sig, Windows)
    releases/<version>/SHA256SUMS
    latest.json            the newest version, its files and release notes (read by the website)
    update.json            signed manifest the app's updater reads (tauri-plugin-updater)

The website (website/, Cloudflare Pages) is built separately and reads
latest.json; the Release workflow asks Pages to rebuild after this runs.

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

CONTENT_TYPES = {
    ".deb": "application/vnd.debian.binary-package",
    ".rpm": "application/x-rpm",
    ".AppImage": "application/octet-stream",
    ".exe": "application/vnd.microsoft.portable-executable",
    ".json": "application/json",
    ".sig": "text/plain; charset=utf-8",
    "": "text/plain; charset=utf-8",
}


def env(name: str) -> str:
    value = os.environ.get(name, "").strip()
    if not value:
        sys.exit(f"missing environment variable {name}")
    return value


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def notes(version: str) -> dict:
    """Release notes per language (Markdown), from CHANGELOG.md / CHANGELOG.vi.md."""
    return {lang: changelog.section(lang, version) or "" for lang in changelog.FILES}


def updater_manifest(version: str, published: str, public: str, files: dict, sigs: dict) -> dict:
    """Static manifest in tauri-plugin-updater's format. `notes` is shown in
    the app; `notes_vi` is QuickDesk's own extra field for Vietnamese."""
    # The updater asks for `<os>-<arch>-<installer>` first, then `<os>-<arch>`.
    platform = {
        "deb": ["linux-x86_64-deb"],
        "rpm": ["linux-x86_64-rpm"],
        "appimage": ["linux-x86_64-appimage"],
        "windows": ["windows-x86_64-nsis", "windows-x86_64"],
    }
    n = notes(version)
    return {
        "version": version,
        "notes": n["en"],
        "notes_vi": n["vi"],
        "pub_date": published,
        "platforms": {
            key: {"signature": sigs[kind].read_text().strip(), "url": f"{public}/{f['url']}"}
            for kind, f in files.items()
            for key in platform[kind]
        },
    }


def main() -> None:
    version = json.loads((ROOT / "src-tauri" / "tauri.conf.json").read_text())["version"]
    expected = {
        "deb": BUNDLE / "deb" / f"QuickDesk_{version}_amd64.deb",
        "rpm": BUNDLE / "rpm" / f"QuickDesk-{version}-1.x86_64.rpm",
        "appimage": BUNDLE / "appimage" / f"QuickDesk_{version}_amd64.AppImage",
        "windows": BUNDLE / "nsis" / f"QuickDesk_{version}_x64-setup.exe",
    }
    if not all(notes(version).values()):
        sys.exit(f"no release notes for {version}: add a '## {version}' section to CHANGELOG.md and CHANGELOG.vi.md")
    missing = [str(p) for p in expected.values() if not p.is_file()]
    if missing:
        sys.exit("build output not found:\n  " + "\n  ".join(missing))
    # Updater signatures sit next to each package.
    sigs = {kind: path.with_name(path.name + ".sig") for kind, path in expected.items()}
    unsigned = [str(p) for p in sigs.values() if not p.is_file()]
    if unsigned:
        sys.exit("updater signatures not found (build with TAURI_SIGNING_PRIVATE_KEY and createUpdaterArtifacts):\n  " + "\n  ".join(unsigned))
    public = env("R2_PUBLIC_URL").rstrip("/")

    prefix = f"releases/{version}"
    files = {
        kind: {"name": path.name, "url": f"{prefix}/{path.name}", "sha256": sha256(path), "size": path.stat().st_size}
        for kind, path in expected.items()
    }
    published = dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat()

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
        upload(sigs[kind], files[kind]["url"] + ".sig", immutable)
    upload(sums, f"{prefix}/SHA256SUMS", immutable)

    # Pointers to "the newest" must not be cached for long.
    fresh = "public, max-age=60"
    latest = out / "latest.json"
    latest.write_text(json.dumps({"version": version, "publishedAt": published, "notes": notes(version), "files": files}, indent=2) + "\n")
    upload(latest, "latest.json", fresh)

    # The app picks `<os>-x86_64-<bundle type>`, so each install updates with
    # the same kind of package it was installed from.
    update = out / "update.json"
    update.write_text(json.dumps(updater_manifest(version, published, public, files, sigs), indent=2) + "\n")
    upload(update, "update.json", fresh)

    print(f"published QuickDesk {version}")


if __name__ == "__main__":
    main()
