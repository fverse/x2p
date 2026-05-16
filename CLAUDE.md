# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this project is

`x2p` (anything-to-prompt) converts arbitrary digital contexts — web pages, codebases, PDFs, images — into structured, token-efficient prompts for LLMs. It is a plugin-driven Rust framework with two delivery surfaces:
- **CLI** (`x2p`): terminal-based capture and render
- **Browser Extension** (Chromium + Firefox, MV3): live web UI capture via a native-messaging host

## Build and test commands

```bash
# Build the entire workspace
cargo build

# Run all tests
cargo test

# Run tests for a single crate
cargo test -p x2p-core

# Run a single test by name
cargo test -p x2p-core error_codes_are_stable_and_unique

# Lint (locally: warnings; CI escalates to errors via RUSTFLAGS="-D warnings")
cargo clippy

# Workspace task runner (subcommands are stubs — implementations land in later tasks)
cargo xtask gen-schemas        # → task 14.1
cargo xtask package-extension  # → task 26.2
cargo xtask release            # → task 24
```

## Crate architecture

The workspace uses a strict layering: `x2p-core` is a pure library with no I/O; every side effect is injected via traits (`Storage`, `Clock`, `Rng`, `Logger`, `PluginHost`, `Tokenizer`). Other crates depend on it and own all I/O.

| Crate | Role | Status |
|---|---|---|
| `x2p-core` | Context_Model schema, Prompt_Engine, Compressor, Redactor, plugin contract | Active |
| `x2p-wire` | Canonical JSON parser/serializer (single wire-format authority) | Skeleton |
| `x2p-cli` | `x2p` CLI binary | Skeleton (task 15.x) |
| `x2p-agent-server` | MCP tool server + JSON-RPC fallback | Phase 2 stub |
| `x2p-web-adapter-host` | Native-messaging host for the browser extension | Skeleton (task 20.x) |
| `x2p-plugin-host` | Plugin contract types (`PluginHost` trait, `PluginManifest`, `CapabilityManifest`) | Phase 1 contract only — no runtime |
| `x2p-storage` | BLAKE3 content-addressed cache + sled index + audit-log hash chain | Skeleton (tasks 7–8) |
| `x2p-tokenizers` | `cl100k`, `o200k`, llama tokenizers behind the `Tokenizer` trait | Skeleton (task 6.x) |
| `x2p-test-utils` | Shared proptest strategies and fault-injection helpers | Skeleton (task 5.x) |
| `xtask` | `cargo xtask` workspace task runner | Active stubs |

## Key domain concepts

**`ContextBundle`** (`x2p-core::model::bundle`) — the wire-level root of a capture. Contains `nodes`, `relationships`, `assets`, `sessions`, `redactions`, and a `bundle_id` (BLAKE3-256 self-attesting hash). The canonical JSON key order is lexicographic by Unicode code-point.

**`Node`** / **`NodeKind`** (`x2p-core::model::node`) — the fundamental unit of captured content. NodeKinds cover web UI primitives (Region, Component, TextField, WorkflowEvent, etc.) and cross-modality types (Text, Image, Code, Data).

**Identifiers** (`x2p-core::model::ids`) — all IDs are 128-bit ULIDs (time-ordered, lexicographically sortable). `ContentKey` is a 32-byte BLAKE3-256 hash, wire-encoded as 64 lowercase hex chars.

**`SchemaVersion`** — every bundle carries a `schema_version` (`semver::Version`). The loader accepts the range `^1` (`compatible_bundle_range()`). Mismatches produce `X2pError::SchemaVersionMismatch`.

**Plugin system** — Phase 1 ships the contract only (`PluginHost` trait, `PluginManifest`, `CapabilityManifest` in `x2p-plugin-host`). The Wasmtime component-model runtime is Phase 2.

## Error taxonomy

`X2pError` in `x2p-core::error` is the single error type for the entire platform. Error codes (`x2p::E####`) are **stable across releases** — never change a code once shipped; telemetry and audit chains depend on them. Every variant must be covered by the exhaustive match in `error_code()` and `remediation()`.

## Lint policy

- `unsafe_code` is `forbid`den in `x2p-core` and `x2p-plugin-host`; it is `warn` at the workspace level for other crates.
- Clippy `all` + `pedantic` are enabled workspace-wide. Accepted noisy lints: `module_name_repetitions`, `missing_errors_doc`, `missing_panics_doc`, `doc_markdown`.
- CI sets `RUSTFLAGS="-D warnings"` to treat all warnings as errors; locally they remain warnings.
- MSRV is **1.78** (enforced by `rust-version` in `Cargo.toml` and `msrv` in `clippy.toml`).
- Design-doc identifiers like `Context_Model`, `Plugin_Manifest` use underscore notation intentionally — do not wrap them in backticks to satisfy `doc_markdown`.
