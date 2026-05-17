# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this project is

`x2p` (anything-to-prompt) is a small Rust CLI that fetches a web page and
turns it into a markdown prompt for an LLM, with optional token-budget
pruning. Scope today is **v0.1**: a single value loop end-to-end. The earlier
9-crate spec in `.kiro/specs/context-to-prompt-platform/` is aspirational —
treat it as the long-term vision, not the current shape of the code.

## Build, test, run

```bash
cargo build
cargo test
cargo run -- capture https://example.com -o /tmp/b.json
cargo run -- render /tmp/b.json --budget 4000
```

## Workspace layout (2 crates)

- `crates/x2p-core` — pure library, no I/O. Owns `Bundle`, `Block`, the
  markdown renderer, the cl100k tokenizer wrapper, and the budget-pruner.
- `crates/x2p-cli` — the `x2p` binary. Owns clap, reqwest (fetching with a
  polite User-Agent), scraper-based HTML → Block conversion, and file I/O.

Surfaces beyond the CLI (browser extension, MCP agent server, plugin host,
storage cache, audit log, policy engine) do **not** exist yet. Add them
*only* when a concrete user need appears, not preemptively. The 9-crate
layout in git history was scrapped for being overbuilt before any user value
had shipped — don't recreate it module-by-module.

## Domain model

The Context Model is intentionally tiny:

```rust
struct Bundle { url, title, captured_at, blocks: Vec<Block> }
enum Block { Heading | Paragraph | List | Code | Table | Link | Form }
```

No Node/Relationship graph, no ULIDs, no `schema_version`, no canonical JSON,
no BLAKE3 `bundle_id`, no annotations, no sessions, no workflows. Bundles
serialize with plain `serde_json`. Internally-tagged via `#[serde(tag = "kind")]`.
Don't add fields speculatively — wait for a renderer or pruner that needs them.

## Capture pipeline

1. `reqwest::Client` with a UA string built from `CARGO_PKG_VERSION`.
   Wikipedia and other large sites 403 the default `reqwest/x.y.z` UA —
   keep ours set.
2. `scraper::Html::parse_document` → walk the body.
3. Tag-by-tag conversion: `h1..h6` → Heading, `p` → Paragraph, `ul/ol` →
   List, `pre>code` → Code (lang from `class="language-…"`), `table` → Table.
4. Skip `script`, `style`, `nav`, `header`, `footer`, `aside`, `noscript`, `svg`.
5. No JS execution — static and SSR pages only. Add headless Chromium
   (`chromiumoxide`) when a real SPA actually fails, not before.

## Render and prune

- `render(&Bundle, &RenderConfig) -> String` — pure function from bundle to
  markdown. Determinism is desirable; don't introduce hashmaps or time-based
  ordering in the output.
- If `RenderConfig.budget_tokens` is set, `prune::shrink` repeatedly drops
  the lowest-priority non-Heading block and re-renders until it fits. The
  priority order is in `prune.rs`; tune there if pruning quality matters for
  a real task. Algorithm is O(n²) in tokens — fine for v0.1, revisit only if
  it bites.

## Lint and toolchain

- MSRV **1.78** (pinned in `rust-toolchain.toml`, `clippy.toml`, and
  `Cargo.toml`).
- `cargo clippy` runs `clippy::all` at workspace level. Pedantic is off —
  add it back only if the codebase grows enough to need it.
- `unsafe_code` is `warn` workspace-wide and `forbid` in `x2p-core` (see
  `lib.rs`).

## When to add complexity back

The reference spec in `.kiro/specs/` lists 25 EARS requirements and 35
correctness properties. Treat that as a menu, not a roadmap. Resist the
urge to scaffold any of:

- BLAKE3 content-addressed storage / sled index
- Plugin manifest, sandbox, signed plugins
- MCP agent server
- Audit log hash chain
- Policy engine + egress allowlist
- Workflow capture + deterministic replay
- Canonical JSON + bundle_id self-attestation
- Stable `x2p::E####` error codes

…until there is a concrete user (yourself counts) blocked on the feature.
Each of these is a 1-PR addition when the need is real, and a multi-week
distraction when it isn't.
