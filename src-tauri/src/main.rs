// Prevents an additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // Hotkey path: hand off to the running instance before any GUI init.
    if quickdesk_lib::ipc::try_forward(&args) {
        return;
    }
    quickdesk_lib::run(args);
}
