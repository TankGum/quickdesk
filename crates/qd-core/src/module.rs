/// One forward-only schema step owned by a module.
#[derive(Debug, Clone, Copy)]
pub struct Migration {
    /// Strictly increasing per module, starting at 1.
    pub version: u32,
    pub sql: &'static str,
}

/// A feature module (notes, clipboard, ...) plugged into the core.
pub trait Module: Send + Sync {
    /// Stable identifier, also the key in `schema_migrations`.
    fn id(&self) -> &'static str;
    fn migrations(&self) -> &'static [Migration];
}
