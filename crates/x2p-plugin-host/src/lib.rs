//! Plugin contract surface for the x2p platform.
//!
//! # Phase 1 scope
//!
//! Per `design.md` § Roadmap, the full plugin host (Wasmtime component model
//! plus subprocess ABI) is a **Phase 2** deliverable. Phase 1 ships only the
//! _contract_: the [`PluginHost`] trait, the [`PluginManifest`] schema, and
//! the [`CapabilityManifest`] schema. There is intentionally **no runtime**
//! in this crate yet — no Wasmtime dependency, no plugin loader, no
//! sandboxing. Surfaces that need plugin functionality at MVP must use
//! built-in adapters / renderers compiled into the binary.
//!
//! When Phase 2 lands, the runtime modules will be added behind feature flags
//! so this crate remains usable as a contract-only dependency for tools that
//! only need the manifest types (e.g. a CI linter for `plugin.toml` files).
//!
//! # Naming
//!
//! The design document uses underscore-separated names like `Plugin_Manifest`
//! and `Capability_Manifest` to refer to schema entities. Rust types follow
//! `UpperCamelCase` (`PluginManifest`, `CapabilityManifest`). The two names
//! refer to the same thing; the canonical wire/TOML form is the underscore
//! version, the in-memory Rust form is the camel-case version.
//!
//! See `design.md` § "`Plugin_Manifest` Schema (TOML)" and § "`Capability_Manifest`
//! Schema" for the source of truth.

#![forbid(unsafe_code)]

use std::path::PathBuf;

// -----------------------------------------------------------------------------
// Local error placeholder.
//
// The platform-wide `X2pError` is implemented in `x2p-core` by task 2.1. To
// keep this contract-only crate dependency-free until that error taxonomy
// lands, we expose a local [`PluginHostError`] alias here. When `x2p-core` is
// available, the alias will be replaced with `x2p_core::X2pError` without
// changing the public trait signatures.
// -----------------------------------------------------------------------------

/// Error type returned by [`PluginHost`] operations.
///
/// Phase 1 placeholder — replaced by `x2p_core::X2pError` once task 2.1 lands.
#[derive(Debug)]
pub struct PluginHostError {
    /// Stable error code, e.g. `"x2p::E0009"`.
    pub code: &'static str,
    /// Human-readable message; structured details are added in Phase 2.
    pub message: String,
}

impl std::fmt::Display for PluginHostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for PluginHostError {}

// -----------------------------------------------------------------------------
// Plugin_Manifest schema
// -----------------------------------------------------------------------------

/// What kind of plugin this manifest describes.
///
/// Mirrors the `[plugin].kind` field in `plugin.toml`. Closed-world enum at
/// the wire level for any given schema version; new kinds bump the schema
/// version (Req. 16.3).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum PluginKind {
    /// Captures a modality and emits a Context_Bundle (e.g. web, code, PDF).
    SourceAdapter,
    /// Renders a Context_Bundle into a Prompt_Document or other artifact.
    OutputRenderer,
    /// A Tera prompt template package.
    PromptTemplate,
    /// A user-supplied compressor stage.
    Compressor,
    /// A user-supplied tokenizer.
    Tokenizer,
}

/// Modality declared by a `source_adapter` plugin (`[plugin].modality`).
///
/// Source adapters claim ownership of a modality so the registry can route
/// captures by modality string. Phase 1 only ships `Web`; later phases ship
/// more (Req. 22.5). Because new modalities can be introduced by external
/// plugins, this enum is `#[non_exhaustive]` and carries an `Other` escape
/// hatch keyed on a registry-reserved string identifier.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum Modality {
    /// Web pages and web applications via the browser extension.
    Web,
    /// Source code repositories (`code2prompt`-style ingest).
    Codebase,
    /// PDF documents.
    Pdf,
    /// Raster images.
    Image,
    /// Video files (frames + transcripts).
    Video,
    /// Terminal session logs.
    Terminal,
    /// Generic log files.
    Log,
    /// A registry-reserved modality identifier we do not yet have a variant
    /// for. The string MUST be reserved through the platform's modality
    /// registry to keep the wire form unambiguous.
    Other(String),
}

/// Plugin ABI declared in `[plugin].abi`.
///
/// `WasmComponent` is the recommended default (see ADR-004 in `design.md`).
/// `Subprocess` exists for plugins wrapping external binaries. `NativeDylib`
/// is discouraged and listed only for completeness; the Phase 2 host may
/// refuse to load native dylibs when the policy mode is `local-only`.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum PluginAbi {
    WasmComponent,
    Subprocess,
    NativeDylib,
}

/// `[entry]` table of `plugin.toml`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntryPoint {
    /// Path to the WASM component, when `abi = "wasm-component"`.
    pub component: Option<PathBuf>,
    /// Path to the subprocess binary, when `abi = "subprocess"`.
    pub binary: Option<PathBuf>,
    /// Names of exported functions the host may invoke.
    pub exports: Vec<String>,
}

/// `[[signature]]` entry in `plugin.toml`.
///
/// Trusted-plugins-only mode (Req. 17.4) verifies these in Phase 2. Phase 1
/// captures the schema only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Signature {
    /// Signing algorithm; the only Phase 2 value will be `"ed25519"`.
    pub algorithm: String,
    /// Stable, globally unique identifier of the signing key.
    pub key_id: String,
    /// Base64-encoded detached signature over the canonicalized manifest.
    pub sig: String,
}

