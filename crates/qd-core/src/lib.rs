//! Shared foundation for QuickDesk modules: database, migrations, settings, ids and clocks.

pub mod db;
pub mod error;
pub mod hlc;
pub mod ids;
pub mod module;
pub mod settings;

pub use db::Db;
pub use error::{Error, Result};
pub use hlc::{Clock, Hlc};
pub use module::{Migration, Module};
