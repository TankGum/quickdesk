-- `seq` is an explicit INTEGER PRIMARY KEY so VACUUM never renumbers the rowids
-- that the external-content FTS index points at.
CREATE TABLE notes (
  seq         INTEGER PRIMARY KEY,
  id          TEXT NOT NULL UNIQUE,      -- UUIDv7, stable across devices
  body        TEXT NOT NULL,
  pinned      INTEGER NOT NULL DEFAULT 0,
  created_at  INTEGER NOT NULL,          -- unix ms
  updated_at  INTEGER NOT NULL,          -- unix ms, for display
  hlc         TEXT NOT NULL,             -- version, orders writes across devices
  base_hlc    TEXT,                      -- version a local edit started from
  deleted_at  INTEGER,                   -- tombstone
  conflict_of TEXT,
  dirty       INTEGER NOT NULL DEFAULT 1 -- pending push
);
CREATE INDEX notes_list ON notes (deleted_at, pinned DESC, updated_at DESC);
CREATE INDEX notes_dirty ON notes (dirty) WHERE dirty = 1;

CREATE VIRTUAL TABLE notes_fts USING fts5(
  body,
  content = 'notes',
  content_rowid = 'seq',
  tokenize = 'unicode61 remove_diacritics 2'
);

CREATE TRIGGER notes_fts_ai AFTER INSERT ON notes BEGIN
  INSERT INTO notes_fts (rowid, body) VALUES (new.seq, new.body);
END;
CREATE TRIGGER notes_fts_ad AFTER DELETE ON notes BEGIN
  INSERT INTO notes_fts (notes_fts, rowid, body) VALUES ('delete', old.seq, old.body);
END;
CREATE TRIGGER notes_fts_au AFTER UPDATE OF body ON notes BEGIN
  INSERT INTO notes_fts (notes_fts, rowid, body) VALUES ('delete', old.seq, old.body);
  INSERT INTO notes_fts (rowid, body) VALUES (new.seq, new.body);
END;
