# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build

```bash
cargo build --release
```

Binary name: `ccs` (defined in Cargo.toml `[[bin]]`)

## Architecture

**Three-layer config selection mechanism** (Volta-style):

```
Priority: User specified > Project pin > Global default
```

Core modules:

- `config/` — Config management (store.rs handles JSON persistence)
- `cost/` — Cost tracking (collector.rs aggregates data, display.rs formats output, session.rs parses JSONL logs)
- `pin.rs` — Project binding (searches upward for .cc-switcher.json, max depth 20)
- `run.rs` — Claude Code executor (injects config via `--settings` flag)

## Storage Locations

```
~/.cc-switcher/
├── configs.json      # Config index (name → path mapping)
├── pricing.json      # Model pricing (auto-initialized from pricing.default.json)
└── configs/          # Config files created by `ccs new`

~/.claude/projects/   # Cost data source (JSONL session logs)
<project-dir>/.cc-switcher.json  # Project binding
```

## Config Resolution Flow

`run.rs:resolve_config_name()`:

1. If user provides name → validate existence, return
2. Else search upward for .cc-switcher.json → validate config exists, return
3. Else get global default from store → validate existence, return
4. Else error with setup instructions

## Cost Tracking

- Data source: `~/.claude/projects/*.jsonl` session files
- `session.rs` parses only `type="assistant"` messages (contains usage data)
- `collector.rs` aggregates by (date, model), calculates cost via `pricing.rs`
- Pricing auto-initialized from `pricing.default.json` on first run

## Key Data Structures

- `Config`: name, path, description, is_default flag
- `PinConfig`: config (name), description
- `CostRecord`: date, model, requests, tokens (input/output/cache_read/cache_creation)
- `AggregatedStats`: CostRecord + pre-calculated cost

## CLI Shorthand

`ccs <name>` uses clap's `external_subcommand` to catch unknown commands as config names.
Example: `ccs deepseek` → resolved as config "deepseek" → runs Claude with that config.

## Dependencies

clap (derive), serde (derive), chrono (serde), anyhow, dirs, walkdir