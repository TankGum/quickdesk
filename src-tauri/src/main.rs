// Prevents an additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // A release build on Windows has no console: a panic during startup
    // would close the app without a word. Keep a note of it instead.
    std::panic::set_hook(Box::new(|info| {
        let report = format!(
            "QuickDesk {} crashed: {info}\n\n{}\n",
            env!("CARGO_PKG_VERSION"),
            std::backtrace::Backtrace::force_capture()
        );
        eprintln!("{report}");
        let file = std::env::temp_dir().join("quickdesk-crash.log");
        let _ = std::fs::write(&file, &report);
        #[cfg(windows)]
        quickdesk_lib::crash_dialog(&format!("{info}\n\nDetails: {}", file.display()));
    }));

    let args: Vec<String> = std::env::args().skip(1).collect();
    // Hotkey path: hand off to the running instance before any GUI init.
    if quickdesk_lib::ipc::try_forward(&args) {
        return;
    }
    quickdesk_lib::run(args);
}
