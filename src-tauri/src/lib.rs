mod ai_usage;
mod cli;
mod clipboard;
mod commands;
mod hotkeys;
mod i18n;
pub mod ipc;
mod ring;
mod secrets;
mod state;
mod sync;
mod tray;
mod windows;

use std::sync::{Mutex, OnceLock};

use qd_core::{settings, Clock, Db};
use qd_platform::Session;
use tauri::{AppHandle, Manager, RunEvent, Runtime};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;

use crate::cli::{CliCommand, Target};
use crate::hotkeys::HotkeyConfig;
use crate::ipc::{now_ms, Forwarded};
use crate::state::AppState;

static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

pub fn run(args: Vec<String>) {
    let initial = match CliCommand::parse(&args) {
        Ok(CliCommand::Quit) => return, // nothing running to quit
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            dispatch(app, Forwarded { args: argv.into_iter().skip(1).collect(), sent_at_ms: now_ms() });
        }))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::app::app_info,
            commands::app::diag_focus_report,
            commands::app::diag_focus_stats,
            commands::app::app_quit,
            commands::app::app_show,
            commands::app::clipboard_write,
            commands::hotkeys::hotkeys_get,
            commands::hotkeys::hotkeys_check,
            commands::hotkeys::hotkeys_set,
            commands::hotkeys::hotkeys_reset,
            commands::hotkeys::hotkeys_suspend,
            commands::hotkeys::hotkeys_resume,
            commands::clipboard::clip_list,
            commands::clipboard::clip_search,
            commands::clipboard::clip_copy,
            commands::clipboard::clip_pin,
            commands::clipboard::clip_delete,
            commands::clipboard::clip_clear,
            commands::clipboard::clip_status,
            commands::clipboard::clip_set_paused,
            commands::clipboard::clip_paste,
            commands::clipboard::clip_set_auto_paste,
            commands::clipboard::clip_paste_info,
            commands::clipboard::clip_set_paste_method,
            commands::clipboard::clip_uinput_enable,
            commands::clipboard::clip_uinput_disable,
            commands::app::app_onboarding,
            commands::app::app_get_language,
            commands::ai::ai_usage_get,
            commands::ai::ai_usage_refresh,
            commands::ai::ai_set_tray,
            commands::app::app_set_language,
            commands::app::app_onboarding_finish,
            commands::ports::ports_scan,
            commands::ports::ports_kill,
            commands::ports::ports_is_alive,
            commands::ports::ports_stop_container,
            commands::ports::ports_open,
            commands::sync::sync_status,
            commands::sync::sync_connect,
            commands::sync::sync_create,
            commands::sync::sync_unlock,
            commands::sync::sync_now,
            commands::sync::sync_disconnect,
            commands::notes::notes_create,
            commands::notes::notes_update,
            commands::notes::notes_delete,
            commands::notes::notes_restore,
            commands::notes::notes_list,
            commands::notes::notes_search,
        ])
        .on_window_event(windows::on_window_event)
        .setup(move |app| {
            init_logging(app.path().app_log_dir()?);
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let db =
                Db::open(&data_dir.join("quickdesk.db"), &[&qd_notes::NotesModule, &qd_clipboard::ClipboardModule])?;
            let (device_id, hotkeys, clip, lang_pref, ai) = {
                let conn = db.conn()?;
                (
                    settings::device_id(&conn)?,
                    HotkeyConfig::load(&conn)?,
                    clipboard::ClipboardService::new(&conn),
                    settings::get::<i18n::LangPref>(&conn, i18n::SETTINGS_KEY)?.unwrap_or_default(),
                    ai_usage::AiUsageService::new(&conn),
                )
            };
            let clip_paused = clip.is_paused();
            let session = Session::detect();
            let strategy = session.hotkey_strategy();
            tracing::info!(?session, ?strategy, data_dir = %data_dir.display(), "starting");

            if strategy == qd_platform::HotkeyStrategy::Plugin {
                app.handle().plugin(tauri_plugin_global_shortcut::Builder::new().build())?;
            }
            if let Err(e) = hotkeys::register(app.handle(), strategy, &hotkeys) {
                tracing::error!(error = %e, "hotkey registration failed");
            }

            app.manage(AppState {
                db,
                clock: Clock::new(device_id.clone()),
                data_dir,
                device_id,
                session,
                strategy,
                hotkeys: Mutex::new(hotkeys),
                focus_reports: Mutex::new(Vec::new()),
                clipboard: clip,
                sync: sync::SyncService::new(),
                lang_pref: Mutex::new(lang_pref),
                ai,
            });

            tray::build(app, clip_paused)?;
            clipboard::start(app.handle());
            sync::start(app.handle());
            ai_usage::start(app.handle());

            let handle = app.handle().clone();
            ipc::serve(move |msg| dispatch(&handle, msg))?;

            run_command(app.handle(), initial, now_ms());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to build QuickDesk");

    app.run(|app, event| {
        if let RunEvent::Exit = event {
            shutdown(app);
        }
    });
}

fn init_logging(dir: std::path::PathBuf) {
    let file = tracing_appender::rolling::daily(dir, "quickdesk.log");
    let (writer, guard) = tracing_appender::non_blocking(file);
    let _ = LOG_GUARD.set(guard);
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(if cfg!(debug_assertions) { "info,quickdesk_lib=debug" } else { "info" }));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).with_writer(writer).with_ansi(false).try_init();
}

fn dispatch<R: Runtime>(app: &AppHandle<R>, msg: Forwarded) {
    match CliCommand::parse(&msg.args) {
        Ok(cmd) => run_command(app, cmd, msg.sent_at_ms),
        Err(e) => tracing::warn!(error = %e, "ignored forwarded args"),
    }
}

fn run_command<R: Runtime>(app: &AppHandle<R>, cmd: CliCommand, sent_at_ms: u64) {
    tracing::debug!(?cmd, "command");
    match cmd {
        CliCommand::Launch => windows::show(app, Target::Main, sent_at_ms),
        CliCommand::Toggle(t) => windows::toggle(app, t, sent_at_ms),
        CliCommand::Show(t) => windows::show(app, t, sent_at_ms),
        CliCommand::Background => {}
        CliCommand::Quit => quit(app),
    }
}

pub(crate) fn quit<R: Runtime>(app: &AppHandle<R>) {
    app.exit(0);
}

fn shutdown<R: Runtime>(app: &AppHandle<R>) {
    if let Some(state) = app.try_state::<AppState>() {
        hotkeys::unregister(app, state.strategy);
    }
    ipc::cleanup();
    tracing::info!("stopped");
}
