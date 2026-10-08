//! Sync bookkeeping: export pending changes and merge remote versions.
//!
//! Each version of a note carries `hlc` (its own version) and `base_hlc` (the
//! newest published version it was derived from). Version B *descends* from A
//! when `B.base_hlc >= A.hlc`. Two versions where neither descends from the
//! other are concurrent and get resolved deterministically, so every device
//! converges to the same state no matter the order it sees operations in:
//!
//! * same title and body  → merge silently (newest `pinned`/`deleted` wins)
//! * edit vs delete       → the edit wins (resurrect)
//! * different text       → newest HLC keeps the id; the other version is saved
//!   as a conflict copy whose id is derived from `(note id, loser hlc)`, so
//!   all devices create the *same* copy instead of duplicating it

use qd_core::{ids, Clock, Hlc};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use crate::Result;

/// Full state of one note version, as exchanged between devices.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteOp {
    pub id: String,
    pub hlc: String,
    pub base_hlc: Option<String>,
    /// Absent in batches written before titles existed.
    #[serde(default)]
    pub title: String,
    pub body: String,
    pub pinned: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
    pub conflict_of: Option<String>,
}

impl NoteOp {
    fn base(&self) -> &str {
        self.base_hlc.as_deref().unwrap_or("")
    }

    /// True when `self` was derived from (has seen) version `hlc`.
    fn descends_from(&self, hlc: &str) -> bool {
        self.base() >= hlc
    }

    /// Title and body: what a person wrote.
    fn same_text(&self, other: &NoteOp) -> bool {
        self.title == other.title && self.body == other.body
    }