/// In-memory representation of `plugin.toml` (the `Plugin_Manifest` schema).
///
/// The canonical TOML form is published at
/// `x2p-core/schema/plugin_manifest.schema.json` and is validated with
/// `jsonschema` at every load (Req. 16.2). Property 23 covers the round-trip.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginManifest {
    /// `[plugin].name`.
    pub name: String,
    /// `[plugin].version` — the plugin's own SemVer.
    ///
    /// Stored as a string in this Phase 1 contract surface so the crate stays
    /// dependency-free; Phase 2 (or task 2.x) will switch this to
    /// `semver::Version` once the workspace consensus dependency is in place.
    pub version: String,
    pub kind: PluginKind,
    /// Required iff `kind == PluginKind::SourceAdapter`.
    pub modality: Option<Modality>,
    pub abi: PluginAbi,
    /// `compatible_core` — the range of `x2p-core` versions this plugin
    /// supports (Req. 16.3). `semver::VersionReq` form, stored as string here.
    pub compatible_core: String,
    /// `schema_version` — the Context_Model schema version this plugin emits.
    pub schema_version: String,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub entry: EntryPoint,
    pub capabilities: CapabilityManifest,
    pub signatures: Vec<Signature>,
}

// -----------------------------------------------------------------------------
// Capability_Manifest schema
// -----------------------------------------------------------------------------

/// In-memory representation of the `[capabilities]` table.
///
/// Path globs are matched against the **canonicalized absolute path** of every
/// operation the plugin attempts; environment-variable interpolation (e.g.
/// `${input_path}`) is resolved at call time using a known-safe set of
/// variables provided by the host (no arbitrary env access). See `design.md`
/// § "`Capability_Manifest` Schema".
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CapabilityManifest {
    /// Glob patterns the plugin is allowed to read.
    pub fs_read: Vec<String>,
    /// Glob patterns the plugin is allowed to write.
    pub fs_write: Vec<String>,
    /// Network destinations the plugin is allowed to reach. Empty in
    /// `local-only` mode (Req. 18.4, 18.5).
    pub network: Vec<String>,
    /// Names of host-provided APIs the plugin may call (e.g. `"clock"`,
    /// `"logger"`, `"redactor"`).
    pub host_api: Vec<String>,
    /// Maximum CPU budget per invocation, in milliseconds.
    pub cpu_ms_max: Option<u64>,
    /// Maximum memory budget per invocation, in MiB.
    pub mem_mb_max: Option<u64>,
}

// -----------------------------------------------------------------------------
// PluginHost trait
// -----------------------------------------------------------------------------

/// Opaque descriptor of a discovered-but-not-yet-loaded plugin.
///
/// Phase 1 declares only the type; the runtime fields are added by the
/// Phase 2 loader.
#[derive(Debug)]
#[non_exhaustive]
pub struct PluginDescriptor {
    pub manifest: PluginManifest,
}

/// Opaque handle to a loaded-but-not-yet-invoked plugin instance.
#[derive(Debug)]
#[non_exhaustive]
pub struct LoadedPlugin {
    /// Stable identifier of the loaded instance, scoped to the host process.
    pub handle: PluginHandle,
}

/// Stable identifier for a loaded plugin instance (Phase 2 fills this in).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PluginHandle(pub u64);

/// Operation requested of a loaded plugin (Phase 2 expands this enum).
#[derive(Debug)]
#[non_exhaustive]
pub enum PluginOp<'a> {
    #[doc(hidden)]
    /// Reserves the lifetime parameter for the Phase 2 host without exposing
    /// any operation in Phase 1.
    _PhaseTwo(std::marker::PhantomData<&'a ()>),
}

/// Result of a plugin invocation (Phase 2 expands this enum).
#[derive(Debug)]
#[non_exhaustive]
pub enum PluginResult {
    /// Phase 1 placeholder — replaced when the operation set is filled in.
    NotImplemented,
}

/// The plugin host port.
///
/// Phase 1 declares this trait only; no implementer is shipped. Phase 2 will
/// add `WasmHost` and `SubprocessHost` implementations behind feature flags.
///
/// The trait signature mirrors `design.md` § "`PluginHost` port". The
/// `invoke` return type is a boxed `Future` rather than the design's
/// `futures::future::BoxFuture` so this crate stays dependency-free; the two
/// shapes are interoperable (`BoxFuture<'a, T>` is a type alias for
/// `Pin<Box<dyn Future<Output = T> + Send + 'a>>`).
pub trait PluginHost: Send + Sync {
    /// Discover plugin manifests under the given search directories.
    fn discover(&self, dirs: &[PathBuf]) -> Result<Vec<PluginDescriptor>, PluginHostError>;

    /// Load (but do not invoke) a previously discovered plugin.
    fn load(&self, descriptor: &PluginDescriptor) -> Result<LoadedPlugin, PluginHostError>;

    /// Invoke an operation on a loaded plugin.
    fn invoke<'a>(
        &'a self,
        handle: &'a PluginHandle,
        op: PluginOp<'a>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<PluginResult, PluginHostError>> + Send + 'a>,
    >;
}
