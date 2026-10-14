//! Pictures in notes. A picture is shrunk when it is added, stored as a file
//! named after the hash of its bytes (`<hash>.png` or `<hash>.jpg`), and the
//! note's Markdown only says `![](qdimg:<hash>)`, so searching, the list
//! previews and the sync of text work as before. The same picture added twice
//! is one file.

use std::collections::HashSet;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use data_encoding::BASE64;
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, Limits, RgbImage};
use rusqlite::Connection;

use crate::{Error, Result};

/// What the Markdown of a note puts before the hash: `![](qdimg:<hash>)`.
pub const SCHEME: &str = "qdimg:";
/// Largest file accepted, before it is shrunk.
pub const MAX_INPUT_BYTES: usize = 20 * 1024 * 1024;
/// Largest picture kept (the sync server's limit for one object).
pub const MAX_STORED_BYTES: usize = 8 * 1024 * 1024;
/// Longest side of a stored picture, in pixels.
const MAX_SIDE: u32 = 1600;
/// Screenshots stay lossless PNG while they are this small; bigger ones (photos) become JPEG.
const KEEP_PNG_UP_TO: usize = 1024 * 1024;
const JPEG_QUALITY: u8 = 82;

/// A picture ready to be stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picture {
    /// Lower-case hex BLAKE3 of `bytes`: 64 characters.
    pub hash: String,
    pub mime: &'static str,
    pub bytes: Vec<u8>,
}

fn bad(msg: impl Into<String>) -> Error {
    Error::Picture(msg.into())
}

/// A valid hash: 64 lower-case hex digits (also what keeps a name from walking out of the folder).
pub fn is_hash(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Decode, turn upright, shrink to `MAX_SIDE`, and encode as PNG (small) or JPEG (big).
pub fn process(input: &[u8]) -> Result<Picture> {
    if input.is_empty() {
        return Err(bad("the file is empty"));
    }
    if input.len() > MAX_INPUT_BYTES {
        return Err(bad(format!("the file is over {} MB", MAX_INPUT_BYTES / 1024 / 1024)));
    }
    let mut reader = ImageReader::new(Cursor::new(input)).with_guessed_format().map_err(|e| bad(e.to_string()))?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(20_000);
    limits.max_image_height = Some(20_000);
    reader.limits(limits);
    let mut decoder = reader.into_decoder().map_err(|_| bad("not a picture QuickDesk can read"))?;
    let orientation = decoder.orientation().ok();
    let mut img = DynamicImage::from_decoder(decoder).map_err(|e| bad(e.to_string()))?;
    if let Some(o) = orientation {
        img.apply_orientation(o);
    }
    if img.width().max(img.height()) > MAX_SIDE {
        img = img.resize(MAX_SIDE, MAX_SIDE, FilterType::Lanczos3);
    }

    let mut png = Vec::new();
    img.write_to(&mut Cursor::new(&mut png), ImageFormat::Png).map_err(|e| bad(e.to_string()))?;
    let (bytes, mime) = if png.len() <= KEEP_PNG_UP_TO {
        (png, "image/png")
    } else {
        let mut jpeg = Vec::new();
        JpegEncoder::new_with_quality(&mut jpeg, JPEG_QUALITY)
            .encode_image(&flatten(&img))
            .map_err(|e| bad(e.to_string()))?;
        (jpeg, "image/jpeg")
    };
    if bytes.len() > MAX_STORED_BYTES {
        return Err(bad("the picture is still too big after shrinking"));
    }
    Ok(Picture { hash: blake3::hash(&bytes).to_hex().to_string(), mime, bytes })
}

/// JPEG has no transparency: lay the picture on white.
fn flatten(img: &DynamicImage) -> RgbImage {
    let rgba = img.to_rgba8();
    let mut out = RgbImage::new(rgba.width(), rgba.height());
    for (x, y, p) in rgba.enumerate_pixels() {
        let a = p[3] as u32;
        let mix = |c: u8| ((c as u32 * a + 255 * (255 - a)) / 255) as u8;
        out.put_pixel(x, y, image::Rgb([mix(p[0]), mix(p[1]), mix(p[2])]));
    }
    out
}

fn ext(mime: &str) -> &'static str {
    if mime == "image/jpeg" {
        "jpg"
    } else {
        "png"
    }
}

