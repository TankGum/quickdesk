//! Dev helper: `cargo run -p qd-platform --example set_clipboard -- "text" [--secret]`
//! Owns the X11 clipboard for a few seconds so watchers can read it.
fn main() {
    #[cfg(target_os = "linux")]
    {
        use arboard::SetExtLinux;
        let args: Vec<String> = std::env::args().skip(1).collect();
        let text = args.first().cloned().unwrap_or_else(|| "hello".into());
        let mut cb = arboard::Clipboard::new().expect("clipboard");
        if args.iter().any(|a| a == "--secret") {
            cb.set().exclude_from_history().text(text).expect("set");
        } else {
            cb.set_text(text).expect("set");
        }
        std::thread::sleep(std::time::Duration::from_secs(3));
    }
}
