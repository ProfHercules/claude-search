use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct Input {
    pub query: Option<String>,
    pub cwd: Option<String>,
}

#[derive(Debug)]
pub struct ParsedQuery {
    /// The actual pattern to match (e.g., "main" from "../src/main")
    pub pattern: String,
    /// Directory to search from
    pub search_base: PathBuf,
    /// Prefix to prepend to output paths (e.g., "../" or "../../")
    pub output_prefix: String,
    /// Whether this is an empty query (shallow listing mode)
    pub is_empty: bool,
}

/// Parse a query string and extract the ../ prefix chain.
///
/// Examples:
/// - "main.rs" -> pattern="main.rs", prefix="", search from cwd
/// - "../foo" -> pattern="foo", prefix="../", search from cwd/..
/// - "../../bar" -> pattern="bar", prefix="../../", search from cwd/../..
/// - "./src" -> pattern="src", prefix="", search from cwd
pub fn parse_query(raw_query: &str, cwd: &Path) -> ParsedQuery {
    let query = raw_query.trim();

    // Count and extract ../ prefix chain
    let mut prefix_count = 0;
    let mut remaining = query;

    // Handle leading ./
    if let Some(stripped) = remaining.strip_prefix("./") {
        remaining = stripped;
    }

    // Count ../ prefixes
    while let Some(stripped) = remaining.strip_prefix("../") {
        prefix_count += 1;
        remaining = stripped;
    }

    // Handle trailing .. without /
    if remaining == ".." {
        prefix_count += 1;
        remaining = "";
    }

    // Build output prefix (e.g., "../../")
    let output_prefix = "../".repeat(prefix_count);

    // Build search base by going up directories
    let mut search_base = cwd.to_path_buf();
    for _ in 0..prefix_count {
        if let Some(parent) = search_base.parent() {
            search_base = parent.to_path_buf();
        }
    }

    // Strip leading ./ from remaining pattern
    let pattern = remaining
        .strip_prefix("./")
        .unwrap_or(remaining)
        .to_string();

    ParsedQuery {
        is_empty: pattern.is_empty(),
        pattern,
        search_base,
        output_prefix,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_query() {
        let parsed = parse_query("", Path::new("/home/user/project"));
        assert_eq!(parsed.pattern, "");
        assert_eq!(parsed.output_prefix, "");
        assert!(parsed.is_empty);
        assert_eq!(parsed.search_base, Path::new("/home/user/project"));
    }

    #[test]
    fn test_simple_query() {
        let parsed = parse_query("main.rs", Path::new("/home/user/project"));
        assert_eq!(parsed.pattern, "main.rs");
        assert_eq!(parsed.output_prefix, "");
        assert!(!parsed.is_empty);
    }

    #[test]
    fn test_path_query() {
        let parsed = parse_query("src/main", Path::new("/home/user/project"));
        assert_eq!(parsed.pattern, "src/main");
        assert_eq!(parsed.output_prefix, "");
    }

    #[test]
    fn test_single_parent_prefix() {
        let parsed = parse_query("../foo", Path::new("/home/user/project"));
        assert_eq!(parsed.pattern, "foo");
        assert_eq!(parsed.output_prefix, "../");
        assert_eq!(parsed.search_base, Path::new("/home/user"));
    }

    #[test]
    fn test_double_parent_prefix() {
        let parsed = parse_query("../../bar", Path::new("/home/user/project"));
        assert_eq!(parsed.pattern, "bar");
        assert_eq!(parsed.output_prefix, "../../");
        assert_eq!(parsed.search_base, Path::new("/home"));
    }

    #[test]
    fn test_parent_prefix_with_path() {
        let parsed = parse_query("../src/main", Path::new("/home/user/project"));
        assert_eq!(parsed.pattern, "src/main");
        assert_eq!(parsed.output_prefix, "../");
    }

    #[test]
    fn test_current_dir_prefix_stripped() {
        let parsed = parse_query("./src/main", Path::new("/home/user/project"));
        assert_eq!(parsed.pattern, "src/main");
        assert_eq!(parsed.output_prefix, "");
    }

    #[test]
    fn test_only_parent_ref() {
        let parsed = parse_query("..", Path::new("/home/user/project"));
        assert_eq!(parsed.pattern, "");
        assert_eq!(parsed.output_prefix, "../");
        assert!(parsed.is_empty);
        assert_eq!(parsed.search_base, Path::new("/home/user"));
    }

    #[test]
    fn test_only_parent_ref_with_slash() {
        let parsed = parse_query("../", Path::new("/home/user/project"));
        assert_eq!(parsed.pattern, "");
        assert_eq!(parsed.output_prefix, "../");
        assert!(parsed.is_empty);
    }

    #[test]
    fn test_json_deserialization() {
        let json = r#"{"query": "src/main", "cwd": "/home/user"}"#;
        let input: Input = serde_json::from_str(json).unwrap();
        assert_eq!(input.query, Some("src/main".to_string()));
        assert_eq!(input.cwd, Some("/home/user".to_string()));
    }

    #[test]
    fn test_json_missing_fields() {
        let json = r#"{}"#;
        let input: Input = serde_json::from_str(json).unwrap();
        assert_eq!(input.query, None);
        assert_eq!(input.cwd, None);
    }

    #[test]
    fn test_whitespace_trimmed() {
        let parsed = parse_query("  main.rs  ", Path::new("/home/user/project"));
        assert_eq!(parsed.pattern, "main.rs");
    }
}
