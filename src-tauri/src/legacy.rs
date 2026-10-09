//! Bring data from earlier app identifiers into the current data directory.
//!
//! QuickDesk was `dev.quickdesk.app` (0.1.x), then `io.github.tankgum.quickdesk`
//! (0.2.x), and is now `click.quickdesk`. The data directory follows the
//! identifier, so after each rename the app starts in a new, empty directory.
//!
//! The first attempt (a rename that ran only when the new directory did not
//! exist yet) never ran: WebKit creates the new directory for its own storage
//! before our setup code runs. This version does not care whether the new
//! directory exists. For each earlier directory that still holds a database and
//! has not been imported yet, it merges notes, clipboard history (with images)
//! and settings into the current database, then leaves a marker file there.
//! The old directory itself is kept untouched as a backup.

use std::fs;
use std::path::Path;

use qd_core::{Db, Module};

/// Earlier identifiers, newest first.
pub const LEGACY_IDENTIFIERS: [&str; 2] = ["io.github.tankgum.quickdesk", "dev.quickdesk.app"];

const DB: &str = "quickdesk.db";
const BLOBS: &str = "clipboard-images";
/// Written into an old directory once its data is in the current one.
pub const MARKER: &str = "IMPORTED-INTO-QUICKDESK.txt";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Imported {
    pub notes: usize,
    pub clips: usize,
    pub images: usize,
}

/// Import every earlier data directory that has not been imported yet. Errors
/// are logged and leave the old directory as it was, to be retried next start.
pub fn import_all(data_dir: &Path, modules: &[&dyn Module]) {
    let Some(root) = data_dir.parent() else { return };
    for (i, id) in LEGACY_IDENTIFIERS.iter().enumerate() {
        let old = root.join(id);
        if old == data_dir || !old.join(DB).is_file() || old.join(MARKER).exists() {
            continue;
        }
        // Settings from the newest earlier version win over the defaults the
        // current version just wrote; older ones only fill gaps.
        match import(&old, data_dir, modules, i == 0) {
            Ok(n) => {
                tracing::info!(from = %old.display(), notes = n.notes, clips = n.clips, images = n.images, "imported earlier data");
                let note = format!(
                    "QuickDesk copied the notes, clipboard history and settings in this folder\n\
                     into {} ({} notes, {} clipboard entries, {} images).\n\
                     This folder is no longer used; you can delete it.\n",
                    data_dir.display(),
                    n.notes,
                    n.clips,
                    n.images
                );
                if let Err(e) = fs::write(old.join(MARKER), note) {
                    tracing::warn!(error = %e, "could not mark the old data folder as imported");
                }
            }
            Err(e) => tracing::error!(from = %old.display(), error = %e, "could not import earlier data; will retry"),
        }
    }
}

fn import(old: &Path, data_dir: &Path, modules: &[&dyn Module], prefer_old_settings: bool) -> Result<Imported> {
    fs::create_dir_all(data_dir)?;

    // Work on a copy of the old database (with its write-ahead log, which holds
    // the most recent changes), upgraded to the current schema. The original
    // stays exactly as it was.
    let copy = data_dir.join(".import.db");
    remove_db_files(&copy);
    fs::copy(old.join(DB), &copy)?;
    let wal = old.join(format!("{DB}-wal"));
    if wal.is_file() {
        fs::copy(&wal, data_dir.join(".import.db-wal"))?;
    }
    drop(Db::open(&copy, modules)?);

    let result = merge(&copy, data_dir, modules, prefer_old_settings);
    remove_db_files(&copy);
    let mut n = result?;
    n.images = copy_blobs(&old.join(BLOBS), &data_dir.join(BLOBS))?;
    Ok(n)
}

