# claude-search

[![CI](https://github.com/ProfHercules/claude-search/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/ProfHercules/claude-search/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/ProfHercules/claude-search/graph/badge.svg)](https://codecov.io/gh/ProfHercules/claude-search)
[![GitHub release](https://img.shields.io/github/v/release/ProfHercules/claude-search)](https://github.com/ProfHercules/claude-search/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A fast file suggestion tool for [Claude Code](https://docs.anthropic.com/en/docs/claude-code). Replaces the default shell script with a native Rust binary.

## Benchmarks

Tested with [hyperfine](https://github.com/sharkdp/hyperfine) (50 runs, 5 warmup) on a codebase with ~1000 files:

| Query     | Shell Script | Rust Binary | Speedup |
| --------- | ------------ | ----------- | ------- |
| "config"  | 433ms        | 13ms        | **35x** |
| "test"    | 432ms        | 13ms        | **32x** |
| "../test" | 434ms        | 14ms        | **32x** |
| (empty)   | 43ms         | 8ms         | **5x**  |

The shell script spawns multiple processes (`jq`, `find`/`fd`, `fzf`). This binary does everything in-process with parallel file traversal.

## Installation

### Option 1: Download Binary

1. Download the latest binary for your platform from [Releases](https://github.com/ProfHercules/claude-search/releases/latest)
2. Rename and move to your PATH:

```bash
# macOS/Linux
mv claude-search-darwin-aarch64 /usr/local/bin/claude-search
chmod +x /usr/local/bin/claude-search

# Windows
move claude-search-windows-x86_64.exe C:\Windows\claude-search.exe
```

### Option 2: Build from Source

```bash
cargo install --git https://github.com/ProfHercules/claude-search
```

Or clone and build:

```bash
git clone https://github.com/ProfHercules/claude-search
cd claude-search
cargo install --path .
```

## Configuration

Add to your Claude Code settings (`~/.claude/settings.json`):

```json
{
  "fileSuggestion": {
    "type": "command",
    "command": "/usr/local/bin/claude-search"
  }
}
```

Adjust the path based on where you installed the binary.

## Features

- Parallel file traversal using [`ignore`](https://crates.io/crates/ignore) (same as `fd`/`ripgrep`)
- Fuzzy matching with [`nucleo-matcher`](https://crates.io/crates/nucleo-matcher) (same as Helix editor)
- Respects `.gitignore` automatically
- Skips common non-code directories (`.git`, `node_modules`, `target`, etc.)

## License

MIT
