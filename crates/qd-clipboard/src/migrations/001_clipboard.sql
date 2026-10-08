CREATE TABLE clip_entries (
  id              INTEGER PRIMARY KEY,
  content         TEXT NOT NULL,
  content_hash    BLOB NOT NULL UNIQUE,      -- blake3, for dedup
  kind            TEXT NOT NULL DEFAULT 'text',
  source_app      TEXT,
  pinned          INTEGER NOT NULL DEFAULT 0,
  first_copied_at INTEGER NOT NULL,
  last_copied_at  INTEGER NOT NULL,
  copy_count      INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX clip_recent ON clip_entries (pinned DESC, last_copied_at DESC);

-- Trigram: substring search ("8000", "pods", part of a URL), accent-insensitive.
CREATE VIRTUAL TABLE clip_fts USING fts5(
  content,
  content = 'clip_entries',
  content_rowid = 'id',
  tokenize = 'trigram remove_diacritics 1'
);

CREATE TRIGGER clip_fts_ai AFTER INSERT ON clip_entries BEGIN
  INSERT INTO clip_fts (rowid, content) VALUES (new.id, new.content);
END;
CREATE TRIGGER clip_fts_ad AFTER DELETE ON clip_entries BEGIN
  INSERT INTO clip_fts (clip_fts, rowid, content) VALUES ('delete', old.id, old.content);
END;
CREATE TRIGGER clip_fts_au AFTER UPDATE OF content ON clip_entries BEGIN
  INSERT INTO clip_fts (clip_fts, rowid, content) VALUES ('delete', old.id, old.content);
  INSERT INTO clip_fts (rowid, content) VALUES (new.id, new.content);
END;
