use qd_core::settings;
use qd_sync::cloud::{self, Account};
use qd_sync::crypto::KdfParams;
use qd_sync::engine::reset_local_state;
use qd_sync::{create_keyring, fetch_keyring, unlock, BlobTransport};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use super::{blocking, CmdError, CmdResult};
use crate::secrets;
use crate::state::AppState;
use crate::sync::{self, CloudConfig, Setup, SyncStatus, Trigger, CONFIG_KEY, NOTICE_KEY};

#[tauri::command]
pub fn sync_status(state: State<'_, AppState>) -> SyncStatus {
    state.sync.status()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Enabled {
    /// What other devices enter to join; can be shown again later.
    sync_code: String,
    /// Shown once: unlocks the notes if the passphrase is forgotten.
    recovery_key: String,
}

fn keyring_err(e: String) -> CmdError {
    CmdError::new("keyring", e)
}

/// Remember the account on this device (token in the OS keyring).
fn remember(app: &AppHandle, endpoint: &str, account: &Account) -> CmdResult<()> {
    secrets::set(secrets::SYNC_TOKEN, &account.token).map_err(keyring_err)?;
    let state = app.state::<AppState>();
    let conn = state.db.conn()?;
    settings::set(&conn, CONFIG_KEY, &CloudConfig { account: account.account.clone(), endpoint: endpoint.to_owned() })?;
    conn.execute("DELETE FROM settings WHERE key = ?1", [NOTICE_KEY]).map_err(qd_core::Error::from)?;
    Ok(())
}

fn store_key(app: &AppHandle, dek: &qd_sync::crypto::Dek) -> CmdResult<()> {
    secrets::set(secrets::SYNC_DEK, &dek.to_base64()).map_err(keyring_err)?;
    sync::refresh_status(app);
    app.state::<AppState>().sync.trigger(Trigger::Now);
    Ok(())
}

/// First device: create a sync account in QuickDesk Cloud and its encryption
/// keys. Returns the sync code and the recovery key.
#[tauri::command]
pub async fn sync_enable(app: AppHandle, passphrase: String) -> CmdResult<Enabled> {
    blocking(move || {
        // Check before creating the account, so a short passphrase leaves nothing behind.
        if passphrase.chars().count() < qd_sync::crypto::MIN_PASSPHRASE_CHARS {
            return Err(CmdError::new(
                "invalid",
                format!("passphrase must be at least {} characters", qd_sync::crypto::MIN_PASSPHRASE_CHARS),
            ));
        }
        let endpoint = cloud::endpoint();
        let account = cloud::create_account(&endpoint)?;
        let transport = qd_sync::CloudTransport::new(&endpoint, &account);
        let (dek, recovery) = match create_keyring(&transport, &sync::layout(), &passphrase, KdfParams::default()) {
            Ok(keys) => keys,
            Err(e) => {
                // Do not leave an empty account behind.
                let _ = cloud::delete_account(&endpoint, &account);
                return Err(e.into());
            }
        };
        reset_local_state(&*app.state::<AppState>().db.conn()?)?;
        remember(&app, &endpoint, &account)?;
        store_key(&app, &dek)?;
        Ok(Enabled { sync_code: account.sync_code(), recovery_key: recovery.display() })
    })
    .await
}

/// Another device: join with the sync code and the passphrase (or recovery key).
#[tauri::command]
pub async fn sync_join(app: AppHandle, code: String, secret: String) -> CmdResult<()> {
    blocking(move || {
        let account = Account::from_sync_code(&code)?;
        let endpoint = cloud::endpoint();
        let transport = qd_sync::CloudTransport::new(&endpoint, &account);
        // Wrong or deleted codes fail here, before anything is stored.
        if fetch_keyring(&transport, &sync::layout())?.is_none() {
            return Err(CmdError::new("not_initialized", "this sync code has no data yet"));
        }
        let dek = unlock(&transport, &sync::layout(), &secret)?;
        reset_local_state(&*app.state::<AppState>().db.conn()?)?;
        remember(&app, &endpoint, &account)?;
        store_key(&app, &dek)
    })
    .await
}

/// This device knows the account but lost its key (e.g. the OS keyring was reset).
#[tauri::command]
pub async fn sync_unlock(app: AppHandle, secret: String) -> CmdResult<()> {
    blocking(move || {
        let Setup::Locked { config, account } = sync::load(&app).map_err(|e| CmdError::new("internal", e))? else {
            return Err(CmdError::new("invalid", "sync is not waiting to be unlocked"));
        };
        let dek = unlock(&Setup::transport(&config, &account), &sync::layout(), &secret)?;
        store_key(&app, &dek)
    })
    .await
}

/// The sync code, to add another device.
#[tauri::command]
pub fn sync_code(app: AppHandle) -> CmdResult<String> {
    match sync::load(&app).map_err(|e| CmdError::new("internal", e))? {
        Setup::Disabled => Err(CmdError::new("invalid", "sync is off")),
        Setup::Locked { account, .. } | Setup::Unlocked { account, .. } => Ok(account.sync_code()),
    }
}

#[tauri::command]
pub fn sync_now(state: State<'_, AppState>) {
    state.sync.trigger(Trigger::Now);
}

/// Stop syncing on this device. Local notes are kept. With `delete_cloud`, the
/// sync account and everything in it are deleted for every device.
#[tauri::command]
pub async fn sync_disconnect(app: AppHandle, delete_cloud: bool) -> CmdResult<()> {
    blocking(move || {
        if delete_cloud {
            if let Setup::Locked { config, account } | Setup::Unlocked { config, account, .. } =
                sync::load(&app).map_err(|e| CmdError::new("internal", e))?
            {
                cloud::delete_account(&config.endpoint, &account)?;
            }
        }
        let state = app.state::<AppState>();
        {
            let conn = state.db.conn()?;
            conn.execute("DELETE FROM settings WHERE key = ?1", [CONFIG_KEY]).map_err(qd_core::Error::from)?;
            reset_local_state(&conn)?;
        }
        for name in [secrets::SYNC_TOKEN, secrets::SYNC_DEK] {
            secrets::delete(name).map_err(keyring_err)?;
        }
        sync::refresh_status(&app);
        Ok(())
    })
    .await
}

/// Probe used by the UI before joining: is this a valid code with data?
#[tauri::command]
pub async fn sync_check_code(code: String) -> CmdResult<()> {
    blocking(move || {
        let account = Account::from_sync_code(&code)?;
        let transport = qd_sync::CloudTransport::new(&cloud::endpoint(), &account);
        transport.list(&sync::layout().keyring(), None)?;
        Ok(())
    })
    .await
}
