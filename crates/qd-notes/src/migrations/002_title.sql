-- Notes get an optional title; search covers both title and body.
ALTER TABLE notes ADD COLUMN title TEXT NOT NULL DEFAULT '';

DROP TRIGGER notes_fts_ai;
DROP TRIGGER notes_fts_ad;
DROP TRIGGER notes_fts_au;
DROP TABLE notes_fts;

CREATE VIRTUAL TABLE notes_fts USING fts5(
  title,
  body,
  content = 'notes',
  content_rowid = 'seq',
  tokenize = 'unicode61 remove_diacritics 2'
);

CREATE TRIGGER notes_fts_ai AFTER INSERT ON notes BEGIN
  INSERT INTO notes_fts (rowid, title, body) VALUES (new.seq, new.title, new.body);
END;
CREATE TRIGGER notes_fts_ad AFTER DELETE ON notes BEGIN
  INSERT INTO notes_fts (notes_fts, rowid, title, body) VALUES ('delete', old.seq, old.title, old.body);
END;
CREATE TRIGGER notes_fts_au AFTER UPDATE OF title, body ON notes BEGIN
  INSERT INTO notes_fts (notes_fts, rowid, title, body) VALUES ('delete', old.seq, old.title, old.body);
  INSERT INTO notes_fts (rowid, title, body) VALUES (new.seq, new.title, new.body);
END;

INSERT INTO notes_fts (notes_fts) VALUES ('rebuild');
