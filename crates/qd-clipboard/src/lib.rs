//! Clipboard history: text, images and copied files. Dedup, retention and
//! substring search. Local-only.
//!
//! Image bytes are stored as files in a "blob" directory next to the
//! database; every function that can remove an image entry also removes its
//! file. Copied files are stored as paths only, never their contents.

use std::fs;
use std::path::{Path, PathBuf};

use data_encoding::{BASE64, HEXLOWER};
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
    #[error("file error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct ClipboardModule;

impl Module for ClipboardModule {
    fn id(&self) -> &'static str {
        "clipboard"
    }

    fn migrations(&self) -> &'static [Migration] {
        &[
            Migration { version: 1, sql: include_str!("migrations/001_clipboard.sql") },
            Migration { version: 2, sql: include_str!("migrations/002_media.sql") },
        ]
    }
}

/// Retention and size limits.
#[derive(Debug, Clone, Copy)]
pub struct Policy {
    pub max_entries: u32,
    pub max_age_ms: i64,
    /// Largest text entry.
    pub max_bytes: usize,
    /// Largest image entry (encoded).
    pub max_image_bytes: usize,
    /// All unpinned images together; the oldest go first.
    pub max_image_total: u64,
    /// Re-copies of the newest entry within this window are not counted again.
    pub burst_ms: i64,
}

impl Default for Policy {
    fn default() -> Self {
        Policy {
            max_entries: 1000,
            max_age_ms: 30 * 24 * 3600 * 1000,
            max_bytes: 1 << 20,
            max_image_bytes: 10 << 20,
            max_image_total: 200 << 20,
            burst_ms: 1500,
        }
    }
}

/// An entry as listed in the UI: a preview, not the (possibly huge) content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipEntry {
    pub id: i64,
    /// `text`, `image` or `files`.
    pub kind: String,
    /// Text: first 400 characters. Files: the paths, one per line.
    pub preview: String,
    /// Length of the full text in characters.
    pub chars: i64,
    pub pinned: bool,
    pub source_app: Option<String>,
    pub first_copied_at: i64,
    pub last_copied_at: i64,
    pub copy_count: i64,
    pub byte_size: i64,
    pub width: Option<i64>,
    pub height: Option<i64>,
    /// `data:image/png;base64,…` thumbnail for images.
    pub thumb: Option<String>,
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
    /// Not an image we can decode.
    BadImage,
}

/// An image decoded and thumbnailed ahead of time, so the database lock is
/// not held during the (slow) decoding.
#[derive(Debug, Clone)]
pub struct PreparedImage {
    hash: Vec<u8>,
    mime: String,
    ext: &'static str,
    bytes: Vec<u8>,
    width: u32,
    height: u32,
    thumb_png: Vec<u8>,
}

const THUMB_MAX: u32 = 320;

pub fn prepare_image(mime: &str, bytes: Vec<u8>, policy: &Policy) -> std::result::Result<PreparedImage, SkipReason> {
    if bytes.is_empty() {
        return Err(SkipReason::Empty);
    }
    if bytes.len() > policy.max_image_bytes {
        return Err(SkipReason::TooLarge);
    }
    let img = image::load_from_memory(&bytes).map_err(|_| SkipReason::BadImage)?;
    let (width, height) = (img.width(), img.height());
    let mut thumb_png = Vec::new();
    img.thumbnail(THUMB_MAX, THUMB_MAX)
        .write_to(&mut std::io::Cursor::new(&mut thumb_png), image::ImageFormat::Png)
        .map_err(|_| SkipReason::BadImage)?;
    let ext = if mime == "image/jpeg" { "jpg" } else { "png" };
    Ok(PreparedImage { hash: hash(&bytes), mime: mime.to_owned(), ext, bytes, width, height, thumb_png })
}

