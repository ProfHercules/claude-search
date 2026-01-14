mod input;
mod matcher;
mod walker;

use mimalloc::MiMalloc;
use std::io::{self, BufWriter, Read, Write};
use std::path::PathBuf;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    if run().is_err() {
        // Silent failure - exit 0 per requirements
        std::process::exit(0);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Read stdin
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    // Parse JSON input
    let input: input::Input = serde_json::from_str(&buffer)?;

    // Get cwd, default to current directory
    let cwd = input
        .cwd
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    // Parse query and extract prefix/pattern
    let query = input.query.as_deref().unwrap_or("");
    let parsed = input::parse_query(query, &cwd);

    // Verify search base exists
    if !parsed.search_base.exists() {
        return Ok(()); // Silent failure
    }

    // Configure walk depth based on whether we have a pattern
    let walk_config = if parsed.is_empty {
        &walker::SHALLOW_CONFIG
    } else {
        &walker::DEEP_CONFIG
    };

    // Walk files
    let paths = walker::walk_files(&parsed.search_base, walk_config);

    // Match and rank
    let mut fuzzy_matcher = matcher::FuzzyMatcher::new();
    let results = fuzzy_matcher.match_paths(paths, &parsed.pattern, 50);

    // Output results with prefix
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    for path in results {
        writeln!(writer, "{}{}", parsed.output_prefix, path)?;
    }
    writer.flush()?;

    Ok(())
}
