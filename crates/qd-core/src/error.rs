use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("migration {module}#{version} failed: {source}")]
    Migration { module: &'static str, version: u32, source: rusqlite::Error },
    #[error("invalid HLC: {0:?}")]
    InvalidHlc(String),
    #[error("database lock poisoned")]
    Poisoned,
}

pub type Result<T> = std::result::Result<T, Error>;