/// What was copied.
#[derive(Debug, Clone, Copy)]
pub enum Captured<'a> {
    Text(&'a str),
    Image(&'a PreparedImage),
    Files(&'a [String]),
}

/// What to put back on the clipboard for an entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    Text(String),
    Image { path: PathBuf, mime: String },
    Files(Vec<String>),
}

const ENTRY_COLS: &str = "id, kind, CASE kind WHEN 'files' THEN content ELSE substr(content, 1, 400) END,
                          length(content), pinned, source_app, first_copied_at, last_copied_at, copy_count,
                          byte_size, width, height, thumb";

fn entry_from_row(r: &Row<'_>) -> rusqlite::Result<ClipEntry> {
    let thumb: Option<Vec<u8>> = r.get(12)?;
    Ok(ClipEntry {
        id: r.get(0)?,
        kind: r.get(1)?,
        preview: r.get(2)?,
        chars: r.get(3)?,
        pinned: r.get(4)?,
        source_app: r.get(5)?,
        first_copied_at: r.get(6)?,
        last_copied_at: r.get(7)?,
        copy_count: r.get(8)?,
        byte_size: r.get(9)?,
        width: r.get(10)?,
        height: r.get(11)?,
        thumb: thumb.map(|t| format!("data:image/png;base64,{}", BASE64.encode(&t))),
    })
}

fn hash(bytes: &[u8]) -> Vec<u8> {
    blake3::hash(bytes).as_bytes().to_vec()
}

pub fn get(conn: &Connection, id: i64) -> Result<ClipEntry> {
    conn.query_row(&format!("SELECT {ENTRY_COLS} FROM clip_entries WHERE id = ?1"), [id], entry_from_row)
        .optional()?
        .ok_or(Error::NotFound(id))
}

/// Full text of a text entry (or the newline-joined paths of a files entry).
pub fn content(conn: &Connection, id: i64) -> Result<String> {
    conn.query_row("SELECT content FROM clip_entries WHERE id = ?1", [id], |r| r.get(0))
        .optional()?
        .ok_or(Error::NotFound(id))
}

/// What to write back to the clipboard for `id`.
pub fn payload(conn: &Connection, blobs: &Path, id: i64) -> Result<Payload> {
    let (kind, content, blob, mime): (String, String, Option<String>, Option<String>) = conn
        .query_row("SELECT kind, content, blob, mime FROM clip_entries WHERE id = ?1", [id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })
        .optional()?
        .ok_or(Error::NotFound(id))?;
    Ok(match (kind.as_str(), blob) {
        ("image", Some(b)) => Payload::Image { path: blobs.join(b), mime: mime.unwrap_or_else(|| "image/png".into()) },
        ("files", _) => Payload::Files(content.lines().map(str::to_owned).collect()),
        _ => Payload::Text(content),
    })
}

/// Record a copied text (kept for callers that only deal with text).
pub fn ingest(conn: &Connection, text: &str, source_app: Option<&str>, now: i64, policy: &Policy) -> Result<Ingested> {
    ingest_capture(conn, Path::new(""), Captured::Text(text), source_app, now, policy)
}

/// Record a copy event of any kind.
pub fn ingest_capture(
    conn: &Connection,
    blobs: &Path,
    captured: Captured<'_>,
    source_app: Option<&str>,
    now: i64,
    policy: &Policy,
) -> Result<Ingested> {
    let (kind, content, h, size) = match captured {
        Captured::Text(text) => {
            if text.trim().is_empty() {
                return Ok(Ingested::Skipped(SkipReason::Empty));
            }
            if text.len() > policy.max_bytes {
                return Ok(Ingested::Skipped(SkipReason::TooLarge));
            }
            ("text", text.to_owned(), hash(text.as_bytes()), text.len())
        }
        Captured::Files(paths) => {
            if paths.is_empty() {
                return Ok(Ingested::Skipped(SkipReason::Empty));
            }
            let joined = paths.join("\n");
            ("files", joined.clone(), hash(format!("files\n{joined}").as_bytes()), joined.len())
        }
        Captured::Image(img) => {
            // Searchable description; the UI shows the thumbnail instead.
            let desc = format!("image ảnh {} {}x{}", img.ext, img.width, img.height);
            ("image", desc, img.hash.clone(), img.bytes.len())
        }
    };

    let existing: Option<(i64, i64)> = conn
        .query_row("SELECT id, last_copied_at FROM clip_entries WHERE content_hash = ?1", [&h], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .optional()?;
    if let Some((id, last)) = existing {
        let newest: i64 = conn.query_row("SELECT MAX(last_copied_at) FROM clip_entries", [], |r| r.get(0))?;
        if last == newest && now - last < policy.burst_ms {
            return Ok(Ingested::Skipped(SkipReason::Repeat));
        }
        conn.execute(
            "UPDATE clip_entries SET last_copied_at = ?2, copy_count = copy_count + 1 WHERE id = ?1",
            params![id, now],
        )?;
        return Ok(Ingested::Bumped(get(conn, id)?));
    }

    let (mime, blob, width, height, thumb) = match captured {
        Captured::Image(img) => {
            let name = format!("{}.{}", HEXLOWER.encode(&img.hash[..16]), img.ext);
            fs::create_dir_all(blobs)?;
            let path = blobs.join(&name);
            if !path.exists() {
                fs::write(&path, &img.bytes)?;
            }
            (
                Some(img.mime.clone()),
                Some(name),
                Some(img.width as i64),
                Some(img.height as i64),
                Some(img.thumb_png.clone()),
            )
        }
        _ => (None, None, None, None, None),
    };
    conn.execute(
        "INSERT INTO clip_entries
           (content, content_hash, kind, source_app, first_copied_at, last_copied_at, mime, blob, byte_size, width, height, thumb)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![content, h, kind, source_app, now, mime, blob, size as i64, width, height, thumb],
    )?;
    Ok(Ingested::Inserted(get(conn, conn.last_insert_rowid())?))
}

/// Mark an entry as just used (picked from history), moving it to the top.
pub fn touch(conn: &Connection, id: i64, now: i64) -> Result<()> {
    if conn.execute("UPDATE clip_entries SET last_copied_at = ?2 WHERE id = ?1", params![id, now])? == 0 {
        return Err(Error::NotFound(id));
    }
    Ok(())
}

/// Newest first, optionally only one `kind` (`text`, `image`, `files`).
pub fn list(conn: &Connection, kind: Option<&str>, limit: u32) -> Result<Vec<ClipEntry>> {
    let mut stmt = conn.prepare_cached(&format!(
        "SELECT {ENTRY_COLS} FROM clip_entries WHERE (?1 IS NULL OR kind = ?1)
         ORDER BY pinned DESC, last_copied_at DESC LIMIT ?2"
    ))?;
    let rows = stmt.query_map(params![kind, limit], entry_from_row)?.collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

/// Case- and accent-insensitive substring search (file paths included).
pub fn search(conn: &Connection, query: &str, kind: Option<&str>, limit: u32) -> Result<Vec<ClipEntry>> {
    let q = query.trim();
    if q.is_empty() {
        return list(conn, kind, limit);
    }
    let rows = if q.chars().count() >= 3 {
        // A quoted FTS5 string is a literal phrase; double any quotes inside.
        let phrase = format!("\"{}\"", q.replace('"', "\"\""));
        let mut stmt = conn.prepare_cached(&format!(
            "SELECT {ENTRY_COLS} FROM clip_entries
             WHERE id IN (SELECT rowid FROM clip_fts WHERE clip_fts MATCH ?1) AND (?2 IS NULL OR kind = ?2)
             ORDER BY pinned DESC, last_copied_at DESC LIMIT ?3"
        ))?;
        let rows = stmt.query_map(params![phrase, kind, limit], entry_from_row)?.collect::<rusqlite::Result<_>>()?;
        rows
    } else {
        // Trigrams need 3 characters; scan recent entries instead.
        let pattern = format!("%{}%", q.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_"));
        let mut stmt = conn.prepare_cached(&format!(
            "SELECT {ENTRY_COLS} FROM (SELECT * FROM clip_entries ORDER BY last_copied_at DESC LIMIT 500)
             WHERE content LIKE ?1 ESCAPE '\\' AND (?2 IS NULL OR kind = ?2)
             ORDER BY pinned DESC, last_copied_at DESC LIMIT ?3"
        ))?;
        let rows = stmt.query_map(params![pattern, kind, limit], entry_from_row)?.collect::<rusqlite::Result<_>>()?;
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

/// Delete rows matching `filter` and their image files. Returns rows removed.
fn delete_where(conn: &Connection, blobs: &Path, filter: &str, args: &[&dyn rusqlite::ToSql]) -> Result<usize> {
    let mut stmt = conn.prepare(&format!("SELECT blob FROM clip_entries WHERE blob IS NOT NULL AND ({filter})"))?;
    let doomed: Vec<String> = stmt.query_map(args, |r| r.get(0))?.collect::<rusqlite::Result<_>>()?;
    let n = conn.execute(&format!("DELETE FROM clip_entries WHERE {filter}"), args)?;
    for b in doomed {
        let _ = fs::remove_file(blobs.join(b));
    }
    Ok(n)
}

pub fn delete(conn: &Connection, blobs: &Path, id: i64) -> Result<()> {
    delete_where(conn, blobs, "id = ?1", &[&id])?;
    Ok(())
}

/// Delete history; returns how many entries were removed.
pub fn clear(conn: &Connection, blobs: &Path, keep_pinned: bool) -> Result<usize> {
    delete_where(conn, blobs, if keep_pinned { "pinned = 0" } else { "1 = 1" }, &[])
}

/// Apply retention: drop unpinned entries that are too old or beyond the
/// caps (entry count, total image bytes), and image files nothing refers to.
pub fn prune(conn: &Connection, blobs: &Path, policy: &Policy, now: i64) -> Result<usize> {
    let cutoff = now - policy.max_age_ms;
    let mut removed = delete_where(conn, blobs, "pinned = 0 AND last_copied_at < ?1", &[&cutoff])?;
    removed += delete_where(
        conn,
        blobs,
        "pinned = 0 AND id NOT IN (SELECT id FROM clip_entries WHERE pinned = 0 ORDER BY last_copied_at DESC LIMIT ?1)",
        &[&policy.max_entries],
    )?;
    // Image budget: keep the newest unpinned images that fit.
    let images: Vec<(i64, i64)> = {
        let mut stmt = conn.prepare(
            "SELECT id, byte_size FROM clip_entries WHERE kind = 'image' AND pinned = 0 ORDER BY last_copied_at DESC",
        )?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<rusqlite::Result<_>>()?;
        rows
    };
    let mut total: u64 = 0;
    for (id, size) in images {
        total += size.max(0) as u64;
        if total > policy.max_image_total {
            removed += delete_where(conn, blobs, "id = ?1", &[&id])?;
        }
    }
    remove_orphans(conn, blobs)?;
    Ok(removed)
}

/// Image files left behind (e.g. a crash between writing and inserting).
fn remove_orphans(conn: &Connection, blobs: &Path) -> Result<()> {
    let Ok(dir) = fs::read_dir(blobs) else { return Ok(()) };
    let mut stmt = conn.prepare_cached("SELECT 1 FROM clip_entries WHERE blob = ?1")?;
    for entry in dir.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !stmt.exists([&name])? {
            let _ = fs::remove_file(entry.path());
        }
    }
    Ok(())
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

    const P: Policy = Policy {
        max_entries: 1000,
        max_age_ms: 30 * 24 * 3600 * 1000,
        max_bytes: 1 << 20,
        max_image_bytes: 10 << 20,
        max_image_total: 200 << 20,
        burst_ms: 1500,
    };

    fn blob_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("qd-clip-{tag}-{}", qd_core::ids::new_id()));
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn png(w: u32, h: u32, seed: u8) -> Vec<u8> {
        let img = image::RgbaImage::from_fn(w, h, |x, y| image::Rgba([seed, x as u8, y as u8, 255]));
        let mut out = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .unwrap();
        out
    }

    #[test]
    fn inserts_and_lists_newest_first() {
        let db = db();
        let c = db.conn().unwrap();
        ingest(&c, "git status", None, 1_000, &P).unwrap();
        ingest(&c, "kubectl get pods -A", None, 2_000, &P).unwrap();
        assert_eq!(previews(list(&c, None, 10).unwrap()), vec!["kubectl get pods -A", "git status"]);
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
        assert_eq!(previews(list(&c, None, 10).unwrap()), vec!["a", "b"]);
    }

    #[test]
    fn skips_empty_and_oversized() {
        let db = db();
        let c = db.conn().unwrap();
        let small = Policy { max_bytes: 10, ..P };
        assert_eq!(ingest(&c, " \n\t", None, 1, &small).unwrap(), Ingested::Skipped(SkipReason::Empty));
        assert_eq!(ingest(&c, "01234567890", None, 1, &small).unwrap(), Ingested::Skipped(SkipReason::TooLarge));
        assert!(list(&c, None, 10).unwrap().is_empty());
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
        assert_eq!(e.kind, "text");
    }

    #[test]
    fn substring_search_with_trigrams_and_short_queries() {
        let db = db();
        let c = db.conn().unwrap();
        ingest(&c, "http://localhost:8000/docs", None, 1, &P).unwrap();
        ingest(&c, "kubectl get pods -A", None, 2, &P).unwrap();
        ingest(&c, "Tiếng Việt có dấu", None, 3, &P).unwrap();
        ingest(&c, "50% off \"quoted\"", None, 4, &P).unwrap();

        let s = |q: &str| previews(search(&c, q, None, 10).unwrap());
        assert_eq!(s("8000"), vec!["http://localhost:8000/docs"]);
        assert_eq!(s("PODS"), vec!["kubectl get pods -A"]);
        assert_eq!(s("tieng viet"), vec!["Tiếng Việt có dấu"]);
        assert_eq!(s("\"quoted\""), vec!["50% off \"quoted\""]);
        assert_eq!(s("-A"), vec!["kubectl get pods -A"]);
        assert_eq!(s("%"), vec!["50% off \"quoted\""]);
        assert!(s("zzz").is_empty());
    }

    #[test]
    fn images_are_stored_as_files_with_thumbnails_and_deduplicated() {
        let db = db();
        let c = db.conn().unwrap();
        let blobs = blob_dir("img");
        let bytes = png(800, 400, 1);
        let img = prepare_image("image/png", bytes.clone(), &P).unwrap();
        let Ingested::Inserted(e) = ingest_capture(&c, &blobs, Captured::Image(&img), None, 1_000, &P).unwrap() else {
            panic!("expected insert")
        };
        assert_eq!((e.kind.as_str(), e.width, e.height), ("image", Some(800), Some(400)));
        assert!(e.thumb.as_deref().unwrap().starts_with("data:image/png;base64,"));
        assert_eq!(e.byte_size as usize, bytes.len());

        let Payload::Image { path, mime } = payload(&c, &blobs, e.id).unwrap() else { panic!() };
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert_eq!(mime, "image/png");

        // Same pixels copied again later: one entry, bumped.
        let again = prepare_image("image/png", bytes, &P).unwrap();
        assert!(matches!(
            ingest_capture(&c, &blobs, Captured::Image(&again), None, 9_000, &P).unwrap(),
            Ingested::Bumped(_)
        ));
        assert_eq!(list(&c, Some("image"), 10).unwrap().len(), 1);

        delete(&c, &blobs, e.id).unwrap();
        assert!(!path.exists(), "deleting the entry removes its file");
        fs::remove_dir_all(blobs).unwrap();
    }

    #[test]
    fn rejects_bad_or_oversized_images() {
        assert_eq!(prepare_image("image/png", b"not a png".to_vec(), &P).unwrap_err(), SkipReason::BadImage);
        let tiny = Policy { max_image_bytes: 10, ..P };
        assert_eq!(prepare_image("image/png", png(4, 4, 0), &tiny).unwrap_err(), SkipReason::TooLarge);
        assert_eq!(prepare_image("image/png", Vec::new(), &P).unwrap_err(), SkipReason::Empty);
    }

    #[test]
    fn files_are_paths_searchable_by_name_and_filterable() {
        let db = db();
        let c = db.conn().unwrap();
        let blobs = blob_dir("files");
        let paths = vec!["/home/me/Báo cáo Q3.pdf".to_owned(), "/home/me/notes.txt".to_owned()];
        ingest_capture(&c, &blobs, Captured::Files(&paths), None, 1, &P).unwrap();
        ingest(&c, "just text", None, 2, &P).unwrap();

        let hits = search(&c, "bao cao", None, 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].kind, "files");
        assert_eq!(hits[0].preview, paths.join("\n"));
        assert_eq!(payload(&c, &blobs, hits[0].id).unwrap(), Payload::Files(paths.clone()));
        assert_eq!(list(&c, Some("files"), 10).unwrap().len(), 1);
        assert_eq!(list(&c, Some("text"), 10).unwrap().len(), 1);
        assert_eq!(search(&c, "notes", Some("text"), 10).unwrap().len(), 0);
        fs::remove_dir_all(blobs).unwrap();
    }

    #[test]
    fn prune_respects_age_cap_pins_and_image_budget() {
        let db = db();
        let c = db.conn().unwrap();
        let blobs = blob_dir("prune");
        let policy = Policy { max_entries: 2, max_age_ms: 1_000, ..P };
        for (i, t) in ["old", "pinned-old", "a", "b", "c"].iter().enumerate() {
            ingest(&c, t, None, if i < 2 { 0 } else { 5_000 + i as i64 }, &policy).unwrap();
        }
        let pinned = search(&c, "pinned-old", None, 1).unwrap()[0].id;
        set_pinned(&c, pinned, true).unwrap();
        assert_eq!(prune(&c, &blobs, &policy, 5_500).unwrap(), 2, "'old' by age, 'a' by cap");
        assert_eq!(previews(list(&c, None, 10).unwrap()), vec!["pinned-old", "c", "b"]);

        // Image budget: three images, room for two.
        let imgs: Vec<PreparedImage> =
            (0..3).map(|i| prepare_image("image/png", png(64, 64, i), &P).unwrap()).collect();
        let budget = Policy { max_entries: 100, max_image_total: (imgs[0].bytes.len() * 2 + 10) as u64, ..P };
        for (i, img) in imgs.iter().enumerate() {
            ingest_capture(&c, &blobs, Captured::Image(img), None, 10_000 + i as i64, &budget).unwrap();
        }
        fs::write(blobs.join("orphan.png"), b"x").unwrap();
        prune(&c, &blobs, &budget, 10_010).unwrap();
        assert_eq!(list(&c, Some("image"), 10).unwrap().len(), 2, "oldest image dropped");
        assert_eq!(fs::read_dir(&blobs).unwrap().count(), 2, "its file and the orphan are gone");
        fs::remove_dir_all(blobs).unwrap();
    }

    #[test]
    fn touch_pin_delete_clear() {
        let db = db();
        let c = db.conn().unwrap();
        let blobs = blob_dir("misc");
        ingest(&c, "x", None, 1, &P).unwrap();
        ingest(&c, "y", None, 2, &P).unwrap();
        let x = search(&c, "x", None, 1).unwrap()[0].id;
        touch(&c, x, 10).unwrap();
        assert_eq!(previews(list(&c, None, 10).unwrap()), vec!["x", "y"]);
        set_pinned(&c, x, true).unwrap();
        assert_eq!(clear(&c, &blobs, true).unwrap(), 1);
        assert_eq!(previews(list(&c, None, 10).unwrap()), vec!["x"]);
        delete(&c, &blobs, x).unwrap();
        assert!(list(&c, None, 10).unwrap().is_empty());
        assert!(matches!(touch(&c, x, 1), Err(Error::NotFound(_))));
        fs::remove_dir_all(blobs).unwrap();
    }
}
