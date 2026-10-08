-- Images and copied files. Image bytes live in files next to the database
-- (`blob`); the row keeps a small PNG thumbnail for the list.
ALTER TABLE clip_entries ADD COLUMN mime TEXT;
ALTER TABLE clip_entries ADD COLUMN blob TEXT;
ALTER TABLE clip_entries ADD COLUMN byte_size INTEGER NOT NULL DEFAULT 0;
ALTER TABLE clip_entries ADD COLUMN width INTEGER;
ALTER TABLE clip_entries ADD COLUMN height INTEGER;
ALTER TABLE clip_entries ADD COLUMN thumb BLOB;
UPDATE clip_entries SET byte_size = length(CAST(content AS BLOB));
CREATE INDEX clip_kind ON clip_entries (kind, last_copied_at DESC);