fn merge(copy: &Path, data_dir: &Path, modules: &[&dyn Module], prefer_old_settings: bool) -> Result<Imported> {
    let db = Db::open(&data_dir.join(DB), modules)?;
    let mut conn = db.conn()?;
    conn.execute("ATTACH DATABASE ?1 AS legacy", [copy.to_string_lossy()])?;
    let tx = conn.transaction()?;
    // Notes have globally unique ids; when both sides have one, keep the
    // newer version (HLC strings sort in causal order).
    let notes = tx.execute(
        "INSERT INTO main.notes (id, title, body, pinned, created_at, updated_at, hlc, base_hlc, deleted_at, conflict_of, dirty)
         SELECT id, title, body, pinned, created_at, updated_at, hlc, base_hlc, deleted_at, conflict_of, dirty
         FROM legacy.notes WHERE true
         ON CONFLICT(id) DO UPDATE SET
           title = excluded.title, body = excluded.body, pinned = excluded.pinned,
           created_at = excluded.created_at, updated_at = excluded.updated_at, hlc = excluded.hlc,
           base_hlc = excluded.base_hlc, deleted_at = excluded.deleted_at,
           conflict_of = excluded.conflict_of, dirty = excluded.dirty
         WHERE excluded.hlc > notes.hlc",
        [],
    )?;
    // Clipboard entries are unique by content hash; an entry copied on both
    // sides keeps the current one.
    let clips = tx.execute(
        "INSERT OR IGNORE INTO main.clip_entries
           (content, content_hash, kind, source_app, pinned, first_copied_at, last_copied_at, copy_count,
            mime, blob, byte_size, width, height, thumb)
         SELECT content, content_hash, kind, source_app, pinned, first_copied_at, last_copied_at, copy_count,
                mime, blob, byte_size, width, height, thumb
         FROM legacy.clip_entries",
        [],
    )?;
    let verb = if prefer_old_settings { "REPLACE" } else { "IGNORE" };
    tx.execute(
        &format!("INSERT OR {verb} INTO main.settings (key, value) SELECT key, value FROM legacy.settings"),
        [],
    )?;
    tx.commit()?;
    conn.execute("DETACH DATABASE legacy", [])?;
    Ok(Imported { notes, clips, images: 0 })
}

/// Copy image files the current folder does not have yet.
fn copy_blobs(from: &Path, to: &Path) -> Result<usize> {
    let Ok(entries) = fs::read_dir(from) else { return Ok(0) };
    fs::create_dir_all(to)?;
    let mut copied = 0;
    for entry in entries.flatten() {
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_file() && !target.exists() {
            fs::copy(entry.path(), target)?;
            copied += 1;
        }
    }
    Ok(copied)
}

