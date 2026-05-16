//! Pure-library core for the x2p platform.
//!
//! This crate owns the Context_Model schema, the Prompt_Engine pipeline, the
//! Compressor, the Tokenizer registry, the Redactor, and the Plugin contract.
//! It performs **no** I/O; every side effect is mediated by an injected trait
//! (`Storage`, `Clock`, `Rng`, `Logger`, `PluginHost`, `Tokenizer`).
//!
//! See `.kiro/specs/context-to-prompt-platform/design.md` for the full
//! architecture, and `requirements.md` for the acceptance criteria this crate
//! is held against.
//!
//! ## Safety posture
//!
//! `unsafe_code` is `forbid`den for the entire crate. Any code path that needs
//! `unsafe` belongs in a sibling crate where it can be audited in isolation.

#![forbid(unsafe_code)]

/// Platform-wide error taxonomy (`X2pError`). See `error::X2pError`
/// documentation for the full list of stable error codes and the stability
/// contract (Req. 24.1, 24.2; design.md § Error Handling).
pub mod error;

/// Context_Model schema: identifiers, nodes, relationships, and bundles.
/// See `design.md` § Data Models and tasks 3.1–3.4.
pub mod model;

pub use error::X2pError;
