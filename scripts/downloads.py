#!/usr/bin/env python3
"""Download counts for dl.quickdesk.click, from the D1 database the dl-worker
keeps (see dl-worker/ and docs/RELEASING.md). Uses wrangler's login (run
`npx wrangler login` once in dl-worker/); no API token needed.

    npm run stats                  the last 30 days
    npm run stats -- --days 7
    npm run stats -- --json        the raw rows

--from-file FILE reads a saved `wrangler d1 execute --json` output instead.

The table holds daily totals: day, kind (install, update, check = an
update.json request, latest, sums, signature, other), system, version, source
(website, terminal = curl/wget e.g. install.sh, powershell = install.ps1,
updater, github, direct, bot, other), country, n.
"""
import argparse
import json
import subprocess
import sys
from collections import Counter
from pathlib import Path

WORKER = Path(__file__).resolve().parent.parent / "dl-worker"
DATABASE = "quickdesk-downloads"


def fail(msg: str) -> None:
    sys.exit(f"stats: {msg}")


def fetch(days: int) -> list:
    sql = (
        "SELECT day, kind, system, version, source, country, n FROM downloads "
        f"WHERE day >= date('now', '-{days - 1} days') ORDER BY day"
    )
    cmd = ["npx", "--no-install", "wrangler", "d1", "execute", DATABASE, "--remote", "--json", "--command", sql]
    try:
        out = subprocess.run(cmd, cwd=WORKER, capture_output=True, text=True, timeout=120)
    except FileNotFoundError:
        fail("npx not found: install Node 22")
    if out.returncode != 0:
        fail(f"wrangler failed (run `npm install` and `npx wrangler login` in dl-worker/):\n{out.stderr.strip()[-600:]}")
    return parse(out.stdout)


def parse(text: str) -> list:
    data = json.loads(text)
    return data[0].get("results", []) if isinstance(data, list) and data else []


def table(title: str, counts: Counter, limit: int = 0) -> None:
    rows = counts.most_common(limit or None)
    if not rows:
        return
    print(f"\n{title}")
    width = max(len(str(k) or "-") for k, _ in rows)
    for key, n in rows:
        print(f"  {str(key) or '-':<{width}}  {n:>7,}")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--days", type=int, default=30)
    ap.add_argument("--json", action="store_true", help="print the rows as JSON")
    ap.add_argument("--from-file", help="read a saved `wrangler d1 execute --json` output")
    args = ap.parse_args()
    if args.days < 1:
        fail("--days must be at least 1")

    if args.from_file:
        rows = parse(Path(args.from_file).read_text())
    else:
        rows = fetch(args.days)
    for r in rows:
        r["n"] = int(r["n"])
    if args.json:
        print(json.dumps(rows, indent=2))
        return

    by_kind: Counter = Counter()
    installs = {k: Counter() for k in ("system", "source", "version", "country")}
    updates: Counter = Counter()
    days: dict = {}
    for r in rows:
        by_kind[r["kind"]] += r["n"]
        days.setdefault(r["day"], Counter())[r["kind"]] += r["n"]
        if r["kind"] == "install":
            for k in installs:
                installs[k][r[k]] += r["n"]
        elif r["kind"] == "update":
            updates[f"{r['version']} {r['system']}"] += r["n"]

    print(f"dl.quickdesk.click, last {args.days} days")
    print(f"  installs        {by_kind['install']:>7,}   downloads of a package, not by the app's updater")
    print(f"  updates         {by_kind['update']:>7,}   packages downloaded by the app's updater")
    print(f"  update checks   {by_kind['check']:>7,}   update.json requests (each running app: at start, then every 6 h)")
    table("Installs by system", installs["system"])
    table("Installs by source", installs["source"])
    table("Installs by version", installs["version"])
    table("Installs by country (top 15)", installs["country"], 15)
    table("Updates to", updates)
    if days:
        print("\nPer day           installs  updates   checks")
        for day in sorted(days):
            c = days[day]
            print(f"  {day}     {c['install']:>7,}  {c['update']:>7,}  {c['check']:>7,}")


if __name__ == "__main__":
    main()
