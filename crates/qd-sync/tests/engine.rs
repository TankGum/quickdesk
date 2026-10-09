use std::sync::atomic::{AtomicUsize, Ordering};

use qd_core::{settings, Clock, Db};
use qd_notes::{repo, NotesModule};
use qd_sync::crypto::{Dek, KdfParams};
use qd_sync::engine::reset_local_state;
use qd_sync::transport::{ObjectInfo, PutOutcome};
use qd_sync::{create_keyring, unlock, BlobTransport, EngineConfig, Error, Layout, MemoryTransport, SyncEngine};

const KDF: KdfParams = KdfParams { memory_kib: 64, iterations: 1, parallelism: 1 };

struct Device {
    id: String,
    db: Db,
    clock: Clock,
}

fn device(id: &str) -> Device {
    Device { id: id.into(), db: Db::open_in_memory(&[&NotesModule]).unwrap(), clock: Clock::new(id) }
}

fn no_compaction() -> EngineConfig {
    EngineConfig { compaction_interval_ms: i64::MAX, ..EngineConfig::default() }
}

impl Device {
    fn sync_with(&self, t: &MemoryTransport, dek: &Dek, config: EngineConfig) -> qd_sync::Result<qd_sync::SyncReport> {
        SyncEngine { transport: t, dek, layout: Layout::new("qd"), device_id: &self.id, device_name: "test", config }
            .sync_once(&self.db, &self.clock)
    }
    fn sync(&self, t: &MemoryTransport, dek: &Dek) -> qd_sync::SyncReport {
        self.sync_with(t, dek, no_compaction()).unwrap()
    }
    fn create(&self, body: &str) -> String {
        repo::create(&self.db.conn().unwrap(), &self.clock, body).unwrap().id
    }
    fn edit(&self, id: &str, body: &str) {
        repo::update(&self.db.conn().unwrap(), &self.clock, id, Some(body), None).unwrap();
    }
    fn bodies(&self) -> Vec<String> {
        let mut b: Vec<String> =
            repo::list(&self.db.conn().unwrap(), 1000).unwrap().into_iter().map(|n| n.body).collect();
        b.sort();
        b
    }
}

fn setup() -> (MemoryTransport, Dek) {
    let t = MemoryTransport::default();
    let (dek, _) = create_keyring(&t, &Layout::new("qd"), "correct horse battery", KDF).unwrap();
    (t, dek)
}

#[test]
fn keyring_lifecycle() {
    let t = MemoryTransport::default();
    let layout = Layout::new("qd");
    assert!(matches!(unlock(&t, &layout, "anything"), Err(Error::NotInitialized)));
    let (dek, recovery) = create_keyring(&t, &layout, "correct horse battery", KDF).unwrap();
    assert!(matches!(create_keyring(&t, &layout, "other passphrase", KDF), Err(Error::AlreadyInitialized)));
    assert_eq!(unlock(&t, &layout, "correct horse battery").unwrap(), dek);
    assert_eq!(unlock(&t, &layout, &recovery.display()).unwrap(), dek);
    assert!(matches!(unlock(&t, &layout, "wrong"), Err(Error::WrongSecret)));
}

#[test]
fn notes_propagate_and_bucket_holds_only_ciphertext() {
    let (t, dek) = setup();
    let (a, b) = (device("a"), device("b"));
    a.create("check Pub/Sub retry after deploy");
    assert_eq!(a.sync(&t, &dek).pushed, 1);
    assert_eq!(b.sync(&t, &dek).pulled, 1);
    assert_eq!(b.bodies(), vec!["check Pub/Sub retry after deploy"]);

    for key in t.keys().iter().filter(|k| !k.ends_with("keyring.json")) {
        let raw = String::from_utf8_lossy(&t.raw(key).unwrap()).into_owned();
        assert!(!raw.contains("Pub/Sub"), "{key} leaks plaintext");
    }
    // Nothing new: a second round transfers nothing.
    assert_eq!(a.sync(&t, &dek), Default::default());
    assert_eq!(b.sync(&t, &dek), Default::default());
}

/// Counts calls, to check what an idle round costs.
struct Counting<'a> {
    inner: &'a MemoryTransport,
    calls: AtomicUsize,
}

impl Counting<'_> {
    fn take(&self) -> usize {
        self.calls.swap(0, Ordering::Relaxed)
    }
    fn hit(&self) {
        self.calls.fetch_add(1, Ordering::Relaxed);
    }
}

impl BlobTransport for Counting<'_> {
    fn put_if_absent(&self, key: &str, bytes: &[u8]) -> qd_sync::Result<PutOutcome> {
        self.hit();
        self.inner.put_if_absent(key, bytes)
    }
    fn put(&self, key: &str, bytes: &[u8]) -> qd_sync::Result<()> {
        self.hit();
        self.inner.put(key, bytes)
    }
    fn get(&self, key: &str) -> qd_sync::Result<Option<Vec<u8>>> {
        self.hit();
        self.inner.get(key)
    }
    fn list(&self, prefix: &str, start_after: Option<&str>) -> qd_sync::Result<Vec<ObjectInfo>> {
        self.hit();
        self.inner.list(prefix, start_after)
    }
    fn delete(&self, key: &str) -> qd_sync::Result<()> {
        self.hit();
        self.inner.delete(key)
    }
    fn revision(&self) -> qd_sync::Result<Option<String>> {
        self.hit();
        self.inner.revision()
    }
}

