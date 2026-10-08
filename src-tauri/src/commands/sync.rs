use qd_core::settings;
use qd_sync::crypto::KdfParams;
use qd_sync::engine::reset_local_state;
use qd_sync::{create_keyring, fetch_keyring, unlock, BlobTransport, Layout, S3Config, S3Transport};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use super::{blocking, CmdError, CmdResult};
use crate::secrets;
use crate::state::AppState;
use crate::sync::{self, Setup, SyncStatus, Trigger, CONFIG_KEY};

#[tauri::command]
pub fn sync_status(state: State<'_, AppState>) -> SyncStatus {
    state.sync.status()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectResult {
    /// A keyring already exists there: unlock instead of creating one.
    initialized: bool,
}

fn same_location(a: &S3Config, b: &S3Config) -> bool {
    (a.endpoint.trim(), a.bucket.trim(), a.prefix.trim_matches('/'))
        == (b.endpoint.trim(), b.bucket.trim(), b.prefix.trim_matches('/'))
}

/// Test the storage credentials and remember them.
#[tauri::command]
pub async fn sync_connect(app: AppHandle, config: S3Config, secret: String) -> CmdResult<ConnectResult> {
    blocking(move || {
        for (name, v) in
            [("endpoint", &config.endpoint), ("bucket", &config.bucket), ("access key id", &config.access_key_id)]
        {
            if v.trim().is_empty() {
                return Err(CmdError::new("invalid", format!("{name} is required")));
            }
        }
        if secret.trim().is_empty() {
            return Err(CmdError::new("invalid", "secret access key is required"));
        }
        let transport = S3Transport::new(&config, &secret)?;
        let layout = Layout::new(&config.prefix);
        // Fails on bad credentials, a missing bucket or no network.
        transport.list(&layout.keyring(), None)?;
        let initialized = fetch_keyring(&transport, &layout)?.is_some();

        let state = app.state::<AppState>();
        {
            let conn = state.db.conn()?;
            let previous: Option<S3Config> = settings::get(&conn, CONFIG_KEY)?;
            if !previous.is_some_and(|p| same_location(&p, &config)) {
                reset_local_state(&conn)?;
                secrets::delete(secrets::SYNC_DEK).map_err(|e| CmdError::new("keyring", e))?;
            }
            settings::set(&conn, CONFIG_KEY, &config)?;
        }
        secrets::set(secrets::S3_SECRET, &secret).map_err(|e| CmdError::new("keyring", e))?;
        sync::refresh_status(&app);
        Ok(ConnectResult { initialized })
    })
    .await
}

fn locked_transport(app: &AppHandle) -> CmdResult<(S3Config, S3Transport)> {
    match sync::load(app).map_err(|e| CmdError::new("internal", e))? {
        Setup::Disabled => Err(CmdError::new("invalid", "connect storage first")),
        Setup::Locked { config, transport } | Setup::Unlocked { config, transport, .. } => Ok((config, transport)),
    }
}

fn store_key(app: &AppHandle, dek: &qd_sync::crypto::Dek) -> CmdResult<()> {
    secrets::set(secrets::SYNC_DEK, &dek.to_base64()).map_err(|e| CmdError::new("keyring", e))?;
    sync::refresh_status(app);
    app.state::<AppState>().sync.trigger(Trigger::Now);
    Ok(())
}

/// First device: create the encryption keys. Returns the recovery key to show once.
#[tauri::command]
pub async fn sync_create(app: AppHandle, passphrase: String) -> CmdResult<String> {
    blocking(move || {
        let (config, transport) = locked_transport(&app)?;
        let (dek, recovery) =
            create_keyring(&transport, &Layout::new(&config.prefix), &passphrase, KdfParams::default())?;
        store_key(&app, &dek)?;
        Ok(recovery.display())
    })
    .await
}

/// Further devices: unlock with the passphrase or recovery key.
#[tauri::command]
pub async fn sync_unlock(app: AppHandle, secret: String) -> CmdResult<()> {
    blocking(move || {
        let (config, transport) = locked_transport(&app)?;
        let dek = unlock(&transport, &Layout::new(&config.prefix), &secret)?;
        store_key(&app, &dek)
    })
    .await
}

#[tauri::command]
pub fn sync_now(state: State<'_, AppState>) {
    state.sync.trigger(Trigger::Now);
}

/// Stop syncing and forget credentials. Local notes are kept.
#[tauri::command]
pub async fn sync_disconnect(app: AppHandle) -> CmdResult<()> {
    blocking(move || {
        let state = app.state::<AppState>();
        {
            let conn = state.db.conn()?;
            conn.execute("DELETE FROM settings WHERE key = ?1", [CONFIG_KEY]).map_err(qd_core::Error::from)?;
            reset_local_state(&conn)?;
        }
        for name in [secrets::S3_SECRET, secrets::SYNC_DEK] {
            secrets::delete(name).map_err(|e| CmdError::new("keyring", e))?;
        }
        sync::refresh_status(&app);
        Ok(())
    })
    .await
}
