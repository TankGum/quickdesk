//! Live test against a running QuickDesk Cloud sync server:
//!
//! ```sh
//! (cd sync-server && npx wrangler dev)          # http://127.0.0.1:8787
//! QD_SYNC_URL=http://127.0.0.1:8787 cargo test -p qd-sync --test cloud_live -- --ignored
//! ```
//! Creates a fresh account and deletes it at the end.

use qd_core::{Clock, Db};
use qd_notes::{repo, NotesModule};
use qd_sync::cloud::{create_account, delete_account, endpoint};
use qd_sync::crypto::KdfParams;
use qd_sync::transport::{BlobTransport, PutOutcome};
use qd_sync::{create_keyring, unlock, Account, CloudTransport, EngineConfig, Layout, SyncEngine};

#[test]
#[ignore = "needs a running sync server (QD_SYNC_URL)"]
fn cloud_transport_and_two_device_sync() {
    let ep = endpoint();
    let account = create_account(&ep).unwrap();
    // The second device only has the sync code.
    let joined = Account::from_sync_code(&account.sync_code()).unwrap();
    let a_t = CloudTransport::new(&ep, &account);
    let b_t = CloudTransport::new(&ep, &joined);

    assert_eq!(a_t.put_if_absent("raw/a", b"1").unwrap(), PutOutcome::Created);
    assert_eq!(a_t.put_if_absent("raw/a", b"2").unwrap(), PutOutcome::AlreadyExists);
    assert_eq!(b_t.get("raw/a").unwrap().as_deref(), Some(&b"1"[..]));
    assert!(a_t.list("raw/", None).unwrap().iter().all(|o| o.last_modified_ms > 0));

    // More keys than one R2 LIST page (1000).
    for i in 0..1005 {
        a_t.put(&format!("many/{i:05}"), b"x").unwrap();
    }
    assert_eq!(b_t.list("many/", None).unwrap().len(), 1005);
    assert_eq!(b_t.list("many/", Some("many/01000")).unwrap().len(), 4);

    // Two devices syncing notes through the server.
    let layout = Layout::new("");
    let kdf = KdfParams { memory_kib: 1024, iterations: 1, parallelism: 1 };
    let (dek, recovery) = create_keyring(&a_t, &layout, "correct horse battery", kdf).unwrap();
    assert_eq!(unlock(&b_t, &layout, "correct horse battery").unwrap(), dek);
    assert_eq!(unlock(&b_t, &layout, &recovery.display()).unwrap(), dek);
    let (a_db, b_db) = (Db::open_in_memory(&[&NotesModule]).unwrap(), Db::open_in_memory(&[&NotesModule]).unwrap());
    let (a_clock, b_clock) = (Clock::new("a"), Clock::new("b"));
    let config = EngineConfig { compaction_interval_ms: i64::MAX, ..Default::default() };
    let engine = |t: &'static CloudTransport, id: &'static str| SyncEngine {
        transport: t,
        dek: &dek,
        layout: layout.clone(),
        device_id: id,
        device_name: id,
        config,
    };
    let (a_t, b_t): (&'static CloudTransport, &'static CloudTransport) =
        (Box::leak(Box::new(a_t)), Box::leak(Box::new(b_t)));
    repo::create(&a_db.conn().unwrap(), &a_clock, "synced through QuickDesk Cloud").unwrap();
    assert_eq!(engine(a_t, "a").sync_once(&a_db, &a_clock).unwrap().pushed, 1);
    assert_eq!(engine(b_t, "b").sync_once(&b_db, &b_clock).unwrap().pulled, 1);
    let bodies: Vec<String> = repo::list(&b_db.conn().unwrap(), 10).unwrap().into_iter().map(|n| n.body).collect();
    assert_eq!(bodies, vec!["synced through QuickDesk Cloud"]);

    delete_account(&ep, &account).unwrap();
    assert!(b_t.list("", None).is_err(), "deleted accounts are gone");
}