#[test]
fn idle_rounds_cost_one_call_and_still_see_changes() {
    let (t, dek) = setup();
    let (a, b) = (device("a"), device("b"));
    let counted = Counting { inner: &t, calls: AtomicUsize::new(0) };
    let a_sync = || {
        SyncEngine {
            transport: &counted,
            dek: &dek,
            layout: Layout::new("qd"),
            device_id: &a.id,
            device_name: "test",
            config: no_compaction(),
        }
        .sync_once(&a.db, &a.clock)
        .unwrap()
    };
    a.create("first");
    assert_eq!(a_sync().pushed, 1);
    // Its own push changed the revision: one more full round, then idle.
    a_sync();
    counted.take();
    assert_eq!(a_sync(), Default::default());
    assert_eq!(counted.take(), 1, "an idle round only reads the revision");

    // Another device writes: the next round looks again.
    b.create("from b");
    b.sync(&t, &dek);
    assert_eq!(a_sync().pulled, 1);
    counted.take();
    assert_eq!(a_sync(), Default::default());
    assert_eq!(counted.take(), 1);

    // A local change is pushed even though the store did not change.
    a.create("second");
    assert_eq!(a_sync().pushed, 1);

    // Forgetting sync state forces a full round.
    a_sync();
    reset_local_state(&a.db.conn().unwrap()).unwrap();
    counted.take();
    a_sync();
    assert!(counted.take() > 1);
}

#[test]
fn concurrent_offline_edits_keep_both_versions() {
    let (t, dek) = setup();
    let (a, b) = (device("a"), device("b"));
    let id = a.create("base");
    a.sync(&t, &dek);
    b.sync(&t, &dek);
    a.edit(&id, "edited on laptop");
    b.edit(&id, "edited on desktop");
    a.sync(&t, &dek);
    let report = b.sync(&t, &dek);
    assert_eq!(report.conflicts, 1);
    a.sync(&t, &dek);
    b.sync(&t, &dek);
    assert_eq!(a.bodies(), b.bodies());
    assert_eq!(a.bodies(), vec!["edited on desktop", "edited on laptop"]);
}

#[test]
fn offline_changes_are_kept_and_pushed_later() {
    let (t, dek) = setup();
    let (a, b) = (device("a"), device("b"));
    t.offline.store(true, Ordering::Relaxed);
    a.create("written on a plane");
    let err = a.sync_with(&t, &dek, no_compaction()).unwrap_err();
    assert!(err.is_transient(), "{err}");
    t.offline.store(false, Ordering::Relaxed);
    a.sync(&t, &dek);
    b.sync(&t, &dek);
    assert_eq!(b.bodies(), vec!["written on a plane"]);
}

#[test]
fn wrong_key_cannot_read_the_log() {
    let (t, dek) = setup();
    let a = device("a");
    a.create("secret");
    a.sync(&t, &dek);
    let err = device("b").sync_with(&t, &Dek::generate(), no_compaction()).unwrap_err();
    assert!(matches!(err, Error::Crypto(_)), "{err}");
}

#[test]
fn reset_state_does_not_overwrite_existing_log_objects() {
    let (t, dek) = setup();
    let (a, b) = (device("a"), device("b"));
    a.create("one");
    a.sync(&t, &dek);
    // Sync disabled and re-enabled: seq counter restarts at 0.
    reset_local_state(&a.db.conn().unwrap()).unwrap();
    a.create("two");
    a.sync(&t, &dek);
    b.sync(&t, &dek);
    assert_eq!(b.bodies(), vec!["one", "two"]);
    let a_logs = t.keys().into_iter().filter(|k| k.contains("/log/a/")).count();
    assert_eq!(a_logs, 2, "second push must not replace the first object");
}

#[test]
fn restored_database_recovers_own_changes_from_bucket() {
    let (t, dek) = setup();
    let a = device("a");
    a.create("only copy lives in the bucket");
    a.sync(&t, &dek);
    // Same device id, but an empty database (e.g. restored from an old backup).
    let restored = Device { id: "a".into(), db: Db::open_in_memory(&[&NotesModule]).unwrap(), clock: Clock::new("a") };
    restored.sync(&t, &dek);
    assert_eq!(restored.bodies(), vec!["only copy lives in the bucket"]);
}

#[test]
fn compaction_snapshot_lets_new_devices_join_after_logs_are_deleted() {
    let (t, dek) = setup();
    let (a, b) = (device("a"), device("b"));
    for i in 0..4 {
        a.create(&format!("note {i}"));
        a.sync(&t, &dek);
    }
    b.sync(&t, &dek);
    // Pretend all logs are old, then compact with a low threshold.
    for k in t.keys() {
        t.set_modified(&k, 1);
    }
    let compact = EngineConfig { compact_threshold: 3, retention_ms: 1000, compaction_interval_ms: 0 };
    assert!(a.sync_with(&t, &dek, compact).unwrap().compacted);
    let logs = t.keys().into_iter().filter(|k| k.contains("/log/")).count();
    assert_eq!(logs, 0, "covered, old logs deleted: {:?}", t.keys());
    assert_eq!(t.keys().iter().filter(|k| k.contains("/snapshots/")).count(), 1);

    // A brand-new device bootstraps from the snapshot.
    let c = device("c");
    c.sync(&t, &dek);
    assert_eq!(c.bodies(), a.bodies());
    assert_eq!(c.bodies().len(), 4);

    // And keeps syncing incrementally afterwards.
    b.create("after compaction");
    b.sync(&t, &dek);
    c.sync(&t, &dek);
    a.sync(&t, &dek);
    assert_eq!(a.bodies(), c.bodies());
    assert_eq!(a.bodies().len(), 5);
    assert!(settings::get::<String>(&c.db.conn().unwrap(), "sync.snapshot").unwrap().is_some());
}
