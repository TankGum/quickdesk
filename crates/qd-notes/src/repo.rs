//! Local CRUD. Every write bumps the note's HLC and marks it dirty for sync.

use qd_core::{ids, Clock};
use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::{fts, Error, Note, Result};

const NOTE_COLS: &str = "id, body, pinned, created_at, updated_at, conflict_of";

pub(crate) fn note_from_row(r: &Row<'_>) -> rusqlite::Result<Note> {
    Ok(Note {
        id: r.get(0)?,
        body: r.get(1)?,
        pinned: r.get(2)?,
        created_at: r.get(3)?,
        updated_at: r.get(4)?,
        conflict_of: r.get(5)?,
    })
}

pub(crate) fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn normalize(body: &str) -> Result<String> {
    let body = body.trim();
    if body.is_empty() {
        return Err(Error::EmptyBody);
    }
    Ok(body.to_owned())
}

pub fn create(conn: &Connection, clock: &Clock, body: &str) -> Result<Note> {
    let body = normalize(body)?;
    let id = ids::new_id();
    let now = now_ms();
    conn.execute(
        "INSERT INTO notes (id, body, created_at, updated_at, hlc, dirty) VALUES (?1, ?2, ?3, ?3, ?4, 1)",
        params![id, body, now, clock.now().to_string()],
    )?;
    get(conn, &id)
}

/// Returns a live (not deleted) note.
pub fn get(conn: &Connection, id: &str) -> Result<Note> {
    conn.query_row(
        &format!("SELECT {NOTE_COLS} FROM notes WHERE id = ?1 AND deleted_at IS NULL"),
        [id],
        note_from_row,
    )
    .optional()?
    .ok_or_else(|| Error::NotFound(id.to_owned()))
}

/// Applies a local change. `base_hlc` records the synced version the edit
/// started from, so sync can tell a concurrent edit from a fast-forward.
fn touch(conn: &Connection, clock: &Clock, id: &str, set_sql: &str, values: &[&dyn rusqlite::ToSql]) -> Result<()> {
    let sql = format!(
        "UPDATE notes SET {set_sql},
            base_hlc = CASE WHEN dirty = 0 THEN hlc ELSE base_hlc END,
            hlc = ?{h}, updated_at = ?{u}, dirty = 1
         WHERE id = ?{i}",
        h = values.len() + 1,
        u = values.len() + 2,
        i = values.len() + 3,
    );
    let hlc = clock.now().to_string();
    let now = now_ms();
    let mut all: Vec<&dyn rusqlite::ToSql> = values.to_vec();
    all.extend([&hlc as &dyn rusqlite::ToSql, &now, &id]);
    if conn.execute(&sql, all.as_slice())? == 0 {
        return Err(Error::NotFound(id.to_owned()));
    }
    Ok(())
}

pub fn update(conn: &Connection, clock: &Clock, id: &str, body: Option<&str>, pinned: Option<bool>) -> Result<Note> {
    let current = get(conn, id)?;
    let body = body.map(normalize).transpose()?.unwrap_or(current.body.clone());
    let pinned = pinned.unwrap_or(current.pinned);
    if body == current.body && pinned == current.pinned {
        return Ok(current);
    }
    touch(conn, clock, id, "body = ?1, pinned = ?2", &[&body, &pinned])?;
    get(conn, id)
}

/// Soft delete: the tombstone must sync to other devices.
pub fn delete(conn: &Connection, clock: &Clock, id: &str) -> Result<()> {
    get(conn, id)?;
    touch(conn, clock, id, "deleted_at = ?1", &[&now_ms()])
}

/// Undo of `delete`.
pub fn restore(conn: &Connection, clock: &Clock, id: &str) -> Result<Note> {
    touch(conn, clock, id, "deleted_at = NULL", &[])?;
    get(conn, id)
}

/// Live notes, pinned first, then most recently updated.
pub fn list(conn: &Connection, limit: u32) -> Result<Vec<Note>> {
    let mut stmt = conn.prepare_cached(&format!(
        "SELECT {NOTE_COLS} FROM notes WHERE deleted_at IS NULL
         ORDER BY pinned DESC, updated_at DESC LIMIT ?1"
    ))?;
    let notes = stmt.query_map([limit], note_from_row)?.collect::<rusqlite::Result<_>>()?;
    Ok(notes)
}

