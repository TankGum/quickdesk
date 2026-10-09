//! Push/pull of encrypted note batches through a blob store.
//!
//! Bucket layout (under the configured prefix, `v1/`):
//!
//! ```text
//! keyring.json                   DEK wrapped by passphrase / recovery key
//! devices/<device>.bin           encrypted device info
//! log/<device>/<seq:020>.bin     immutable encrypted batch of NoteOps
//! snapshots/<hlc>.bin            encrypted full state + per-device cursors
//! ```
//!
//! Every device appends only under its own `log/<device>/`, so devices never
//! overwrite each other's objects; conflicts are resolved by `qd_notes::sync`.

use std::collections::BTreeMap;

use qd_core::{settings, Clock, Db};
use qd_notes::sync::{all_ops, apply_remote, dirty_ops, mark_pushed, Applied, NoteOp};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::crypto::{Dek, KdfParams, Keyring, RecoveryKey};
use crate::transport::{BlobTransport, PutOutcome};
use crate::{Error, Result};

const SEQ_KEY: &str = "sync.seq";
const CURSORS_KEY: &str = "sync.cursors";
const SNAPSHOT_KEY: &str = "sync.snapshot";
const LAST_COMPACTION_KEY: &str = "sync.last_compaction";
/// The store's revision when the last full round started.
const REVISION_KEY: &str = "sync.revision";

