//! `cargo run -p qd-platform --example hotkey_conflicts -- Super+Alt+N Ctrl+Alt+V ...`
fn main() {
    let kb = qd_platform::gnome::GnomeKeybindings::new();
    for arg in std::env::args().skip(1) {
        match arg.parse::<qd_platform::Accelerator>() {
            Ok(a) => println!("{:<16} {:?}", a.to_string(), kb.conflicts(&a)),
            Err(e) => println!("{arg}: {e}"),
        }
    }
}
