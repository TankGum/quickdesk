#!/usr/bin/env python3
"""Prepare a release: bump the version everywhere, commit and tag.

    npm run release 0.2.3            bump, commit "Release 0.2.3", tag v0.2.3
    npm run release 0.2.3 -- --push  …and push main and the tag (starts the Release workflow)
    npm run release 0.2.3 -- --dry-run

Before running it, write the release notes: a `## 0.2.3 — <date>` section in
both CHANGELOG.md and CHANGELOG.vi.md. They may be uncommitted; they go into
the release commit. Any other uncommitted change stops the script, so a
release is exactly what is on the branch.
"""

import json
import re
import subprocess
import sys
from pathlib import Path

import changelog

ROOT = Path(__file__).resolve().parent.parent
TAURI_CONF = ROOT / "src-tauri" / "tauri.conf.json"
PACKAGE = ROOT / "package.json"
CARGO = ROOT / "Cargo.toml"
NOTES = {"CHANGELOG.md", "CHANGELOG.vi.md"}


def run(*cmd: str, check: bool = True) -> str:
    return subprocess.run(cmd, cwd=ROOT, check=check, capture_output=True, text=True).stdout.strip()


def fail(msg: str) -> None:
    sys.exit(f"release: {msg}")


def parse(v: str) -> tuple[int, int, int]:
    m = re.fullmatch(r"(\d+)\.(\d+)\.(\d+)", v)
    if not m:
        fail(f"'{v}' is not a version like 0.2.3")
    return tuple(int(x) for x in m.groups())  # type: ignore[return-value]


def main() -> None:
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    flags = {a for a in sys.argv[1:] if a.startswith("--")}
    if len(args) != 1 or flags - {"--push", "--dry-run"}:
        sys.exit(__doc__)
    version = args[0].lstrip("v")
    dry, push = "--dry-run" in flags, "--push" in flags

    current = json.loads(TAURI_CONF.read_text())["version"]
    if parse(version) <= parse(current):
        fail(f"{version} must be newer than the current version {current}")

    # Clean tree, except the release notes being written for this release.
    # Untracked files are fine: only the files below go into the release commit.
    changed = run("git", "status", "--porcelain", "--untracked-files=no").splitlines()
    dirty = [p for p in (re.sub(r"^\s*\S+\s+", "", line) for line in changed) if p not in NOTES]
    if dirty:
        fail("commit or stash these first:\n  " + "\n  ".join(dirty))
    if run("git", "tag", "--list", f"v{version}"):
        fail(f"tag v{version} already exists")
    for lang in changelog.FILES:
        if not changelog.section(lang, version):
            fail(f"no '## {version}' section in {changelog.FILES[lang].name}: write the release notes first")
    branch = run("git", "rev-parse", "--abbrev-ref", "HEAD")
    if branch != "main":
        print(f"note: releasing from branch '{branch}', not main")

    print(f"{current} → {version}")
    if dry:
        print("dry run: nothing changed")
        return

    # tauri.conf.json and package.json: only the top-level "version" line.
    for path in (TAURI_CONF, PACKAGE):
        text = path.read_text()
        new, n = re.subn(r'^(  "version": ")[^"]+(")', rf"\g<1>{version}\g<2>", text, count=1, flags=re.M)
        if n != 1:
            fail(f"could not find the version in {path.name}")
        path.write_text(new)
    # Cargo.toml: the workspace version.
    text = CARGO.read_text()
    new, n = re.subn(r'(\[workspace\.package\][^\[]*?\nversion = ")[^"]+(")', rf"\g<1>{version}\g<2>", text, count=1, flags=re.S)
    if n != 1:
        fail("could not find [workspace.package] version in Cargo.toml")
    CARGO.write_text(new)

    # Keep the lock files in step (offline: only the workspace's own entries change).
    run("npm", "install", "--package-lock-only", "--ignore-scripts", "--no-audit", "--no-fund")
    if subprocess.run(["cargo", "update", "--workspace", "--offline"], cwd=ROOT, capture_output=True).returncode != 0:
        print("note: could not refresh Cargo.lock offline; CI updates it while building")

    files = [str(p.relative_to(ROOT)) for p in (TAURI_CONF, PACKAGE, CARGO)]
    files += ["package-lock.json", "Cargo.lock", *sorted(NOTES)]
    run("git", "add", *files)
    run("git", "commit", "-m", f"Release {version}")
    run("git", "tag", f"v{version}")
    print(f"committed and tagged v{version}")

    if push:
        print(run("git", "push", "origin", branch, f"v{version}") or f"pushed {branch} and v{version}")
        print("The Release workflow is running: https://github.com/TankGum/quickdesk/actions")
    else:
        print(f"next: git push origin {branch} v{version}   (starts the Release workflow)")


if __name__ == "__main__":
    main()