    fn same_content(&self, other: &NoteOp) -> bool {
        self.same_text(other)
            && self.pinned == other.pinned
            && self.deleted_at.is_some() == other.deleted_at.is_some()
            && self.conflict_of == other.conflict_of
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Applied {
    Inserted,
    FastForwarded,
    /// Already had this version or a newer one that includes it.
    Skipped,
    /// Concurrent, but resolved without a conflict copy.
    Merged,
    /// Concurrent edits with different bodies; the loser was saved as `copy_id`.
    Conflict {
        copy_id: String,
    },
}

const OP_COLS: &str = "id, hlc, base_hlc, title, body, pinned, created_at, updated_at, deleted_at, conflict_of";

fn op_from_row(r: &Row<'_>) -> rusqlite::Result<NoteOp> {
    Ok(NoteOp {
        id: r.get(0)?,
        hlc: r.get(1)?,
        base_hlc: r.get(2)?,
        title: r.get(3)?,
        body: r.get(4)?,
        pinned: r.get(5)?,
        created_at: r.get(6)?,
        updated_at: r.get(7)?,
        deleted_at: r.get(8)?,
        conflict_of: r.get(9)?,
    })
}

fn query_ops(conn: &Connection, filter: &str) -> Result<Vec<NoteOp>> {
    let mut stmt = conn.prepare(&format!("SELECT {OP_COLS} FROM notes {filter} ORDER BY hlc"))?;
    let ops = stmt.query_map([], op_from_row)?.collect::<rusqlite::Result<_>>()?;
    Ok(ops)
}

/// Local changes not yet pushed (including tombstones).
pub fn dirty_ops(conn: &Connection) -> Result<Vec<NoteOp>> {
    query_ops(conn, "WHERE dirty = 1")
}

/// Every row including tombstones, for snapshots.
pub fn all_ops(conn: &Connection) -> Result<Vec<NoteOp>> {
    query_ops(conn, "")
}

/// Clear `dirty` for versions that were pushed, unless edited meanwhile.
pub fn mark_pushed(conn: &Connection, ops: &[NoteOp]) -> Result<()> {
    let mut stmt = conn.prepare_cached("UPDATE notes SET dirty = 0 WHERE id = ?1 AND hlc = ?2")?;
    for op in ops {
        stmt.execute(params![op.id, op.hlc])?;
    }
    Ok(())
}

fn load(conn: &Connection, id: &str) -> Result<Option<NoteOp>> {
    Ok(conn.query_row(&format!("SELECT {OP_COLS} FROM notes WHERE id = ?1"), [id], op_from_row).optional()?)
}

/// Insert or replace the row for `op.id` with exactly `op`.
fn write(conn: &Connection, op: &NoteOp, dirty: bool) -> Result<()> {
    conn.execute(
        "INSERT INTO notes (id, hlc, base_hlc, title, body, pinned, created_at, updated_at, deleted_at, conflict_of, dirty)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
         ON CONFLICT (id) DO UPDATE SET
           hlc = excluded.hlc, base_hlc = excluded.base_hlc, title = excluded.title, body = excluded.body,
           pinned = excluded.pinned, created_at = excluded.created_at, updated_at = excluded.updated_at,
           deleted_at = excluded.deleted_at, conflict_of = excluded.conflict_of, dirty = excluded.dirty",
        params![
            op.id,
            op.hlc,
            op.base_hlc,
            op.title,
            op.body,
            op.pinned,
            op.created_at,
            op.updated_at,
            op.deleted_at,
            op.conflict_of,
            dirty
        ],
    )?;
    Ok(())
}

/// Merge one remote version into the local database.
pub fn apply_remote(conn: &Connection, clock: &Clock, remote: &NoteOp) -> Result<Applied> {
    clock.observe(&remote.hlc.parse::<Hlc>()?);
    let Some(local) = load(conn, &remote.id)? else {
        write(conn, remote, false)?;
        return Ok(Applied::Inserted);
    };

    if remote.hlc == local.hlc {
        return Ok(Applied::Skipped);
    }
    if remote.hlc > local.hlc && remote.descends_from(&local.hlc) {
        write(conn, remote, false)?;
        return Ok(Applied::FastForwarded);
    }
    if remote.hlc < local.hlc && local.descends_from(&remote.hlc) {
        return Ok(Applied::Skipped);
    }
    resolve_concurrent(conn, clock, local, remote)
}

fn resolve_concurrent(conn: &Connection, clock: &Clock, local: NoteOp, remote: &NoteOp) -> Result<Applied> {
    let remote_newer = remote.hlc > local.hlc;
    let (newer, older) = if remote_newer { (remote, &local) } else { (&local, remote) };
    let seen = local.hlc.as_str().max(remote.hlc.as_str()).to_owned();

    // A version with a fresh HLC above both, so every device fast-forwards to it.
    let republish = |mut state: NoteOp| -> Result<()> {
        state.hlc = clock.now().to_string();
        state.base_hlc = Some(seen.clone());
        write(conn, &state, true)
    };
    // Keep `winner` as the note's state, telling peers it has seen both sides.
    let keep = |winner: &NoteOp| -> Result<()> {
        if std::ptr::eq(winner, remote) {
            write(conn, remote, false)
        } else {
            let mut kept = local.clone();
            kept.base_hlc = Some(remote.hlc.clone());
            write(conn, &kept, true)
        }
    };

    if local.same_content(remote) {
        keep(newer)?;
        return Ok(Applied::Merged);
    }

    match (local.deleted_at.is_some(), remote.deleted_at.is_some()) {
        // Edit beats delete, even when the delete is newer.
        (false, true) | (true, false) => {
            let edit = if remote.deleted_at.is_none() { remote } else { &local };
            if std::ptr::eq(edit, newer) {
                keep(edit)?;
            } else {
                republish(edit.clone())?;
            }
            Ok(Applied::Merged)
        }
        (true, true) => {
            keep(newer)?;
            Ok(Applied::Merged)
        }
        (false, false) if local.same_text(remote) => {
            keep(newer)?;
            Ok(Applied::Merged)
        }
        (false, false) => {
            keep(newer)?;
            let copy_id = ids::derived_id(&[&local.id, &older.hlc]);
            let copy = NoteOp {
                id: copy_id.clone(),
                hlc: older.hlc.clone(),
                base_hlc: None,
                title: older.title.clone(),
                body: older.body.clone(),
                pinned: false,
                created_at: older.created_at,
                updated_at: older.updated_at,
                deleted_at: None,
                conflict_of: Some(local.id.clone()),
            };
            if load(conn, &copy_id)?.is_none() {
                write(conn, &copy, true)?;
            }
            Ok(Applied::Conflict { copy_id })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::{self, tests::setup};
    use crate::Note;

    struct Device {
        db: qd_core::Db,
        clock: Clock,
    }

    fn device(name: &str) -> Device {
        let (db, _) = setup();
        Device { db, clock: Clock::new(name) }
    }

    impl Device {
        fn create(&self, body: &str) -> Note {
            repo::create(&self.db.conn().unwrap(), &self.clock, body).unwrap()
        }
        fn edit(&self, id: &str, body: &str) {
            repo::update(&self.db.conn().unwrap(), &self.clock, id, Some(body), None).unwrap();
        }
        fn delete(&self, id: &str) {
            repo::delete(&self.db.conn().unwrap(), &self.clock, id).unwrap();
        }
        /// Push: export dirty ops and mark them clean.
        fn push(&self) -> Vec<NoteOp> {
            let conn = self.db.conn().unwrap();
            let ops = dirty_ops(&conn).unwrap();
            mark_pushed(&conn, &ops).unwrap();
            ops
        }
        fn pull(&self, ops: &[NoteOp]) -> Vec<Applied> {
            let conn = self.db.conn().unwrap();
            ops.iter().map(|op| apply_remote(&conn, &self.clock, op).unwrap()).collect()
        }
        fn visible(&self) -> Vec<(String, String, Option<String>)> {
            let mut v: Vec<_> = repo::list(&self.db.conn().unwrap(), 1000)
                .unwrap()
                .into_iter()
                .map(|n| (n.id, n.body, n.conflict_of))
                .collect();
            v.sort();
            v
        }
    }

    /// Exchange until nobody has anything left to push.
    fn settle(devices: &[&Device]) {
        for _ in 0..10 {
            let batches: Vec<Vec<NoteOp>> = devices.iter().map(|d| d.push()).collect();
            if batches.iter().all(Vec::is_empty) {
                return;
            }
            for (i, d) in devices.iter().enumerate() {
                for (j, ops) in batches.iter().enumerate() {
                    if i != j {
                        d.pull(ops);
                    }
                }
            }
        }
        panic!("did not settle");
    }

    #[test]
    fn new_note_propagates_and_echo_is_skipped() {
        let (a, b) = (device("a"), device("b"));
        let n = a.create("hello");
        let ops = a.push();
        assert_eq!(b.pull(&ops), vec![Applied::Inserted]);
        assert_eq!(b.pull(&ops), vec![Applied::Skipped]);
        assert_eq!(b.visible(), vec![(n.id, "hello".into(), None)]);
    }

    #[test]
    fn sequential_edit_fast_forwards() {
        let (a, b) = (device("a"), device("b"));
        let n = a.create("v1");
        b.pull(&a.push());
        b.edit(&n.id, "v2");
        assert_eq!(a.pull(&b.push()), vec![Applied::FastForwarded]);
        assert_eq!(a.visible(), b.visible());
        assert_eq!(a.visible()[0].1, "v2");
    }

    #[test]
    fn concurrent_edits_keep_both_bodies_on_every_device() {
        let (a, b) = (device("a"), device("b"));
        let n = a.create("base");
        b.pull(&a.push());
        a.edit(&n.id, "from a");
        b.edit(&n.id, "from b");
        settle(&[&a, &b]);

        assert_eq!(a.visible(), b.visible());
        let bodies: Vec<String> = a.visible().into_iter().map(|v| v.1).collect();
        assert_eq!(bodies.len(), 2, "{bodies:?}");
        assert!(bodies.contains(&"from a".into()) && bodies.contains(&"from b".into()));
        assert_eq!(a.visible().iter().filter(|v| v.2.as_deref() == Some(n.id.as_str())).count(), 1);
    }

    #[test]
    fn concurrent_edits_pushed_before_pulling_still_conflict() {
        // The case a dirty-flag based check would miss: both sides are clean.
        let (a, b) = (device("a"), device("b"));
        let n = a.create("base");
        b.pull(&a.push());
        a.edit(&n.id, "from a");
        b.edit(&n.id, "from b");
        let (pa, pb) = (a.push(), b.push());
        a.pull(&pb);
        b.pull(&pa);
        settle(&[&a, &b]);

        assert_eq!(a.visible(), b.visible());
        assert_eq!(a.visible().len(), 2);
    }

    #[test]
    fn concurrent_title_edits_conflict_too() {
        let (a, b) = (device("a"), device("b"));
        let n = a.create("same body");
        b.pull(&a.push());
        let retitle = |d: &Device, t: &str| {
            repo::update_note(
                &d.db.conn().unwrap(),
                &d.clock,
                &n.id,
                &repo::NotePatch { title: Some(t), ..Default::default() },
            )
            .unwrap();
        };
        retitle(&a, "Title A");
        retitle(&b, "Title B");
        settle(&[&a, &b]);
        let titles = |d: &Device| -> Vec<String> {
            let mut v: Vec<String> =
                repo::list(&d.db.conn().unwrap(), 10).unwrap().into_iter().map(|n| n.title).collect();
            v.sort();
            v
        };
        assert_eq!(titles(&a), titles(&b));
        assert_eq!(titles(&a), vec!["Title A", "Title B"]);
    }

    #[test]
    fn ops_without_title_from_older_versions_still_apply() {
        let b = device("b");
        let json = r#"{"id":"n1","hlc":"0000000001000-000000-a","baseHlc":null,"body":"old client","pinned":false,
                      "createdAt":1,"updatedAt":1,"deletedAt":null,"conflictOf":null}"#;
        let op: NoteOp = serde_json::from_str(json).unwrap();
        assert_eq!(b.pull(&[op]), vec![Applied::Inserted]);
        assert_eq!(b.visible()[0].1, "old client");
    }

    #[test]
    fn identical_concurrent_edits_merge_without_copy() {
        let (a, b) = (device("a"), device("b"));
        let n = a.create("base");
        b.pull(&a.push());
        a.edit(&n.id, "same");
        b.edit(&n.id, "same");
        settle(&[&a, &b]);
        assert_eq!(a.visible(), b.visible());
        assert_eq!(a.visible().len(), 1);
    }

    #[test]
    fn edit_beats_newer_delete() {
        let (a, b) = (device("a"), device("b"));
        let n = a.create("keep me");
        b.pull(&a.push());
        a.edit(&n.id, "edited");
        b.delete(&n.id); // later HLC than the edit
        settle(&[&a, &b]);
        assert_eq!(a.visible(), b.visible());
        assert_eq!(a.visible(), vec![(n.id, "edited".into(), None)]);
    }

    #[test]
    fn delete_propagates_when_not_concurrent() {
        let (a, b) = (device("a"), device("b"));
        let n = a.create("bye");
        b.pull(&a.push());
        b.delete(&n.id);
        settle(&[&a, &b]);
        assert!(a.visible().is_empty() && b.visible().is_empty());
    }

    /// xorshift64*: deterministic, dependency-free randomness for the fuzz test.
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 >> 12;
            self.0 ^= self.0 << 25;
            self.0 ^= self.0 >> 27;
            self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
        }
        fn below(&mut self, n: usize) -> usize {
            (self.next() % n as u64) as usize
        }
    }

    #[test]
    fn random_histories_converge() {
        for seed in 1..=300u64 {
            let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
            let devs = [device("a"), device("b"), device("c")];
            // Ops each device has published but a given peer has not received yet.
            let mut inbox: Vec<Vec<NoteOp>> = vec![vec![], vec![], vec![]];
            for step in 0..40 {
                let d = rng.below(3);
                let dev = &devs[d];
                let all: Vec<(String, bool)> = {
                    let conn = dev.db.conn().unwrap();
                    all_ops(&conn).unwrap().into_iter().map(|o| (o.id, o.deleted_at.is_some())).collect()
                };
                match rng.below(7) {
                    0 => drop(dev.create(&format!("n{seed}-{step}"))),
                    1 | 2 if !all.is_empty() => {
                        let (id, deleted) = &all[rng.below(all.len())];
                        if !deleted {
                            dev.edit(id, &format!("e{seed}-{step}-{d}"));
                        }
                    }
                    3 if !all.is_empty() => {
                        let (id, deleted) = &all[rng.below(all.len())];
                        let conn = dev.db.conn().unwrap();
                        if *deleted {
                            repo::restore(&conn, &dev.clock, id).unwrap();
                        } else {
                            repo::delete(&conn, &dev.clock, id).unwrap();
                        }
                    }
                    4 if !all.is_empty() => {
                        let (id, deleted) = &all[rng.below(all.len())];
                        if !deleted {
                            let conn = dev.db.conn().unwrap();
                            let pinned = repo::get(&conn, id).unwrap().pinned;
                            repo::update(&conn, &dev.clock, id, None, Some(!pinned)).unwrap();
                        }
                    }
                    5 => {
                        let ops = dev.push();
                        for (peer, inbox) in inbox.iter_mut().enumerate() {
                            if peer != d {
                                inbox.extend(ops.iter().cloned());
                            }
                        }
                    }
                    _ => {
                        // Deliver a random prefix of this device's inbox (partial sync).
                        let n = rng.below(inbox[d].len() + 1);
                        let batch: Vec<NoteOp> = inbox[d].drain(..n).collect();
                        dev.pull(&batch);
                    }
                }
            }
            for (d, ops) in inbox.iter().enumerate() {
                devs[d].pull(ops);
            }
            settle(&[&devs[0], &devs[1], &devs[2]]);
            let pinned = |dev: &Device| -> Vec<(String, bool)> {
                repo::list(&dev.db.conn().unwrap(), 1000).unwrap().into_iter().map(|n| (n.id, n.pinned)).collect()
            };
            assert_eq!(devs[0].visible(), devs[1].visible(), "seed {seed}: a vs b");
            assert_eq!(devs[1].visible(), devs[2].visible(), "seed {seed}: b vs c");
            let mut p0 = pinned(&devs[0]);
            let mut p2 = pinned(&devs[2]);
            p0.sort();
            p2.sort();
            assert_eq!(p0, p2, "seed {seed}: pinned flags");
        }
    }

    #[test]
    fn three_devices_converge_regardless_of_delivery_order() {
        let (a, b, c) = (device("a"), device("b"), device("c"));
        let n = a.create("base");
        let ops = a.push();
        b.pull(&ops);
        c.pull(&ops);
        a.edit(&n.id, "a1");
        b.edit(&n.id, "b1");
        c.delete(&n.id);
        let (pa, pb, pc) = (a.push(), b.push(), c.push());
        a.pull(&pc);
        a.pull(&pb);
        b.pull(&pa);
        b.pull(&pc);
        c.pull(&pb);
        c.pull(&pa);
        settle(&[&a, &b, &c]);

        assert_eq!(a.visible(), b.visible());
        assert_eq!(b.visible(), c.visible());
        let bodies: Vec<String> = a.visible().into_iter().map(|v| v.1).collect();
        assert!(bodies.contains(&"a1".into()) && bodies.contains(&"b1".into()), "{bodies:?}");
    }
}
