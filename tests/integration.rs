use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::TempDir;

fn run_claude_search(query: &str, cwd: &str) -> String {
    let input = format!(r#"{{"query": "{}", "cwd": "{}"}}"#, query, cwd);

    let mut child = Command::new(env!("CARGO_BIN_EXE_claude-search"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();

    let output = child.wait_with_output().unwrap();
    String::from_utf8(output.stdout).unwrap()
}

fn create_test_project() -> TempDir {
    let dir = TempDir::new().unwrap();
    let base = dir.path();

    // Create a git repo structure
    fs::create_dir_all(base.join(".git")).unwrap();
    fs::create_dir_all(base.join("src")).unwrap();
    fs::create_dir_all(base.join("tests")).unwrap();

    fs::write(base.join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(base.join("src/lib.rs"), "").unwrap();
    fs::write(base.join("tests/test.rs"), "").unwrap();
    fs::write(base.join("Cargo.toml"), "[package]").unwrap();
    fs::write(base.join("README.md"), "# Test").unwrap();

    dir
}

#[test]
fn test_basic_query() {
    let dir = create_test_project();
    let output = run_claude_search("main", dir.path().to_str().unwrap());

    assert!(
        output.contains("main.rs"),
        "Expected main.rs in output: {}",
        output
    );
}

#[test]
fn test_path_query() {
    let dir = create_test_project();
    let output = run_claude_search("src/main", dir.path().to_str().unwrap());

    assert!(
        output.contains("src/main.rs"),
        "Expected src/main.rs in output: {}",
        output
    );
}

#[test]
fn test_empty_query_shallow_listing() {
    let dir = create_test_project();
    let output = run_claude_search("", dir.path().to_str().unwrap());

    // Should list files at shallow depth
    assert!(!output.is_empty(), "Expected some output for empty query");
    // Should have Cargo.toml or src directory
    assert!(
        output.contains("Cargo.toml") || output.contains("src"),
        "Expected Cargo.toml or src in output: {}",
        output
    );
}

#[test]
fn test_parent_directory_query() {
    let dir = create_test_project();
    let src_dir = dir.path().join("src");

    let output = run_claude_search("../Cargo", src_dir.to_str().unwrap());

    // Output should have ../ prefix
    assert!(
        output.contains("../Cargo.toml"),
        "Expected ../Cargo.toml in output: {}",
        output
    );
}

#[test]
fn test_double_parent_prefix() {
    let dir = create_test_project();
    // Create nested structure
    let nested = dir.path().join("src/nested");
    fs::create_dir_all(&nested).unwrap();

    let output = run_claude_search("../../README", nested.to_str().unwrap());

    assert!(
        output.contains("../../README.md"),
        "Expected ../../README.md in output: {}",
        output
    );
}

#[test]
fn test_excludes_git_directory() {
    let dir = create_test_project();
    // Add a file inside .git
    fs::write(dir.path().join(".git/config"), "[core]").unwrap();

    let output = run_claude_search("config", dir.path().to_str().unwrap());

    // Should not find .git/config
    assert!(
        !output.contains(".git"),
        "Should not include .git directory: {}",
        output
    );
}

#[test]
fn test_excludes_node_modules() {
    let dir = create_test_project();
    fs::create_dir_all(dir.path().join("node_modules/pkg")).unwrap();
    fs::write(dir.path().join("node_modules/pkg/index.js"), "").unwrap();

    let output = run_claude_search("index", dir.path().to_str().unwrap());

    assert!(
        !output.contains("node_modules"),
        "Should not include node_modules: {}",
        output
    );
}

#[test]
fn test_respects_gitignore() {
    let dir = create_test_project();
    fs::write(dir.path().join(".gitignore"), "ignored.txt\n").unwrap();
    fs::write(dir.path().join("ignored.txt"), "should be ignored").unwrap();
    fs::write(dir.path().join("included.txt"), "should be included").unwrap();

    let output = run_claude_search("txt", dir.path().to_str().unwrap());

    assert!(
        !output.contains("ignored.txt"),
        "Should not include ignored.txt: {}",
        output
    );
    assert!(
        output.contains("included.txt"),
        "Should include included.txt: {}",
        output
    );
}

#[test]
fn test_invalid_json_exits_cleanly() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_claude-search"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"not valid json")
        .unwrap();

    let output = child.wait_with_output().unwrap();

    // Should exit 0 with no output
    assert!(output.status.success(), "Should exit with success");
    assert!(output.stdout.is_empty(), "Should have no stdout output");
}

#[test]
fn test_nonexistent_directory() {
    let output = run_claude_search("foo", "/nonexistent/path/12345");

    // Should return empty (silent failure)
    assert!(
        output.is_empty(),
        "Should return empty for nonexistent path"
    );
}

#[test]
fn test_missing_cwd_uses_current() {
    // Test with missing cwd field
    let input = r#"{"query": "main"}"#;

    let mut child = Command::new(env!("CARGO_BIN_EXE_claude-search"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();

    let output = child.wait_with_output().unwrap();

    // Should exit successfully (may or may not find files depending on current dir)
    assert!(output.status.success(), "Should exit with success");
}

#[test]
fn test_fuzzy_matching() {
    let dir = create_test_project();
    // Query for "mn" should match "main"
    let output = run_claude_search("mn", dir.path().to_str().unwrap());

    assert!(
        output.contains("main.rs"),
        "Fuzzy match 'mn' should find main.rs: {}",
        output
    );
}

#[test]
fn test_result_limit() {
    let dir = TempDir::new().unwrap();
    let base = dir.path();
    fs::create_dir_all(base.join(".git")).unwrap();

    // Create 100 files
    for i in 0..100 {
        fs::write(base.join(format!("file{}.txt", i)), "").unwrap();
    }

    let output = run_claude_search("file", base.to_str().unwrap());
    let lines: Vec<&str> = output.lines().collect();

    // Should be limited to 50 results
    assert!(
        lines.len() <= 50,
        "Should return at most 50 results, got {}",
        lines.len()
    );
}
