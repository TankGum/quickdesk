//! End-to-end encrypted sync of Quick Notes through QuickDesk Cloud (`cloud`).

pub mod cloud;
pub mod crypto;
pub mod engine;
pub mod transport;

pub use cloud::{Account, CloudTransport};
pub use engine::{create_keyring, fetch_keyring, unlock, EngineConfig, Layout, SyncEngine, SyncReport};
pub use transport::{BlobTransport, MemoryTransport};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("network error: {0}")]
    Network(String),
    #[error("storage rejected the credentials: {0}")]
    Auth(String),
    #[error("storage error: {0}")]
    Remote(String),
    #[error("{0}")]
    Crypto(String),
    #[error("wrong passphrase or recovery key")]
    WrongSecret,
    #[error("sync is already set up at this location; unlock it with your passphrase instead")]
    AlreadyInitialized,
    #[error("sync has not been set up at this location yet")]
    NotInitialized,
    #[error("{0}")]
    InvalidInput(String),
    #[error(transparent)]
    Core(#[from] qd_core::Error),
    #[error(transparent)]
    Notes(#[from] qd_notes::Error),
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

impl Error {
    /// Transient failures worth retrying with backoff.
    pub fn is_transient(&self) -> bool {
        matches!(self, Error::Network(_) | Error::Remote(_))
    }
}

pub type Result<T> = std::result::Result<T, Error>;
