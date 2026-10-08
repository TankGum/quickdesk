use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use rusqlite::{params, Connection};

use crate::{Error, Migration, Module, Result};

struct Core;

impl Module for Core {
    fn id(&self) -> &'static str {
        "core"
    }

    fn migrations(&self) -> &'static [Migration] {
        &[Migration {
            version: 1,
            sql: "CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
        }]
    }
}

/// The app database. A single writer connection guarded by a mutex; with WAL
/// this is plenty for a desktop app (a read pool can be added when needed).
pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    pub fn open(path: &Path, modules: &[&dyn Module]) -> Result<Self> {
        Self::init(Connection::open(path)?, modules)
    }

    pub fn open_in_memory(modules: &[&dyn Module]) -> Result<Self> {
        Self::init(Connection::open_in_memory()?, modules)
    }

    fn init(mut conn: Connection, modules: &[&dyn Module]) -> Result<Self> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "busy_timeout", 3000)?;
        migrate(&mut conn, &Core)?;
        for module in modules {
            migrate(&mut conn, *module)?;
        }
        Ok(Db { conn: Mutex::new(conn) })
    }

    pub fn conn(&self) -> Result<MutexGuard<'_, Connection>> {
        self.conn.lock().map_err(|_| Error::Poisoned)
    }
}

fn migrate(conn: &mut Connection, module: &dyn Module) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            module TEXT NOT NULL, version INTEGER NOT NULL, applied_at INTEGER NOT NULL,
            PRIMARY KEY (module, version));",
    )?;
    let current: u32 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations WHERE module = ?1",
        [module.id()],
        |r| r.get(0),
    )?;
    for m in module.migrations().iter().filter(|m| m.version > current) {
        let fail = |source| Error::Migration { module: module.id(), version: m.version, source };
        let tx = conn.transaction()?;
        tx.execute_batch(m.sql).map_err(fail)?;
        tx.execute(
            "INSERT INTO schema_migrations (module, version, applied_at)
             VALUES (?1, ?2, CAST(unixepoch('subsec') * 1000 AS INTEGER))",
            params![module.id(), m.version],
        )?;
        tx.commit()?;
        tracing::info!(module = module.id(), version = m.version, "applied migration");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake(&'static [Migration]);

    impl Module for Fake {
        fn id(&self) -> &'static str {
            "fake"
        }
        fn migrations(&self) -> &'static [Migration] {
            self.0
        }
    }

    const V1: &[Migration] = &[Migration { version: 1, sql: "CREATE TABLE fake_a (x INTEGER);" }];
    const V2: &[Migration] = &[
        Migration { version: 1, sql: "CREATE TABLE fake_a (x INTEGER);" },
        Migration { version: 2, sql: "ALTER TABLE fake_a ADD COLUMN y TEXT;" },
    ];

    fn applied(db: &Db) -> Vec<(String, u32)> {
        let conn = db.conn().unwrap();
        let mut stmt =
            conn.prepare("SELECT module, version FROM schema_migrations ORDER BY module, version").unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).unwrap().map(|r| r.unwrap()).collect()
    }

    #[test]
    fn applies_core_and_module_migrations() {
        let db = Db::open_in_memory(&[&Fake(V1)]).unwrap();
        assert_eq!(applied(&db), vec![("core".into(), 1), ("fake".into(), 1)]);
    }

    #[test]
    fn reopening_only_applies_new_versions() {
        let dir = std::env::temp_dir().join(format!("qd-core-test-{}", crate::ids::new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.db");

        drop(Db::open(&path, &[&Fake(V1)]).unwrap());
        let db = Db::open(&path, &[&Fake(V2)]).unwrap();
        assert_eq!(applied(&db), vec![("core".into(), 1), ("fake".into(), 1), ("fake".into(), 2)]);
        db.conn().unwrap().execute("INSERT INTO fake_a (x, y) VALUES (1, 'ok')", []).unwrap();

        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn failed_migration_is_rolled_back_and_reported() {
        const BAD: &[Migration] = &[Migration {
            version: 1,
            sql: "CREATE TABLE fake_b (x INTEGER); THIS IS NOT SQL;",
        }];
        let err = Db::open_in_memory(&[&Fake(BAD)]).err().expect("should fail");
        assert!(matches!(err, Error::Migration { module: "fake", version: 1, .. }), "{err}");
    }
}
