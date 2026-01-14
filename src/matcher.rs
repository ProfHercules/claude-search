use nucleo_matcher::{
    Config, Matcher, Utf32Str,
    pattern::{CaseMatching, Normalization, Pattern},
};

pub struct FuzzyMatcher {
    matcher: Matcher,
}

impl FuzzyMatcher {
    pub fn new() -> Self {
        // Config optimized for file path matching
        Self {
            matcher: Matcher::new(Config::DEFAULT.match_paths()),
        }
    }

    /// Match paths against pattern, return top N sorted by score (descending).
    pub fn match_paths(&mut self, paths: Vec<String>, pattern: &str, limit: usize) -> Vec<String> {
        if pattern.is_empty() {
            // No pattern - return first N paths as-is
            return paths.into_iter().take(limit).collect();
        }

        // Parse pattern with smart case matching
        let pat = Pattern::parse(pattern, CaseMatching::Smart, Normalization::Smart);

        // Score each path
        let mut scored: Vec<(String, u32)> = paths
            .into_iter()
            .filter_map(|path| {
                let mut buf = Vec::new();
                let haystack = Utf32Str::new(&path, &mut buf);
                pat.score(haystack, &mut self.matcher)
                    .map(|score| (path, score))
            })
            .collect();

        // Sort by score descending
        scored.sort_by(|a, b| b.1.cmp(&a.1));

        // Take top N
        scored
            .into_iter()
            .take(limit)
            .map(|(path, _)| path)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_pattern_returns_first_n() {
        let mut matcher = FuzzyMatcher::new();
        let paths = vec![
            "a.txt".to_string(),
            "b.txt".to_string(),
            "c.txt".to_string(),
        ];

        let results = matcher.match_paths(paths, "", 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], "a.txt");
        assert_eq!(results[1], "b.txt");
    }

    #[test]
    fn test_exact_match_ranked_high() {
        let mut matcher = FuzzyMatcher::new();
        let paths = vec![
            "something_main_else.rs".to_string(),
            "main.rs".to_string(),
            "mainly.rs".to_string(),
        ];

        let results = matcher.match_paths(paths, "main.rs", 10);
        // Exact match should be first
        assert_eq!(results[0], "main.rs");
    }

    #[test]
    fn test_path_matching() {
        let mut matcher = FuzzyMatcher::new();
        let paths = vec![
            "src/main.rs".to_string(),
            "tests/main_test.rs".to_string(),
            "docs/main.md".to_string(),
        ];

        let results = matcher.match_paths(paths, "src/main", 10);
        assert_eq!(results[0], "src/main.rs");
    }

    #[test]
    fn test_fuzzy_matching() {
        let mut matcher = FuzzyMatcher::new();
        let paths = vec![
            "configuration.rs".to_string(),
            "config.rs".to_string(),
            "constants.rs".to_string(),
        ];

        let results = matcher.match_paths(paths, "cfg", 10);
        // Both config files should match
        assert!(results.iter().any(|p| p == "config.rs"));
    }

    #[test]
    fn test_limit_respected() {
        let mut matcher = FuzzyMatcher::new();
        let paths: Vec<String> = (0..100).map(|i| format!("file{}.rs", i)).collect();

        let results = matcher.match_paths(paths, "file", 10);
        assert_eq!(results.len(), 10);
    }

    #[test]
    fn test_case_insensitive_by_default() {
        let mut matcher = FuzzyMatcher::new();
        let paths = vec!["README.md".to_string(), "readme.txt".to_string()];

        let results = matcher.match_paths(paths, "readme", 10);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_no_matches_returns_empty() {
        let mut matcher = FuzzyMatcher::new();
        let paths = vec!["foo.rs".to_string(), "bar.rs".to_string()];

        let results = matcher.match_paths(paths, "xyz123", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_partial_path_match() {
        let mut matcher = FuzzyMatcher::new();
        let paths = vec![
            "src/components/Button.tsx".to_string(),
            "src/components/Input.tsx".to_string(),
            "src/utils/helpers.ts".to_string(),
        ];

        let results = matcher.match_paths(paths, "comp/but", 10);
        assert!(!results.is_empty());
        assert!(results[0].contains("Button"));
    }
}
