use ignore::{DirEntry, WalkBuilder, WalkState};
use std::path::Path;
use std::sync::mpsc;

pub struct WalkConfig {
    pub max_depth: usize,
}

pub const SHALLOW_CONFIG: WalkConfig = WalkConfig { max_depth: 2 };
pub const DEEP_CONFIG: WalkConfig = WalkConfig { max_depth: 6 };

const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    ".venv",
    "__pycache__",
    ".mypy_cache",
    ".cache",
    "dist",
    "build",
    ".next",
    "target",
    ".tox",
    ".pytest_cache",
];

/// Check if entry should be skipped based on directory name
#[inline]
fn should_skip_entry(entry: &DirEntry) -> bool {
    if let Some(file_type) = entry.file_type() {
        if file_type.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                return SKIP_DIRS.contains(&name);
            }
        }
    }
    false
}

/// Check if path contains any skip directories
#[inline]
fn path_contains_skip_dir(path: &str) -> bool {
    for skip in SKIP_DIRS {
        if path.starts_with(skip) && path.as_bytes().get(skip.len()) == Some(&b'/') {
            return true;
        }
        if path.contains(&format!("/{}/", skip)) {
            return true;
        }
        if path == *skip {
            return true;
        }
    }
    false
}

/// Walk files in the given directory using parallel traversal.
/// Respects .gitignore and skips common directories.
/// Returns paths relative to the base directory.
pub fn walk_files(base: &Path, config: &WalkConfig) -> Vec<String> {
    let (tx, rx) = mpsc::channel();

    let walker = WalkBuilder::new(base)
        .hidden(false)
        .max_depth(Some(config.max_depth))
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .threads(
            std::thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(4),
        )
        .build_parallel();

    walker.run(|| {
        let tx = tx.clone();
        Box::new(move |result| {
            let entry = match result {
                Ok(e) => e,
                Err(_) => return WalkState::Continue,
            };

            // Skip root directory
            if entry.depth() == 0 {
                return WalkState::Continue;
            }

            // Skip directories in our skip list (and don't descend into them)
            if should_skip_entry(&entry) {
                return WalkState::Skip;
            }

            // Get relative path
            if let Ok(rel_path) = entry.path().strip_prefix(base) {
                if let Some(s) = rel_path.to_str() {
                    if !path_contains_skip_dir(s) {
                        let _ = tx.send(s.to_string());
                    }
                }
            }

            WalkState::Continue
        })
    });

    drop(tx); // Close sender so receiver iterator terminates
    rx.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_tree() -> TempDir {
        let dir = TempDir::new().unwrap();
        let base = dir.path();

        // Create structure:
        // ├── src/
        // │   ├── main.rs
        // │   └── lib.rs
        // ├── tests/
        // │   └── test.rs
        // ├── .git/
        // │   └── config
        // ├── node_modules/
        // │   └── pkg/
        // │       └── index.js
        // └── README.md

        fs::create_dir_all(base.join("src")).unwrap();
        fs::create_dir_all(base.join("tests")).unwrap();
        fs::create_dir_all(base.join(".git")).unwrap();
        fs::create_dir_all(base.join("node_modules/pkg")).unwrap();

        fs::write(base.join("src/main.rs"), "").unwrap();
        fs::write(base.join("src/lib.rs"), "").unwrap();
        fs::write(base.join("tests/test.rs"), "").unwrap();
        fs::write(base.join(".git/config"), "").unwrap();
        fs::write(base.join("node_modules/pkg/index.js"), "").unwrap();
        fs::write(base.join("README.md"), "").unwrap();

        dir
    }

    #[test]
    fn test_walk_excludes_git() {
        let dir = create_test_tree();
        let paths = walk_files(dir.path(), &DEEP_CONFIG);

        assert!(!paths.iter().any(|p| p.contains(".git")));
    }

    #[test]
    fn test_walk_excludes_node_modules() {
        let dir = create_test_tree();
        let paths = walk_files(dir.path(), &DEEP_CONFIG);

        assert!(!paths.iter().any(|p| p.contains("node_modules")));
    }

    #[test]
    fn test_walk_includes_src_files() {
        let dir = create_test_tree();
        let paths = walk_files(dir.path(), &DEEP_CONFIG);

        assert!(paths.iter().any(|p| p.ends_with("main.rs")));
        assert!(paths.iter().any(|p| p.ends_with("lib.rs")));
    }

    #[test]
    fn test_walk_includes_readme() {
        let dir = create_test_tree();
        let paths = walk_files(dir.path(), &DEEP_CONFIG);

        assert!(paths.iter().any(|p| p == "README.md"));
    }

    #[test]
    fn test_walk_max_depth_shallow() {
        let dir = TempDir::new().unwrap();
        let base = dir.path();

        // Create structure where shallow.txt is at depth 2, deep.txt at depth 7
        // depth 0: root, depth 1: a, depth 2: a/shallow.txt
        fs::create_dir_all(base.join("a/b/c/d/e/f")).unwrap();
        fs::write(base.join("a/b/c/d/e/f/deep.txt"), "").unwrap();
        fs::write(base.join("a/shallow.txt"), "").unwrap();

        // Shallow config (depth 2)
        let shallow = walk_files(base, &SHALLOW_CONFIG);
        assert!(shallow.iter().any(|p| p.contains("shallow.txt")));
        assert!(!shallow.iter().any(|p| p.contains("deep.txt")));
    }

    #[test]
    fn test_walk_max_depth_deep() {
        let dir = TempDir::new().unwrap();
        let base = dir.path();

        // Create deep structure
        fs::create_dir_all(base.join("a/b/c/d/e")).unwrap();
        fs::write(base.join("a/b/c/d/e/deep.txt"), "").unwrap();

        // Deep config (depth 6)
        let deep = walk_files(base, &DEEP_CONFIG);
        assert!(deep.iter().any(|p| p.contains("deep.txt")));
    }

    #[test]
    fn test_walk_respects_gitignore() {
        let dir = TempDir::new().unwrap();
        let base = dir.path();

        // Create .git directory so ignore crate treats this as a git repo
        fs::create_dir_all(base.join(".git")).unwrap();
        fs::write(base.join(".gitignore"), "ignored.txt\n").unwrap();
        fs::write(base.join("ignored.txt"), "").unwrap();
        fs::write(base.join("included.txt"), "").unwrap();

        let paths = walk_files(base, &DEEP_CONFIG);

        assert!(!paths.iter().any(|p| p.contains("ignored.txt")));
        assert!(paths.iter().any(|p| p.contains("included.txt")));
    }

    #[test]
    fn test_walk_includes_directories() {
        let dir = TempDir::new().unwrap();
        let base = dir.path();

        fs::create_dir_all(base.join("src")).unwrap();
        fs::write(base.join("src/main.rs"), "").unwrap();

        let paths = walk_files(base, &DEEP_CONFIG);

        // Should include both the directory and the file
        assert!(paths.iter().any(|p| p == "src"));
        assert!(paths.iter().any(|p| p == "src/main.rs"));
    }
}
