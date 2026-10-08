//! Live test against a real S3-compatible endpoint (R2, S3, MinIO, RustFS):
//!
//! ```sh
//! QD_S3_ENDPOINT=http://127.0.0.1:19000 QD_S3_BUCKET=qd-test QD_S3_REGION=us-east-1 \
//! QD_S3_KEY=... QD_S3_SECRET=... cargo test -p qd-sync --test s3_live -- --ignored
//! ```
//! Creates the bucket if needed and writes under a random prefix.

use std::time::Duration;

use qd_core::{Clock, Db};
use qd_notes::{repo, NotesModule};
use qd_sync::crypto::KdfParams;
use qd_sync::transport::{BlobTransport, PutOutcome};
use qd_sync::{create_keyring, unlock, EngineConfig, Layout, S3Config, S3Transport, SyncEngine};
use rusty_s3::{Bucket, Credentials, S3Action, UrlStyle};

fn env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} not set"))
}

fn config(prefix: &str) -> (S3Config, String) {
    (
        S3Config {
            endpoint: env("QD_S3_ENDPOINT"),
            bucket: env("QD_S3_BUCKET"),
            region: std::env::var("QD_S3_REGION").unwrap_or_else(|_| "us-east-1".into()),
            access_key_id: env("QD_S3_KEY"),
            prefix: prefix.into(),
        },
        env("QD_S3_SECRET"),
    )
}

fn ensure_bucket(cfg: &S3Config, secret: &str) {
    let bucket =
        Bucket::new(cfg.endpoint.parse().unwrap(), UrlStyle::Path, cfg.bucket.clone(), cfg.region.clone()).unwrap();
    let creds = Credentials::new(cfg.access_key_id.clone(), secret.to_owned());
    let url = bucket.create_bucket(&creds).sign(Duration::from_secs(60));
    match ureq::put(url.as_str()).call() {
        Ok(_) | Err(ureq::Error::Status(409, _)) => {}
        Err(e) => panic!("create bucket: {e}"),
    }
}

#[test]
#[ignore = "needs a live S3-compatible endpoint"]
fn s3_transport_and_two_device_sync() {
    let prefix = format!("it-{}", qd_core::ids::new_id());
    let (cfg, secret) = config(&prefix);
    ensure_bucket(&cfg, &secret);
    let t = S3Transport::new(&cfg, &secret).unwrap();

    // Conditional create, get, list with start_after, delete.
    let k = |s: &str| format!("{prefix}/raw/{s}");
    assert_eq!(t.put_if_absent(&k("a"), b"1").unwrap(), PutOutcome::Created);
    assert_eq!(t.put_if_absent(&k("a"), b"2").unwrap(), PutOutcome::AlreadyExists, "conditional PUT not honored");
    assert_eq!(t.get(&k("a")).unwrap().as_deref(), Some(&b"1"[..]));
    assert_eq!(t.get(&k("missing")).unwrap(), None);
    t.put(&k("b"), b"3").unwrap();
    let listed: Vec<String> =
        t.list(&format!("{prefix}/raw/"), Some(&k("a"))).unwrap().into_iter().map(|o| o.key).collect();
    assert_eq!(listed, vec![k("b")]);
    assert!(t.list(&format!("{prefix}/raw/"), None).unwrap().iter().all(|o| o.last_modified_ms > 0));

    // Pagination: more keys than one LIST page (1000).
    for i in 0..1005 {
        t.put(&format!("{prefix}/many/{i:05}"), b"x").unwrap();
    }
    assert_eq!(t.list(&format!("{prefix}/many/"), None).unwrap().len(), 1005);

    // Two devices syncing through the real endpoint.
    let layout = Layout::new(&prefix);
    let kdf = KdfParams { memory_kib: 1024, iterations: 1, parallelism: 1 };
    let (dek, recovery) = create_keyring(&t, &layout, "correct horse battery", kdf).unwrap();
    assert_eq!(unlock(&t, &layout, &recovery.display()).unwrap(), dek);
    let (a_db, b_db) = (Db::open_in_memory(&[&NotesModule]).unwrap(), Db::open_in_memory(&[&NotesModule]).unwrap());
    let (a_clock, b_clock) = (Clock::new("a"), Clock::new("b"));
    let engine = |id: &'static str| SyncEngine {
        transport: &t,
        dek: &dek,
        layout: layout.clone(),
        device_id: id,
        device_name: id,
        config: EngineConfig { compaction_interval_ms: i64::MAX, ..Default::default() },
    };
    repo::create(&a_db.conn().unwrap(), &a_clock, "synced through real S3").unwrap();
    assert_eq!(engine("a").sync_once(&a_db, &a_clock).unwrap().pushed, 1);
    assert_eq!(engine("b").sync_once(&b_db, &b_clock).unwrap().pulled, 1);
    let bodies: Vec<String> = repo::list(&b_db.conn().unwrap(), 10).unwrap().into_iter().map(|n| n.body).collect();
    assert_eq!(bodies, vec!["synced through real S3"]);

    // Clean up everything under the prefix.
    for o in t.list(&format!("{prefix}/"), None).unwrap() {
        t.delete(&o.key).unwrap();
    }
    assert!(t.list(&format!("{prefix}/"), None).unwrap().is_empty());
}