/// Accent-insensitive prefix search ("ghi chu" finds "ghi chú").
/// Pinned notes rank first, then BM25 relevance, then recency.
pub fn search(conn: &Connection, query: &str, limit: u32) -> Result<Vec<Note>> {
    let Some(q) = fts::prefix_query(query) else { return list(conn, limit) };
    let mut stmt = conn.prepare_cached(&format!(
        "SELECT {cols} FROM notes_fts f JOIN notes n ON n.seq = f.rowid
         WHERE notes_fts MATCH ?1 AND n.deleted_at IS NULL
         ORDER BY n.pinned DESC, bm25(notes_fts), n.updated_at DESC LIMIT ?2",
        cols = NOTE_COLS.split(", ").map(|c| format!("n.{c}")).collect::<Vec<_>>().join(", ")
    ))?;
    let notes = stmt.query_map(params![q, limit], note_from_row)?.collect::<rusqlite::Result<_>>()?;
    Ok(notes)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::NotesModule;
    use qd_core::Db;

    pub fn setup() -> (Db, Clock) {
        (Db::open_in_memory(&[&NotesModule]).unwrap(), Clock::new("dev-a"))
    }

    fn sync_cols(conn: &Connection, id: &str) -> (String, Option<String>, bool) {
        conn.query_row("SELECT hlc, base_hlc, dirty FROM notes WHERE id = ?1", [id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })
        .unwrap()
    }

    #[test]
    fn create_trims_and_rejects_empty() {
        let (db, clock) = setup();
        let conn = db.conn().unwrap();
        let n = create(&conn, &clock, "  check Pub/Sub retry after deploy \n").unwrap();
        assert_eq!(n.body, "check Pub/Sub retry after deploy");
        assert!(!n.pinned);
        assert!(matches!(create(&conn, &clock, "   "), Err(Error::EmptyBody)));
    }

    #[test]
    fn update_bumps_hlc_and_keeps_base_while_dirty() {
        let (db, clock) = setup();
        let conn = db.conn().unwrap();
        let n = create(&conn, &clock, "a").unwrap();
        let (h0, base0, dirty0) = sync_cols(&conn, &n.id);
        assert_eq!((base0, dirty0), (None, true));

        // Simulate a completed push.
        conn.execute("UPDATE notes SET dirty = 0", []).unwrap();
        update(&conn, &clock, &n.id, Some("b"), None).unwrap();
        let (h1, base1, _) = sync_cols(&conn, &n.id);
        assert!(h1 > h0);
        assert_eq!(base1.as_deref(), Some(h0.as_str()), "base = last synced version");

        update(&conn, &clock, &n.id, Some("c"), Some(true)).unwrap();
        let (h2, base2, _) = sync_cols(&conn, &n.id);
        assert!(h2 > h1);
        assert_eq!(base2.as_deref(), Some(h0.as_str()), "still based on the synced version");
    }

    #[test]
    fn noop_update_does_not_bump() {
        let (db, clock) = setup();
        let conn = db.conn().unwrap();
        let n = create(&conn, &clock, "same").unwrap();
        let (h0, ..) = sync_cols(&conn, &n.id);
        update(&conn, &clock, &n.id, Some("same "), Some(false)).unwrap();
        assert_eq!(sync_cols(&conn, &n.id).0, h0);
    }

    #[test]
    fn delete_hides_and_restore_brings_back() {
        let (db, clock) = setup();
        let conn = db.conn().unwrap();
        let n = create(&conn, &clock, "temp").unwrap();
        delete(&conn, &clock, &n.id).unwrap();
        assert!(list(&conn, 10).unwrap().is_empty());
        assert!(search(&conn, "temp", 10).unwrap().is_empty());
        assert!(matches!(get(&conn, &n.id), Err(Error::NotFound(_))));
        // Tombstone row is kept for sync.
        assert!(sync_cols(&conn, &n.id).2);

        assert_eq!(restore(&conn, &clock, &n.id).unwrap().body, "temp");
        assert_eq!(list(&conn, 10).unwrap().len(), 1);
    }

    #[test]
    fn list_orders_pinned_then_recent() {
        let (db, clock) = setup();
        let conn = db.conn().unwrap();
        let a = create(&conn, &clock, "a").unwrap();
        let b = create(&conn, &clock, "b").unwrap();
        let c = create(&conn, &clock, "c").unwrap();
        conn.execute("UPDATE notes SET updated_at = CASE id WHEN ?1 THEN 1 WHEN ?2 THEN 2 ELSE 3 END", [&a.id, &b.id])
            .unwrap();
        update(&conn, &clock, &a.id, None, Some(true)).unwrap();
        conn.execute("UPDATE notes SET updated_at = 0 WHERE id = ?1", [&a.id]).unwrap();
        let ids: Vec<String> = list(&conn, 10).unwrap().into_iter().map(|n| n.id).collect();
        assert_eq!(ids, vec![a.id, c.id, b.id]);
    }

    #[test]
    fn search_is_prefix_and_accent_insensitive() {
        let (db, clock) = setup();
        let conn = db.conn().unwrap();
        create(&conn, &clock, "Ghi chú: check Pub/Sub retry after deploy").unwrap();
        create(&conn, &clock, "kubectl get pods -A").unwrap();
        create(&conn, &clock, "Việt Nam vô địch").unwrap();

        let bodies = |q: &str| -> Vec<String> { search(&conn, q, 10).unwrap().into_iter().map(|n| n.body).collect() };
        assert_eq!(bodies("ghi chu"), vec!["Ghi chú: check Pub/Sub retry after deploy"]);
        assert_eq!(bodies("viet"), vec!["Việt Nam vô địch"]);
        assert_eq!(bodies("VIỆT nam"), vec!["Việt Nam vô địch"]);
        assert_eq!(bodies("kube po"), vec!["kubectl get pods -A"]);
        assert_eq!(bodies("pub/sub"), vec!["Ghi chú: check Pub/Sub retry after deploy"]);
        assert!(bodies("nothing").is_empty());
        assert_eq!(bodies("  ").len(), 3, "blank query lists everything");
    }

    #[test]
    fn search_reflects_edits() {
        let (db, clock) = setup();
        let conn = db.conn().unwrap();
        let n = create(&conn, &clock, "old words").unwrap();
        update(&conn, &clock, &n.id, Some("fresh text"), None).unwrap();
        assert!(search(&conn, "old", 10).unwrap().is_empty());
        assert_eq!(search(&conn, "fresh", 10).unwrap().len(), 1);
    }
}