fn mime_of(ext: &str) -> &'static str {
    if ext == "jpg" {
        "image/jpeg"
    } else {
        "image/png"
    }
}

/// Write the picture into `dir` (nothing happens if it is already there).
pub fn store(dir: &Path, pic: &Picture) -> Result<PathBuf> {
    let path = dir.join(format!("{}.{}", pic.hash, ext(pic.mime)));
    if path.exists() {
        return Ok(path);
    }
    fs::create_dir_all(dir).map_err(|e| bad(e.to_string()))?;
    // Written under another name first, so a half-written file is never read.
    let tmp = dir.join(format!("{}.tmp", pic.hash));
    fs::write(&tmp, &pic.bytes).map_err(|e| bad(e.to_string()))?;
    fs::rename(&tmp, &path).map_err(|e| bad(e.to_string()))?;
    Ok(path)
}

/// The stored picture's bytes and type; `None` if it is not here (yet).
pub fn read(dir: &Path, hash: &str) -> Result<Option<(Vec<u8>, &'static str)>> {
    if !is_hash(hash) {
        return Err(bad("not a picture id"));
    }
    for e in ["png", "jpg"] {
        match fs::read(dir.join(format!("{hash}.{e}"))) {
            Ok(bytes) => return Ok(Some((bytes, mime_of(e)))),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(bad(err.to_string())),
        }
    }
    Ok(None)
}

/// Whether the picture is stored in `dir`.
pub fn exists(dir: &Path, hash: &str) -> bool {
    is_hash(hash) && ["png", "jpg"].iter().any(|e| dir.join(format!("{hash}.{e}")).is_file())
}

/// Store a picture another device sent. It is only accepted if it is a PNG or
/// JPEG small enough to have come from `process` and its bytes hash to `hash`.
pub fn store_received(dir: &Path, hash: &str, bytes: Vec<u8>) -> Result<()> {
    if !is_hash(hash) || bytes.is_empty() || bytes.len() > MAX_STORED_BYTES {
        return Err(bad("not a picture"));
    }
    let mime = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        "image/png"
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg"
    } else {
        return Err(bad("not a PNG or JPEG"));
    };
    if blake3::hash(&bytes).to_hex().as_str() != hash {
        return Err(bad("the picture does not match its id"));
    }
    store(dir, &Picture { hash: hash.to_owned(), mime, bytes })?;
    Ok(())
}

/// Add a picture sent as base64 (what the page has after reading a file):
/// shrink it, store it, return its hash.
pub fn add_base64(dir: &Path, data: &str) -> Result<String> {
    let bytes = BASE64.decode(data.trim().as_bytes()).map_err(|_| bad("not a picture"))?;
    let pic = process(&bytes)?;
    store(dir, &pic)?;
    Ok(pic.hash)
}

/// The stored picture as a data URL; `None` if it is not here.
pub fn read_data_url(dir: &Path, hash: &str) -> Result<Option<String>> {
    Ok(read(dir, hash)?.map(|(bytes, mime)| data_url(&bytes, mime)))
}

/// `data:image/png;base64,…`, which the app's page may show.
pub fn data_url(bytes: &[u8], mime: &str) -> String {
    format!("data:{mime};base64,{}", BASE64.encode(bytes))
}

/// The hashes a note's Markdown points to, in order, without repeats.
pub fn referenced(body: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for (at, _) in body.match_indices(SCHEME) {
        let rest = &body[at + SCHEME.len()..];
        let Some(candidate) = rest.get(..64) else { continue };
        if is_hash(candidate) && !out.iter().any(|h| h == candidate) {
            out.push(candidate.to_owned());
        }
    }
    out
}