/// Forget all sync progress (used when sync is disabled or re-pointed).
pub fn reset_local_state(conn: &rusqlite::Connection) -> Result<()> {
    conn.execute(
        "DELETE FROM settings WHERE key IN (?1, ?2, ?3, ?4, ?5)",
        [SEQ_KEY, CURSORS_KEY, SNAPSHOT_KEY, LAST_COMPACTION_KEY, REVISION_KEY],
    )?;
    // Everything must be pushed again to whatever bucket comes next.
    conn.execute("UPDATE notes SET dirty = 1", [])?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct Layout {
    root: String,
}

impl Layout {
    /// `prefix` is the user-configured key prefix (may be empty).
    pub fn new(prefix: &str) -> Self {
        let p = prefix.trim().trim_matches('/');
        Layout { root: if p.is_empty() { "v1/".into() } else { format!("{p}/v1/") } }
    }
    pub fn keyring(&self) -> String {
        format!("{}keyring.json", self.root)
    }
    fn devices(&self) -> String {
        format!("{}devices/", self.root)
    }
    fn device(&self, id: &str) -> String {
        format!("{}devices/{id}.bin", self.root)
    }
    fn logs(&self) -> String {
        format!("{}log/", self.root)
    }
    fn log_prefix(&self, device: &str) -> String {
        format!("{}log/{device}/", self.root)
    }
    fn log_key(&self, device: &str, seq: u64) -> String {
        format!("{}log/{device}/{seq:020}.bin", self.root)
    }
    fn snapshots(&self) -> String {
        format!("{}snapshots/", self.root)
    }
    fn snapshot_key(&self, hlc: &str) -> String {
        format!("{}snapshots/{hlc}.bin", self.root)
    }
    /// `log/<device>/<seq>.bin` → (device, seq)
    fn parse_log_key(&self, key: &str) -> Option<(String, u64)> {
        let rest = key.strip_prefix(&self.logs())?;
        let (dev, file) = rest.split_once('/')?;
        Some((dev.to_owned(), file.strip_suffix(".bin")?.parse().ok()?))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Batch {
    device: String,
    seq: u64,
    ops: Vec<NoteOp>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    created_by: String,
    /// Last log key of each device already folded into `ops`.
    cursors: BTreeMap<String, String>,
    ops: Vec<NoteOp>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeviceInfo {
    device_id: String,
    name: String,
    last_seq: u64,
    updated_at: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct EngineConfig {
    /// Write a snapshot once the bucket holds this many log objects.
    pub compact_threshold: usize,
    /// Logs covered by a snapshot are deleted once older than this.
    pub retention_ms: i64,
    pub compaction_interval_ms: i64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        EngineConfig {
            compact_threshold: 500,
            retention_ms: 30 * 24 * 3600 * 1000,
            compaction_interval_ms: 6 * 3600 * 1000,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncReport {
    pub pulled: usize,
    pub pushed: usize,
    pub conflicts: usize,
    pub compacted: bool,
}

fn now_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

/// Fetch `keyring.json`, if this sync space was initialized.
pub fn fetch_keyring(t: &dyn BlobTransport, layout: &Layout) -> Result<Option<Keyring>> {
    t.get(&layout.keyring())?
        .map(|b| serde_json::from_slice(&b).map_err(|e| Error::Remote(format!("corrupt keyring.json: {e}"))))
        .transpose()
}

/// Initialize a new sync space. Fails if one already exists at this location.
pub fn create_keyring(
    t: &dyn BlobTransport,
    layout: &Layout,
    passphrase: &str,
    kdf: KdfParams,
) -> Result<(Dek, RecoveryKey)> {
    let dek = Dek::generate();
    let recovery = RecoveryKey::generate();
    let keyring = Keyring::create(&dek, passphrase, &recovery, kdf)?;
    match t.put_if_absent(&layout.keyring(), &serde_json::to_vec_pretty(&keyring)?)? {
        PutOutcome::Created => Ok((dek, recovery)),
        PutOutcome::AlreadyExists => Err(Error::AlreadyInitialized),
    }
}

/// Unlock an existing sync space with the passphrase or recovery key.
pub fn unlock(t: &dyn BlobTransport, layout: &Layout, secret: &str) -> Result<Dek> {
    fetch_keyring(t, layout)?.ok_or(Error::NotInitialized)?.unlock(secret)
}

pub struct SyncEngine<'a> {
    pub transport: &'a dyn BlobTransport,
    pub dek: &'a Dek,
    pub layout: Layout,
    pub device_id: &'a str,
    pub device_name: &'a str,
    pub config: EngineConfig,
}

impl SyncEngine<'_> {
    fn write<T: Serialize>(&self, key: &str, value: &T) -> Result<Vec<u8>> {
        self.dek.seal(key, &serde_json::to_vec(value)?)
    }

    fn read<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let Some(sealed) = self.transport.get(key)? else { return Ok(None) };
        let plain = self.dek.open(key, &sealed)?;
        Ok(Some(serde_json::from_slice(&plain)?))
    }

    /// One round: pull, push, and occasionally compact. Costs a single call
    /// when the store's revision is the one seen at the last round and
    /// nothing local is waiting.
    pub fn sync_once(&self, db: &Db, clock: &Clock) -> Result<SyncReport> {
        // Read before pulling: a write that lands during the round changes the
        // revision again, so the next round still looks.
        let revision = self.transport.revision()?;
        if let Some(rev) = &revision {
            let conn = db.conn()?;
            let seen: Option<String> = settings::get(&conn, REVISION_KEY)?;
            let pending: bool =
                conn.query_row("SELECT EXISTS(SELECT 1 FROM notes WHERE dirty = 1)", [], |r| r.get(0))?;
            if seen.as_deref() == Some(rev.as_str()) && !pending {
                return Ok(SyncReport::default());
            }
        }
        let mut report = SyncReport::default();
        self.pull(db, clock, &mut report)?;
        self.push(db, &mut report)?;
        report.compacted = self.maybe_compact(db, clock)?;
        if let Some(rev) = revision {
            settings::set(&*db.conn()?, REVISION_KEY, &rev)?;
        }
        Ok(report)
    }

    fn cursors(conn: &rusqlite::Connection) -> Result<BTreeMap<String, String>> {
        Ok(settings::get(conn, CURSORS_KEY)?.unwrap_or_default())
    }

    /// Apply `ops` and persist `cursors` atomically.
    fn apply(
        &self,
        db: &Db,
        clock: &Clock,
        ops: &[NoteOp],
        update: impl FnOnce(&mut BTreeMap<String, String>, &rusqlite::Connection) -> Result<()>,
        report: &mut SyncReport,
    ) -> Result<()> {
        let mut conn = db.conn()?;
        let tx = conn.transaction()?;
        for op in ops {
            if let Applied::Conflict { .. } = apply_remote(&tx, clock, op)? {
                report.conflicts += 1;
            }
        }
        let mut cursors = Self::cursors(&tx)?;
        update(&mut cursors, &tx)?;
        settings::set(&tx, CURSORS_KEY, &cursors)?;
        tx.commit()?;
        report.pulled += ops.len();
        Ok(())
    }

    fn pull(&self, db: &Db, clock: &Clock, report: &mut SyncReport) -> Result<()> {
        let (applied_snapshot, cursors): (Option<String>, BTreeMap<String, String>) = {
            let conn = db.conn()?;
            (settings::get(&conn, SNAPSHOT_KEY)?, Self::cursors(&conn)?)
        };

        // A newer snapshot may cover logs that were compacted away.
        let newer = self.transport.list(&self.layout.snapshots(), applied_snapshot.as_deref())?;
        if let Some(latest) = newer.last() {
            if let Some(snap) = self.read::<Snapshot>(&latest.key)? {
                let key = latest.key.clone();
                self.apply(
                    db,
                    clock,
                    &snap.ops,
                    |cursors, conn| {
                        for (dev, last) in snap.cursors {
                            let c = cursors.entry(dev).or_default();
                            if *c < last {
                                *c = last;
                            }
                        }
                        settings::set(conn, SNAPSHOT_KEY, &key)?;
                        Ok(())
                    },
                    report,
                )?;
                tracing::info!(snapshot = %key, ops = snap.ops.len(), "applied snapshot");
            }
        }

        let cursors = if newer.is_empty() { cursors } else { Self::cursors(&*db.conn()?)? };
        let mut devices: Vec<String> = self
            .transport
            .list(&self.layout.devices(), None)?
            .into_iter()
            .filter_map(|o| o.key.strip_prefix(&self.layout.devices())?.strip_suffix(".bin").map(str::to_owned))
            .collect();
        // Include ourselves: recovers our own changes if the local DB was restored from a backup.
        if !devices.iter().any(|d| d == self.device_id) {
            devices.push(self.device_id.to_owned());
        }
        for dev in devices.iter().map(String::as_str) {
            let after = cursors.get(dev).map(String::as_str);
            for obj in self.transport.list(&self.layout.log_prefix(dev), after)? {
                let Some(batch) = self.read::<Batch>(&obj.key)? else { continue };
                let key = obj.key.clone();
                self.apply(
                    db,
                    clock,
                    &batch.ops,
                    |c, _| {
                        c.insert(dev.to_owned(), key);
                        Ok(())
                    },
                    report,
                )?;
            }
        }
        Ok(())
    }

    fn push(&self, db: &Db, report: &mut SyncReport) -> Result<()> {
        let (ops, last_seq): (Vec<NoteOp>, u64) = {
            let conn = db.conn()?;
            (dirty_ops(&conn)?, settings::get(&conn, SEQ_KEY)?.unwrap_or(0))
        };
        if ops.is_empty() {
            return Ok(());
        }
        let mut seq = last_seq + 1;
        let key = loop {
            let key = self.layout.log_key(self.device_id, seq);
            let batch = Batch { device: self.device_id.to_owned(), seq, ops: ops.clone() };
            match self.transport.put_if_absent(&key, &self.write(&key, &batch)?)? {
                PutOutcome::Created => break key,
                PutOutcome::AlreadyExists => {
                    // Our counter is behind (restored DB, or sync re-enabled): continue after the last one.
                    let existing = self.transport.list(&self.layout.log_prefix(self.device_id), None)?;
                    let max = existing
                        .iter()
                        .filter_map(|o| self.layout.parse_log_key(&o.key))
                        .map(|(_, s)| s)
                        .max()
                        .unwrap_or(0);
                    if max < seq {
                        return Err(Error::Remote(format!("{key} exists but is not listed")));
                    }
                    tracing::warn!(seq, next = max + 1, "log sequence was behind the bucket");
                    seq = max + 1;
                }
            }
        };
        {
            let conn = db.conn()?;
            mark_pushed(&conn, &ops)?;
            settings::set(&conn, SEQ_KEY, &seq)?;
            let mut cursors = Self::cursors(&conn)?;
            cursors.insert(self.device_id.to_owned(), key);
            settings::set(&conn, CURSORS_KEY, &cursors)?;
        }
        let info = DeviceInfo {
            device_id: self.device_id.into(),
            name: self.device_name.into(),
            last_seq: seq,
            updated_at: now_ms(),
        };
        let dkey = self.layout.device(self.device_id);
        self.transport.put(&dkey, &self.write(&dkey, &info)?)?;
        report.pushed += ops.len();
        Ok(())
    }

    fn maybe_compact(&self, db: &Db, clock: &Clock) -> Result<bool> {
        let now = now_ms();
        let last: i64 = settings::get(&*db.conn()?, LAST_COMPACTION_KEY)?.unwrap_or(0);
        if now - last < self.config.compaction_interval_ms {
            return Ok(false);
        }
        settings::set(&*db.conn()?, LAST_COMPACTION_KEY, &now)?;
        let logs = self.transport.list(&self.layout.logs(), None)?;
        if logs.len() < self.config.compact_threshold {
            return Ok(false);
        }

        // Pull and push just ran, so local state includes everything up to `cursors`.
        let (ops, cursors) = {
            let conn = db.conn()?;
            (all_ops(&conn)?, Self::cursors(&conn)?)
        };
        let key = self.layout.snapshot_key(&clock.now().to_string());
        let snapshot = Snapshot { created_by: self.device_id.into(), cursors: cursors.clone(), ops };
        if self.transport.put_if_absent(&key, &self.write(&key, &snapshot)?)? == PutOutcome::AlreadyExists {
            return Ok(false);
        }
        settings::set(&*db.conn()?, SNAPSHOT_KEY, &key)?;

        let cutoff = now - self.config.retention_ms;
        let mut deleted = 0;
        for o in &logs {
            let covered =
                self.layout.parse_log_key(&o.key).and_then(|(dev, _)| cursors.get(&dev)).is_some_and(|c| o.key <= *c);
            if covered && o.last_modified_ms > 0 && o.last_modified_ms < cutoff {
                self.transport.delete(&o.key)?;
                deleted += 1;
            }
        }
        let snaps = self.transport.list(&self.layout.snapshots(), None)?;
        for s in snaps.iter().filter(|s| s.key != key && s.last_modified_ms > 0 && s.last_modified_ms < cutoff) {
            self.transport.delete(&s.key)?;
        }
        tracing::info!(snapshot = %key, deleted, "compacted sync log");
        Ok(true)
    }
}
