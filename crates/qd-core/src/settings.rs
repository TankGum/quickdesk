//! Local-only key/value settings stored as JSON.

use rusqlite::{params, Connection, OptionalExtension};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::Result;

pub fn get<T: DeserializeOwned>(conn: &Connection, key: &str) -> Result<Option<T>> {
    let raw: Option<String> = conn
        .query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| r.get(0))
        .optional()?;
    Ok(raw.map(|v| serde_json::from_str(&v)).transpose()?)
}

pub fn set<T: Serialize>(conn: &Connection, key: &str, value: &T) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT (key) DO UPDATE SET value = excluded.value",
        params![key, serde_json::to_string(value)?],
    )?;
    Ok(())
}

/// Returns the stored value, or stores and returns `init()` on first use.
pub fn get_or_init<T: Serialize + DeserializeOwned>(
    conn: &Connection,
    key: &str,
    init: impl FnOnce() -> T,
) -> Result<T> {
    if let Some(v) = get(conn, key)? {
        return Ok(v);
    }
    let v = init();
    set(conn, key, &v)?;
    Ok(v)
}

/// Stable identifier of this installation, used as HLC node and sync device id.
pub fn device_id(conn: &Connection) -> Result<String> {
    get_or_init(conn, "device_id", crate::ids::new_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Db;

    #[test]
    fn set_get_overwrite() {
        let db = Db::open_in_memory(&[]).unwrap();
        let conn = db.conn().unwrap();
        assert_eq!(get::<String>(&conn, "k").unwrap(), None);
        set(&conn, "k", &vec![1, 2]).unwrap();
        set(&conn, "k", &vec![3]).unwrap();
        assert_eq!(get::<Vec<i32>>(&conn, "k").unwrap(), Some(vec![3]));
    }

    #[test]
    fn device_id_is_stable() {
        let db = Db::open_in_memory(&[]).unwrap();
        let conn = db.conn().unwrap();
        let a = device_id(&conn).unwrap();
        assert_eq!(device_id(&conn).unwrap(), a);
    }
}
