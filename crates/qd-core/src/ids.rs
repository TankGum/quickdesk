use uuid::Uuid;

/// Time-ordered unique id (UUIDv7), used as primary key for synced records.
pub fn new_id() -> String {
    Uuid::now_v7().to_string()
}

/// Deterministic id derived from `parts`, so every device derives the same id
/// for the same logical record (e.g. a sync conflict copy).
pub fn derived_id(parts: &[&str]) -> String {
    Uuid::new_v5(&Uuid::NAMESPACE_OID, parts.join("\u{1f}").as_bytes()).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ids_are_unique_and_time_ordered() {
        let a = new_id();
        let b = new_id();
        assert_ne!(a, b);
        assert!(a < b, "{a} should sort before {b}");
    }

    #[test]
    fn derived_id_is_deterministic() {
        assert_eq!(derived_id(&["n1", "h1"]), derived_id(&["n1", "h1"]));
        assert_ne!(derived_id(&["n1", "h1"]), derived_id(&["n1h", "1"]));
    }
}
