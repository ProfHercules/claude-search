# claude-search

Fast file suggestion tool for Claude Code.

## Commands

```bash
# Run tests
cargo test

# Build release
cargo build --release

# Install globally
cargo install --path .
```

## Architecture

```
src/
├── main.rs      # Entry point, orchestration
├── input.rs     # JSON parsing, ../ prefix extraction
├── walker.rs    # Parallel file traversal (ignore crate)
└── matcher.rs   # Fuzzy matching (nucleo-matcher)
```

## Input/Output

**stdin:** `{"query": "src/main", "cwd": "/path/to/project"}`

**stdout:** One matching path per line

## Key Behaviors

- Empty query → shallow listing (depth 2)
- Query present → deep search (depth 6) with fuzzy matching
- `../` prefix → search from parent, prepend prefix to output
- Respects `.gitignore`
- Skips: `.git`, `node_modules`, `target`, etc.
- Silent failure on errors (exit 0, no output)