/// Every picture any note points to. Deleted notes count: sync can bring one back.
pub fn in_use(conn: &Connection) -> Result<HashSet<String>> {
    let mut stmt = conn.prepare("SELECT body FROM notes")?;
    let bodies = stmt.query_map([], |r| r.get::<_, String>(0))?;
    let mut used = HashSet::new();
    for body in bodies {
        used.extend(referenced(&body?));
    }
    Ok(used)
}

/// Delete the pictures in `dir` that no note points to. Files younger than
/// `grace` stay: a picture added to a note that is still being written is not
/// saved in it yet. Returns how many were removed.
pub fn clean(conn: &Connection, dir: &Path, grace: Duration) -> Result<usize> {
    let used = in_use(conn)?;
    let Ok(entries) = fs::read_dir(dir) else { return Ok(0) };
    let now = SystemTime::now();
    let mut removed = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else { continue };
        if !is_hash(stem) || used.contains(stem) {
            continue;
        }
        let age = entry.metadata().and_then(|m| m.modified()).ok().and_then(|t| now.duration_since(t).ok());
        if age.is_some_and(|a| a >= grace) && fs::remove_file(&path).is_ok() {
            removed += 1;
        }
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::{create, tests::setup};
    use image::{Rgba, RgbaImage};

    fn png_of(w: u32, h: u32, fill: impl FnMut(u32, u32) -> Rgba<u8>) -> Vec<u8> {
        let img = RgbaImage::from_fn(w, h, fill);
        let mut out = Vec::new();
        DynamicImage::ImageRgba8(img).write_to(&mut Cursor::new(&mut out), ImageFormat::Png).unwrap();
        out
    }

    fn temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("qd-notes-{name}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_small_screenshot_stays_png_and_has_a_stable_hash() {
        let png = png_of(120, 80, |x, y| Rgba([(x % 255) as u8, (y % 255) as u8, 40, 255]));
        let a = process(&png).unwrap();
        let b = process(&png).unwrap();
        assert_eq!((a.mime, a.hash.len()), ("image/png", 64));
        assert_eq!(a, b, "the same picture is the same file");
        assert!(is_hash(&a.hash));
    }

    #[test]
    fn big_pictures_are_shrunk_and_noisy_ones_become_jpeg() {
        let mut seed = 12345u32;
        let noise = png_of(2400, 1800, move |_, _| {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            Rgba([(seed >> 24) as u8, (seed >> 16) as u8, (seed >> 8) as u8, 255])
        });
        let pic = process(&noise).unwrap();
        assert_eq!(pic.mime, "image/jpeg", "random pixels do not fit a small PNG");
        let decoded = image::load_from_memory(&pic.bytes).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (1600, 1200), "longest side is 1600, ratio kept");
        assert!(pic.bytes.len() <= MAX_STORED_BYTES);
    }

    #[test]
    fn transparency_is_laid_on_white_when_it_has_to_be_jpeg() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, Rgba([255, 0, 0, 0])));
        assert_eq!(flatten(&img).get_pixel(0, 0).0, [255, 255, 255]);
        let half = DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, Rgba([0, 0, 0, 255])));
        assert_eq!(flatten(&half).get_pixel(0, 0).0, [0, 0, 0]);
    }

    #[test]
    fn what_is_not_a_picture_is_refused() {
        assert!(process(b"").is_err());
        assert!(process(b"hello, this is text").is_err());
        assert!(process(&vec![0u8; MAX_INPUT_BYTES + 1]).is_err());
    }

    #[test]
    fn store_and_read_roundtrip_and_reject_odd_names() {
        let dir = temp_dir("store");
        let pic = process(&png_of(10, 10, |_, _| Rgba([1, 2, 3, 255]))).unwrap();
        let path = store(&dir, &pic).unwrap();
        assert!(path.ends_with(format!("{}.png", pic.hash)));
        assert_eq!(store(&dir, &pic).unwrap(), path, "storing twice is harmless");
        let (bytes, mime) = read(&dir, &pic.hash).unwrap().unwrap();
        assert_eq!((bytes, mime), (pic.bytes.clone(), "image/png"));
        assert_eq!(read(&dir, &"0".repeat(64)).unwrap(), None, "not here yet");
        for odd in ["../etc/passwd", "ABCDEF", &"g".repeat(64), &format!("../{}", "a".repeat(61))] {
            assert!(read(&dir, odd).is_err(), "{odd}");
        }
        assert!(data_url(&pic.bytes, pic.mime).starts_with("data:image/png;base64,"));
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn referenced_finds_each_picture_once() {
        let (a, b) = ("a".repeat(64), "b".repeat(64));
        let body =
            format!("text ![]({SCHEME}{a}) more ![x]({SCHEME}{b})\n![]({SCHEME}{a}) ![]({SCHEME}short) {SCHEME}");
        assert_eq!(referenced(&body), [a, b]);
        assert!(referenced("nothing here").is_empty());
    }

    #[test]
    fn clean_keeps_what_notes_use_and_what_is_new() {
        let (db, clock) = setup();
        let conn = db.conn().unwrap();
        let dir = temp_dir("clean");
        let keep = process(&png_of(8, 8, |_, _| Rgba([9, 9, 9, 255]))).unwrap();
        let lone = process(&png_of(8, 8, |_, _| Rgba([200, 9, 9, 255]))).unwrap();
        let fresh = process(&png_of(8, 8, |_, _| Rgba([9, 200, 9, 255]))).unwrap();
        for p in [&keep, &lone, &fresh] {
            store(&dir, p).unwrap();
        }
        create(&conn, &clock, &format!("a note ![]({SCHEME}{})", keep.hash)).unwrap();
        // Make `keep` and `lone` old; `fresh` stays new.
        let old = SystemTime::now() - Duration::from_secs(3 * 86_400);
        for p in [&keep, &lone] {
            let f = fs::File::options().write(true).open(dir.join(format!("{}.png", p.hash))).unwrap();
            f.set_modified(old).unwrap();
        }
        assert_eq!(clean(&conn, &dir, Duration::from_secs(86_400)).unwrap(), 1);
        assert!(read(&dir, &keep.hash).unwrap().is_some(), "used by a note");
        assert!(read(&dir, &lone.hash).unwrap().is_none(), "old and unused");
        assert!(read(&dir, &fresh.hash).unwrap().is_some(), "too new to judge");
        // A deleted note still counts: sync can bring it back.
        let n = create(&conn, &clock, &format!("![]({SCHEME}{})", fresh.hash)).unwrap();
        crate::repo::delete(&conn, &clock, &n.id).unwrap();
        let f = fs::File::options().write(true).open(dir.join(format!("{}.png", fresh.hash))).unwrap();
        f.set_modified(old).unwrap();
        assert_eq!(clean(&conn, &dir, Duration::from_secs(86_400)).unwrap(), 0);
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn received_pictures_must_match_their_id() {
        let dir = temp_dir("received");
        let pic = process(&png_of(10, 10, |_, _| Rgba([5, 6, 7, 255]))).unwrap();
        assert!(!exists(&dir, &pic.hash));
        assert!(store_received(&dir, &"0".repeat(64), pic.bytes.clone()).is_err(), "wrong id");
        assert!(store_received(&dir, &pic.hash, b"<svg onload=alert(1)>".to_vec()).is_err(), "not a PNG or JPEG");
        assert!(store_received(&dir, "../x", pic.bytes.clone()).is_err());
        store_received(&dir, &pic.hash, pic.bytes.clone()).unwrap();
        assert!(exists(&dir, &pic.hash));
        assert_eq!(read(&dir, &pic.hash).unwrap().unwrap().0, pic.bytes);
    }
}
