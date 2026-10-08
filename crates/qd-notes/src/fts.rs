/// Turn free-form user input into a safe FTS5 query: every whitespace-separated
/// term becomes a quoted prefix match, all terms must match (implicit AND).
/// Returns `None` when the input has no searchable characters.
pub fn prefix_query(input: &str) -> Option<String> {
    let terms: Vec<String> = input
        .split_whitespace()
        .map(|t| {
            t.chars().filter(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | ':')).collect::<String>()
        })
        .filter(|t| t.chars().any(char::is_alphanumeric))
        .map(|t| format!("\"{t}\"*"))
        .collect();
    (!terms.is_empty()).then(|| terms.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_terms_and_drops_operators() {
        assert_eq!(prefix_query("pub sub").as_deref(), Some("\"pub\"* \"sub\"*"));
        assert_eq!(prefix_query("  \"NEAR( OR *").as_deref(), Some("\"NEAR\"* \"OR\"*"));
        assert_eq!(prefix_query("kubectl get-pods").as_deref(), Some("\"kubectl\"* \"get-pods\"*"));
        assert_eq!(prefix_query("ghi chú").as_deref(), Some("\"ghi\"* \"chú\"*"));
        assert_eq!(prefix_query(" * \" ( "), None);
    }
}
