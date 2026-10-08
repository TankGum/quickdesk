//! Live test against the session's X server / Xwayland:
//! `cargo test -p qd-platform --test x11_clipboard -- --ignored --test-threads=1`
#![cfg(target_os = "linux")]

use std::sync::mpsc;
use std::time::Duration;

use arboard::{Clipboard, ImageData, SetExtLinux};
use qd_platform::clipboard::{spawn_watcher, ClipContent, WatcherState};

fn recv_text(rx: &mpsc::Receiver<String>, within: Duration) -> Option<String> {
    rx.recv_timeout(within).ok()
}

#[test]
#[ignore = "needs a live X11/Xwayland display"]
fn watcher_sees_copies_skips_secrets_and_reads_large_selections() {
    let (tx, rx) = mpsc::channel();
    let (any_tx, any_rx) = mpsc::channel::<ClipContent>();
    let handle = spawn_watcher(move |ev| match ev.content {
        ClipContent::Text(t) => tx.send(t).unwrap(),
        other => any_tx.send(other).unwrap(),
    });
    for _ in 0..50 {
        if handle.state() == WatcherState::Running {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(handle.state(), WatcherState::Running);
    let mut cb = Clipboard::new().unwrap();
    let wait = Duration::from_secs(3);

    cb.set_text("hello from quickdesk").unwrap();
    assert_eq!(recv_text(&rx, wait).as_deref(), Some("hello from quickdesk"));

    // Password managers mark secrets; the watcher must not report them.
    cb.set().exclude_from_history().text("hunter2").unwrap();
    assert_eq!(recv_text(&rx, Duration::from_millis(800)), None, "secret leaked");

    // Larger than a single X request: exercises the INCR protocol.
    let big = "x".repeat(600_000);
    cb.set_text(big.clone()).unwrap();
    assert_eq!(recv_text(&rx, wait).map(|t| t.len()), Some(big.len()));

    // A burst of copies coalesces to the last one.
    for i in 0..10 {
        cb.set_text(format!("burst {i}")).unwrap();
    }
    assert_eq!(recv_text(&rx, wait).as_deref(), Some("burst 9"));
    assert_eq!(recv_text(&rx, Duration::from_millis(500)), None, "burst not coalesced");

    // Images arrive as encoded PNG.
    let pixels: Vec<u8> = (0..4 * 3 * 2).map(|i| (i * 20) as u8).collect();
    cb.set_image(ImageData { width: 3, height: 2, bytes: pixels.into() }).unwrap();
    match any_rx.recv_timeout(wait).expect("image event") {
        ClipContent::Image { mime, bytes } => {
            assert_eq!(mime, "image/png");
            assert!(bytes.starts_with(b"\x89PNG"), "png signature");
        }
        other => panic!("expected image, got {other:?}"),
    }

    // Copied files come back as paths.
    let dir = std::env::temp_dir();
    let (a, b) = (dir.join("qd test a.txt"), dir.join("qd-test-b.txt"));
    // Must exist: copying a file list resolves the real paths.
    std::fs::write(&a, "a").unwrap();
    std::fs::write(&b, "b").unwrap();
    let (a, b) = (a.canonicalize().unwrap(), b.canonicalize().unwrap());
    cb.set().file_list(&[&a, &b]).unwrap();
    match any_rx.recv_timeout(wait).expect("files event") {
        ClipContent::Files(paths) => assert_eq!(paths, vec![a.display().to_string(), b.display().to_string()]),
        other => panic!("expected files, got {other:?}"),
    }
    let _ = (std::fs::remove_file(&a), std::fs::remove_file(&b));
}
