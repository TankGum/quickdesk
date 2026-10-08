//! Quick Notes: local storage, full-text search and sync bookkeeping.

mod fts;
pub mod repo;
pub mod sync;

use qd_core::{Migration, Module};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use repo::*;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] qd_core::Error),
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("note {0} not found")]
    NotFound(String),
    #[error("note is empty")]
    EmptyBody,
}

pub type Result<T> = std::result::Result<T, Error>;

/// A note as shown to the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    /// May be empty; the UI then shows the first line of the body.
    pub title: String,
    pub body: String,
    pub pinned: bool,
    pub created_at: i64,
    pub updated_at: i64,
    /// Set when this note is the losing side of a sync conflict.
    pub conflict_of: Option<String>,
}

pub struct NotesModule;

impl Module for NotesModule {
    fn id(&self) -> &'static str {
        "notes"
    }

    fn migrations(&self) -> &'static [Migration] {
        &[
            Migration { version: 1, sql: include_str!("migrations/001_notes.sql") },
            Migration { version: 2, sql: include_str!("migrations/002_title.sql") },
        ]
    }
}
