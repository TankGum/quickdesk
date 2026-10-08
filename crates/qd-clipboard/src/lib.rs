//! Clipboard history: dedup, retention and substring search. Local-only.

use qd_core::{Migration, Module};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("clipboard entry {0} not found")]
    NotFound(i64),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct ClipboardModule;

impl Module for ClipboardModule {
    fn id(&self) -> &'static str {
        "clipboard"
    }

    fn migrations(&self) -> &'static [Migration] {
        &[Migration { version: 1, sql: include_str!("migrations/001_clipboard.sql") }]
    }
}

/// Retention and size limits.
#[derive(Debug, Clone, Copy)]
pub struct Policy {
    pub max_entries: u32,
    pub max_age_ms: i64,
    pub max_bytes: usize,
    /// Re-copies of the newest entry within this window are not counted again.
    pub burst_ms: i64,
}

impl Default for Policy {
    fn default() -> Self {
        Policy { max_entries: 1000, max_age_ms: 30 * 24 * 3600 * 1000, max_bytes: 1 << 20, burst_ms: 1500 }
    }
}

/// An entry as listed in the UI: a preview, not the (possibly huge) content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipEntry {
    pub id: i64,
    pub preview: String,
    /// Length of the full content in characters.
    pub chars: i64,
    pub pinned: bool,
    pub source_app: Option<String>,
    pub first_copied_at: i64,
    pub last_copied_at: i64,
    pub copy_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ingested {
    Inserted(ClipEntry),
    /// Copied again: moved to the top.
    Bumped(ClipEntry),
    Skipped(SkipReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    Empty,
    TooLarge,
    /// Same content as the newest entry, within the burst window.
    Repeat,
}

/// Previews are the first 400 characters.
const ENTRY_COLS: &str = "id, substr(content, 1, 400), length(content), pinned, source_app,
                          first_copied_at, last_copied_at, copy_count";

fn entry_from_row(r: &Row<'_>) -> rusqlite::Result<ClipEntry> {
    Ok(ClipEntry {
        id: r.get(0)?,
        preview: r.get(1)?,
        chars: r.get(2)?,
        pinned: r.get(3)?,
        source_app: r.get(4)?,
        first_copied_at: r.get(5)?,
        last_copied_at: r.get(6)?,
        copy_count: r.get(7)?,
    })
}

fn hash(text: &str) -> Vec<u8> {
    blake3::hash(text.as_bytes()).as_bytes().to_vec()
}

pub fn get(conn: &Connection, id: i64) -> Result<ClipEntry> {
    conn.query_row(&format!("SELECT {ENTRY_COLS} FROM clip_entries WHERE id = ?1"), [id], entry_from_row)
        .optional()?
        .ok_or(Error::NotFound(id))
}

/// Full text of an entry.
pub fn content(conn: &Connection, id: i64) -> Result<String> {
    conn.query_row("SELECT content FROM clip_entries WHERE id = ?1", [id], |r| r.get(0))
        .optional()?
        .ok_or(Error::NotFound(id))
}

/// Record a copy event.
pub fn ingest(conn: &Connection, text: &str, source_app: Option<&str>, now: i64, policy: &Policy) -> Result<Ingested> {
    if text.trim().is_empty() {
        return Ok(Ingested::Skipped(SkipReason::Empty));
    }
    if text.len() > policy.max_bytes {
        return Ok(Ingested::Skipped(SkipReason::TooLarge));
    }
    let h = hash(text);
    let existing: Option<(i64, i64)> = conn
        .query_row("SELECT id, last_copied_at FROM clip_entries WHERE content_hash = ?1", [&h], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .optional()?;
    match existing {
        Some((id, last)) => {
            let newest: i64 = conn.query_row("SELECT MAX(last_copied_at) FROM clip_entries", [], |r| r.get(0))?;
            if last == newest && now - last < policy.burst_ms {
                return Ok(Ingested::Skipped(SkipReason::Repeat));
            }
            conn.execute(
                "UPDATE clip_entries SET last_copied_at = ?2, copy_count = copy_count + 1 WHERE id = ?1",
                params![id, now],
            )?;
            Ok(Ingested::Bumped(get(conn, id)?))
        }
        None => {
            conn.execute(
                "INSERT INTO clip_entries (content, content_hash, source_app, first_copied_at, last_copied_at)
                 VALUES (?1, ?2, ?3, ?4, ?4)",
                params![text, h, source_app, now],
            )?;
            Ok(Ingested::Inserted(get(conn, conn.last_insert_rowid())?))
        }
    }
}

/// Mark an entry as just used (picked from history), moving it to the top.
pub fn touch(conn: &Connection, id: i64, now: i64) -> Result<()> {
    if conn.execute("UPDATE clip_entries SET last_copied_at = ?2 WHERE id = ?1", params![id, now])? == 0 {
        return Err(Error::NotFound(id));
    }
    Ok(())
}

pub fn list(conn: &Connection, limit: u32) -> Result<Vec<ClipEntry>> {
    let mut stmt = conn.prepare_cached(&format!(
        "SELECT {ENTRY_COLS} FROM clip_entries ORDER BY pinned DESC, last_copied_at DESC LIMIT ?1"
    ))?;
    let rows = stmt.query_map([limit], entry_from_row)?.collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

/// Case- and accent-insensitive substring search.
pub fn search(conn: &Connection, query: &str, limit: u32) -> Result<Vec<ClipEntry>> {
    let q = query.trim();
    if q.is_empty() {
        return list(conn, limit);
    }
    let rows = if q.chars().count() >= 3 {
        // A quoted FTS5 string is a literal phrase; double any quotes inside.
        let phrase = format!("\"{}\"", q.replace('"', "\"\""));
        let mut stmt = conn.prepare_cached(&format!(
            "SELECT {ENTRY_COLS} FROM clip_entries
             WHERE id IN (SELECT rowid FROM clip_fts WHERE clip_fts MATCH ?1)
             ORDER BY pinned DESC, last_copied_at DESC LIMIT ?2"
        ))?;
        let rows = stmt.query_map(params![phrase, limit], entry_from_row)?.collect::<rusqlite::Result<_>>()?;
        rows
    } else {
        // Trigrams need 3 characters; scan recent entries instead.
        let pattern = format!("%{}%", q.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_"));
        let mut stmt = conn.prepare_cached(&format!(
            "SELECT {ENTRY_COLS} FROM (SELECT * FROM clip_entries ORDER BY last_copied_at DESC LIMIT 500)
             WHERE content LIKE ?1 ESCAPE '\\' ORDER BY pinned DESC, last_copied_at DESC LIMIT ?2"
        ))?;
        let rows = stmt.query_map(params![pattern, limit], entry_from_row)?.collect::<rusqlite::Result<_>>()?;
        rows
    };
    Ok(rows)
}

pub fn set_pinned(conn: &Connection, id: i64, pinned: bool) -> Result<ClipEntry> {
    if conn.execute("UPDATE clip_entries SET pinned = ?2 WHERE id = ?1", params![id, pinned])? == 0 {
        return Err(Error::NotFound(id));
    }
    get(conn, id)
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM clip_entries WHERE id = ?1", [id])?;
    Ok(())
}

/// Delete history; returns how many entries were removed.
pub fn clear(conn: &Connection, keep_pinned: bool) -> Result<usize> {
    let sql = if keep_pinned { "DELETE FROM clip_entries WHERE pinned = 0" } else { "DELETE FROM clip_entries" };
    Ok(conn.execute(sql, [])?)
}

/// Apply retention: drop unpinned entries that are too old or beyond the cap.
pub fn prune(conn: &Connection, policy: &Policy, now: i64) -> Result<usize> {
    let old = conn.execute(
        "DELETE FROM clip_entries WHERE pinned = 0 AND last_copied_at < ?1",
        [now - policy.max_age_ms],
    )?;
    let excess = conn.execute(
        "DELETE FROM clip_entries WHERE pinned = 0 AND id NOT IN (
           SELECT id FROM clip_entries WHERE pinned = 0 ORDER BY last_copied_at DESC LIMIT ?1)",
        [policy.max_entries],
    )?;
    Ok(old + excess)
}

#[cfg(test)]
mod tests {
    use super::*;
    use qd_core::Db;

    fn db() -> Db {
        Db::open_in_memory(&[&ClipboardModule]).unwrap()
    }

    fn previews(entries: Vec<ClipEntry>) -> Vec<String> {
        entries.into_iter().map(|e| e.preview).collect()
    }

    const P: Policy = Policy { max_entries: 1000, max_age_ms: 30 * 24 * 3600 * 1000, max_bytes: 1 << 20, burst_ms: 1500 };

    #[test]
    fn inserts_and_lists_newest_first() {
        let db = db();
        let c = db.conn().unwrap();
        ingest(&c, "git status", None, 1_000, &P).unwrap();
        ingest(&c, "kubectl get pods -A", None, 2_000, &P).unwrap();
        assert_eq!(previews(list(&c, 10).unwrap()), vec!["kubectl get pods -A", "git status"]);
    }

    #[test]
    fn recopy_bumps_but_bursts_are_ignored() {
        let db = db();
        let c = db.conn().unwrap();
        ingest(&c, "a", None, 1_000, &P).unwrap();
        assert_eq!(ingest(&c, "a", None, 1_200, &P).unwrap(), Ingested::Skipped(SkipReason::Repeat));
        ingest(&c, "b", None, 2_000, &P).unwrap();
        let Ingested::Bumped(e) = ingest(&c, "a", None, 2_500, &P).unwrap() else { panic!("expected bump") };
        assert_eq!((e.copy_count, e.last_copied_at, e.first_copied_at), (2, 2_500, 1_000));
        assert_eq!(previews(list(&c, 10).unwrap()), vec!["a", "b"]);
        assert_eq!(list(&c, 10).unwrap().len(), 2, "deduplicated by content");
    }

    #[test]
    fn skips_empty_and_oversized() {
        let db = db();
        let c = db.conn().unwrap();
        let small = Policy { max_bytes: 10, ..P };
        assert_eq!(ingest(&c, " \n\t", None, 1, &small).unwrap(), Ingested::Skipped(SkipReason::Empty));
        assert_eq!(ingest(&c, "01234567890", None, 1, &small).unwrap(), Ingested::Skipped(SkipReason::TooLarge));
        assert!(list(&c, 10).unwrap().is_empty());
    }

    #[test]
    fn preview_is_truncated_but_content_is_complete() {
        let db = db();
        let c = db.conn().unwrap();
        let long = "é".repeat(1000);
        let Ingested::Inserted(e) = ingest(&c, &long, None, 1, &P).unwrap() else { panic!() };
        assert_eq!(e.preview.chars().count(), 400);
        assert_eq!(e.chars, 1000);
        assert_eq!(content(&c, e.id).unwrap(), long);
    }

    #[test]
    fn substring_search_with_trigrams_and_short_queries() {
        let db = db();
        let c = db.conn().unwrap();
        ingest(&c, "http://localhost:8000/docs", None, 1, &P).unwrap();
        ingest(&c, "kubectl get pods -A", None, 2, &P).unwrap();
        ingest(&c, "Tiếng Việt có dấu", None, 3, &P).unwrap();
        ingest(&c, "50% off \"quoted\"", None, 4, &P).unwrap();

        assert_eq!(previews(search(&c, "8000", 10).unwrap()), vec!["http://localhost:8000/docs"]);
        assert_eq!(previews(search(&c, "PODS", 10).unwrap()), vec!["kubectl get pods -A"]);
        assert_eq!(previews(search(&c, "tieng viet", 10).unwrap()), vec!["Tiếng Việt có dấu"]);
        assert_eq!(previews(search(&c, "\"quoted\"", 10).unwrap()), vec!["50% off \"quoted\""]);
        // Short queries: LIKE fallback, with wildcards escaped.
        assert_eq!(previews(search(&c, "-A", 10).unwrap()), vec!["kubectl get pods -A"]);
        assert_eq!(previews(search(&c, "%", 10).unwrap()), vec!["50% off \"quoted\""]);
        assert!(search(&c, "zzz", 10).unwrap().is_empty());
    }

    #[test]
    fn prune_respects_age_cap_and_pins() {
        let db = db();
        let c = db.conn().unwrap();
        let policy = Policy { max_entries: 2, max_age_ms: 1_000, ..P };
        for (i, t) in ["old", "pinned-old", "a", "b", "c"].iter().enumerate() {
            ingest(&c, t, None, if i < 2 { 0 } else { 5_000 + i as i64 }, &policy).unwrap();
        }
        let pinned = search(&c, "pinned-old", 1).unwrap()[0].id;
        set_pinned(&c, pinned, true).unwrap();

        let removed = prune(&c, &policy, 5_500).unwrap();
        assert_eq!(removed, 2, "'old' by age, 'a' by cap");
        assert_eq!(previews(list(&c, 10).unwrap()), vec!["pinned-old", "c", "b"]);
    }

    #[test]
    fn touch_pin_delete_clear() {
        let db = db();
        let c = db.conn().unwrap();
        ingest(&c, "x", None, 1, &P).unwrap();
        ingest(&c, "y", None, 2, &P).unwrap();
        let x = search(&c, "x", 1).unwrap()[0].id;
        touch(&c, x, 10).unwrap();
        assert_eq!(previews(list(&c, 10).unwrap()), vec!["x", "y"]);
        set_pinned(&c, x, true).unwrap();
        assert_eq!(clear(&c, true).unwrap(), 1);
        assert_eq!(previews(list(&c, 10).unwrap()), vec!["x"]);
        delete(&c, x).unwrap();
        assert!(list(&c, 10).unwrap().is_empty());
        assert!(matches!(touch(&c, x, 1), Err(Error::NotFound(_))));
    }
}