fn remove_db_files(db: &Path) {
    for suffix in ["", "-wal", "-shm", "-journal"] {
        let _ = fs::remove_file(format!("{}{suffix}", db.display()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qd_clipboard::ClipboardModule;
    use qd_notes::NotesModule;

    const MODULES: &[&dyn Module] = &[&NotesModule, &ClipboardModule];

    fn note(db: &Db, id: &str, body: &str, hlc: &str) {
        db.conn()
            .unwrap()
            .execute(
                "INSERT INTO notes (id, body, created_at, updated_at, hlc) VALUES (?1, ?2, 0, 0, ?3)",
                rusqlite::params![id, body, hlc],
            )
            .unwrap();
    }

    fn clip(db: &Db, content: &str, blob: Option<&str>) {
        db.conn()
            .unwrap()
            .execute(
                "INSERT INTO clip_entries (content, content_hash, first_copied_at, last_copied_at, blob) VALUES (?1, ?1, 0, 0, ?2)",
                rusqlite::params![content, blob],
            )
            .unwrap();
    }

    fn bodies(db: &Db) -> Vec<String> {
        let conn = db.conn().unwrap();
        let mut stmt = conn.prepare("SELECT body FROM notes ORDER BY id").unwrap();
        stmt.query_map([], |r| r.get(0)).unwrap().map(|r| r.unwrap()).collect()
    }

    /// The real failure: the new directory already exists (WebKit made it) and
    /// already has a fresh database when the import runs.
    #[test]
    fn merges_into_an_existing_directory_and_keeps_the_old_one() {
        let root = std::env::temp_dir().join(format!("qd-legacy-{}", qd_core::ids::new_id()));
        let (old, new) = (root.join(LEGACY_IDENTIFIERS[0]), root.join("click.quickdesk"));
        fs::create_dir_all(old.join(BLOBS)).unwrap();
        fs::create_dir_all(new.join("CacheStorage")).unwrap();
        {
            let db = Db::open(&old.join(DB), MODULES).unwrap();
            note(&db, "a", "old note", "0000000000001-000000-x");
            note(&db, "shared", "older edit", "0000000000001-000000-x");
            clip(&db, "copied before", Some("img1.png"));
            clip(&db, "copied twice", None);
            db.conn().unwrap().execute("INSERT INTO settings VALUES ('ui.language', '\"vi\"')", []).unwrap();
            fs::write(old.join(BLOBS).join("img1.png"), b"png").unwrap();
        }
        {
            let db = Db::open(&new.join(DB), MODULES).unwrap();
            note(&db, "b", "new note", "0000000000002-000000-y");
            note(&db, "shared", "newer edit", "0000000000002-000000-y");
            clip(&db, "copied twice", None);
            db.conn().unwrap().execute("INSERT INTO settings VALUES ('ui.language', '\"auto\"')", []).unwrap();
        }

        import_all(&new, MODULES);

        let db = Db::open(&new.join(DB), MODULES).unwrap();
        assert_eq!(bodies(&db), ["old note", "new note", "newer edit"]);
        let conn = db.conn().unwrap();
        let clips: i64 = conn.query_row("SELECT COUNT(*) FROM clip_entries", [], |r| r.get(0)).unwrap();
        assert_eq!(clips, 2, "duplicates are not imported twice");
        let lang: String =
            conn.query_row("SELECT value FROM settings WHERE key = 'ui.language'", [], |r| r.get(0)).unwrap();
        assert_eq!(lang, "\"vi\"", "the user's earlier settings win over fresh defaults");
        let found: i64 =
            conn.query_row("SELECT COUNT(*) FROM notes_fts WHERE notes_fts MATCH 'old'", [], |r| r.get(0)).unwrap();
        assert_eq!(found, 1, "imported notes are searchable");
        drop(conn);
        assert_eq!(fs::read(new.join(BLOBS).join("img1.png")).unwrap(), b"png");
        assert!(old.join(DB).is_file(), "the old folder is kept as a backup");
        assert!(old.join(MARKER).is_file());
        assert!(!new.join(".import.db").exists());

        // A second start imports nothing again.
        db.conn().unwrap().execute("DELETE FROM notes WHERE id = 'a'", []).unwrap();
        drop(db);
        import_all(&new, MODULES);
        let db = Db::open(&new.join(DB), MODULES).unwrap();
        assert_eq!(bodies(&db), ["new note", "newer edit"], "a deleted note does not come back");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn imports_every_earlier_version_newest_settings_first() {
        let root = std::env::temp_dir().join(format!("qd-legacy-{}", qd_core::ids::new_id()));
        let new = root.join("click.quickdesk");
        for (id, body, lang) in
            [(LEGACY_IDENTIFIERS[0], "from 0.2", "\"vi\""), (LEGACY_IDENTIFIERS[1], "from 0.1", "\"en\"")]
        {
            let dir = root.join(id);
            fs::create_dir_all(&dir).unwrap();
            let db = Db::open(&dir.join(DB), MODULES).unwrap();
            note(&db, body, body, "0000000000001-000000-x");
            db.conn().unwrap().execute("INSERT INTO settings VALUES ('ui.language', ?1)", [lang]).unwrap();
        }

        import_all(&new, MODULES);

        let db = Db::open(&new.join(DB), MODULES).unwrap();
        assert_eq!(bodies(&db), ["from 0.1", "from 0.2"]);
        let lang: String = db
            .conn()
            .unwrap()
            .query_row("SELECT value FROM settings WHERE key = 'ui.language'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(lang, "\"vi\"");
        fs::remove_dir_all(root).unwrap();
    }
}

/// Manual check against a copy of real data folders:
/// `QD_IMPORT_ROOT=/path/to/copy cargo test -p quickdesk legacy::real -- --ignored`
#[cfg(test)]
mod real {
    #[test]
    #[ignore = "needs QD_IMPORT_ROOT with copies of real data folders"]
    fn import_a_copy_of_real_data() {
        let root = std::path::PathBuf::from(std::env::var("QD_IMPORT_ROOT").expect("QD_IMPORT_ROOT"));
        super::import_all(&root.join("click.quickdesk"), &[&qd_notes::NotesModule, &qd_clipboard::ClipboardModule]);
    }
}
