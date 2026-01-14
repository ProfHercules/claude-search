# claude-search

[![CI](https://github.com/ProfHercules/claude-search/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/ProfHercules/claude-search/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/ProfHercules/claude-search/graph/badge.svg)](https://codecov.io/gh/ProfHercules/claude-search)
[![GitHub release](https://img.shields.io/github/v/release/ProfHercules/claude-search)](https://github.com/ProfHercules/claude-search/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A fast file suggestion tool for [Claude Code](https://docs.anthropic.com/en/docs/claude-code), written in Rust.

Replaces the default shell script with a native binary that's **65x faster**.

## Performance

Benchmarked on a codebase with ~2100 files (after gitignore filtering):

| Scenario                         | Shell Script | Rust Binary | Speedup |
| -------------------------------- | ------------ | ----------- | ------- |
| Large codebase (query: "config") | 727ms        | 11ms        | **65x** |
| Small project (query: "main")    | 64ms         | 3ms         | **20x** |
| Empty query (shallow listing)    | 45ms         | 8ms         | **6x**  |

### Why So Fast?

The shell script spawns multiple processes (`jq`, `find`/`fd`, `fzf`) with IPC overhead:

```
Shell: JSON → jq → find/fd → fzf → output  (5 processes, ~700ms)
Rust:  JSON → parse → walk → match → output (1 process, ~11ms)
```

### Time Breakdown (Rust)

```
walk:   ~7ms   (parallel filesystem traversal - I/O bound)
match:  ~160µs (fuzzy matching 2100 paths)
other:  ~10µs  (parsing, output)
```

The bottleneck is filesystem I/O. Further optimization would require caching.

## Features

- **Parallel file traversal** using the `ignore` crate (same as `fd`/`ripgrep`)
- **Fuzzy matching** with `nucleo-matcher` (same algorithm as Helix editor)
- **Respects `.gitignore`** automatically
- **Handles relative paths** (`../foo`, `../../bar`)
- **Silent failures** - exits cleanly on errors

## Installation

```bash
# From source
cargo install --path .

# Or build manually
cargo build --release
cp target/release/claude-search ~/.cargo/bin/
```

## Configuration

Add to `~/.claude/settings.json`:

```json
{
  "fileSuggestion": {
    "type": "command",
    "command": "~/.cargo/bin/claude-search"
  }
}
```

## Usage

The tool reads JSON from stdin and outputs matching file paths:

```bash
echo '{"query": "main", "cwd": "/path/to/project"}' | claude-search
# Output:
# src/main.rs
# src/main/config.rs
```

### Input Format

```json
{
  "query": "src/main", // Search pattern (supports fuzzy matching)
  "cwd": "/path/to/dir" // Directory to search in
}
```

### Relative Path Prefixes

Queries can include `../` prefixes to search parent directories:

```bash
# From /project/src, search parent for "config"
echo '{"query": "../config", "cwd": "/project/src"}' | claude-search
# Output:
# ../config.json
# ../config/settings.rs
```

## How It Works

1. **Parse input** - Extract query pattern and `../` prefix chain
2. **Walk files** - Parallel traversal with gitignore support
3. **Fuzzy match** - Score and rank paths using nucleo algorithm
4. **Output** - Return top 50 matches with prefix prepended

### Skipped Directories

The following directories are automatically skipped:

- `.git`, `node_modules`, `.venv`, `__pycache__`
- `.mypy_cache`, `.cache`, `dist`, `build`
- `.next`, `target`, `.tox`, `.pytest_cache`

## Development

```bash
# Run tests
cargo test

# Build release
cargo build --release

# Run with profiling (dev builds only)
CLAUDE_SEARCH_PROFILE=1 echo '{"query": "test", "cwd": "."}' | ./target/debug/claude-search
```

## Dependencies

- [`serde`](https://crates.io/crates/serde) - JSON parsing
- [`ignore`](https://crates.io/crates/ignore) - Fast gitignore-aware file walking
- [`nucleo-matcher`](https://crates.io/crates/nucleo-matcher) - Fuzzy matching
- [`mimalloc`](https://crates.io/crates/mimalloc) - Fast memory allocator

## License

MIT

## Credits

The original shell script (`file-suggestion.sh.original`) used `fd` + `fzf` for file discovery and fuzzy matching. This Rust implementation achieves the same functionality with significantly better performance by avoiding process spawning overhead.
