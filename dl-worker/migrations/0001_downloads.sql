-- Daily totals: one row per day and kind of download, counted up as requests
-- come in. No per-request rows, so nothing here points to a person.
CREATE TABLE downloads (
  day     TEXT NOT NULL,              -- UTC, YYYY-MM-DD
  kind    TEXT NOT NULL,              -- install, update, check, latest, sums, signature, other
  system  TEXT NOT NULL,              -- linux-deb, linux-rpm, linux-appimage, windows, macos-dmg, macos-app, or ''
  version TEXT NOT NULL,              -- 0.6.0, or ''
  source  TEXT NOT NULL,              -- website, terminal, powershell, updater, github, direct, bot, other
  country TEXT NOT NULL,              -- two-letter code from Cloudflare, or ''
  n       INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (day, kind, system, version, source, country)
) WITHOUT ROWID;
