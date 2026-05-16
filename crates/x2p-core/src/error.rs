//! Platform-wide error taxonomy for the `x2p` core library.
//!
//! This module implements the single error type that every public surface of
//! `x2p-core` returns, and is the **stable** authority for error codes that
//! telemetry, scripts, audit logs, and external automation pivot on.
//!
//! See `design.md` § Error Handling for the full taxonomy and the stable
//! `x2p::E####` prefix; see `requirements.md` Requirement 24 for the
//! acceptance criteria this module is held against.
//!
//! # Stability contract
//!
//! - The string returned by [`X2pError::error_code`] is **stable** across all
//!   future versions of `x2p-core`. It MUST NOT be changed once a release
//!   has been cut: tooling and audit chains depend on equality on this
//!   identifier (Req. 24.1, Property 24).
//! - The string returned by [`X2pError::remediation`] is documentation
//!   intended for human operators. Wording MAY be improved across versions
//!   but the **meaning** SHALL remain stable.
//! - The enum is marked `#[non_exhaustive]` so that future minor releases of
//!   `x2p-core` may add new variants without breaking downstream `match`es.
//! - The `Display` message for each variant is structured (`E#### key:
//!   field=value, …`) so that `tracing` output can be parsed in dashboards
//!   without needing a JSON formatter.
//!
//! # Type placeholders
//!
//! Several variants reference identifiers that are owned by other tasks in
//! the implementation plan (see `tasks.md` task 3.x for `NodeId`,
//! `SchemaVersion`, etc.). To keep this module compilable on its own, those
//! payloads are typed today as the most general values that round-trip
//! through `Display`:
//!
//! - `id: String` — used for `NodeId`-shaped variants. When task 3.1 lands,
//!   this will be tightened to the canonical newtype. Because the enum is
//!   `#[non_exhaustive]`, the change will be a SemVer-minor in practice for
//!   typical consumers (who match with a wildcard arm).
//! - `loaded: semver::Version` — used for the `SchemaVersion` payload. The
//!   `SchemaVersion` newtype defined in task 3.1 is a transparent wrapper
//!   over `semver::Version`, so this representation is wire-equivalent.

use thiserror::Error;

use crate::model::NodeId;

/// The single error type returned by every public function in `x2p-core`.
///
/// Every variant is documented with:
/// - Its stable code (`x2p::E####`), accessible via [`X2pError::error_code`].
/// - A one-line remediation hint, accessible via [`X2pError::remediation`].
/// - The canonical situation that causes it.
///
/// New variants are added under `#[non_exhaustive]` so the public API is
/// forward-compatible across MINOR releases (Req. 16.2, design § Schema
/// versioning).
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum X2pError {
    // -----------------------------------------------------------------
    // E0001 — schema_version_mismatch
    // -----------------------------------------------------------------
    /// `x2p::E0001` — A `Context_Bundle` (or other schema-versioned artifact)
    /// declares a `schema_version` that is not in the loader's
    /// `Compatible_Bundle_Range`.
    ///
    /// Remediation: upgrade `x2p-core` or regenerate the bundle so its
    /// `schema_version` satisfies the loader's required range.
    #[error("E0001 schema_version_mismatch: loaded={loaded}, required={required}")]
    SchemaVersionMismatch {
        /// The `schema_version` declared by the inbound artifact.
        loaded: semver::Version,
        /// The SemVer range the active `x2p-core` accepts.
        required: semver::VersionReq,
    },

    // -----------------------------------------------------------------
    // E0010 — capture_failed
    // -----------------------------------------------------------------
    /// `x2p::E0010` — A `Source_Adapter` could not complete a capture.
    ///
    /// Remediation: inspect the inner cause; rerun with `--profile` to
    /// capture stage timings and re-issue with a longer deadline if the
    /// cause was transient.
    #[error("E0010 capture_failed: kind={kind}")]
    CaptureFailed {
        /// Adapter-defined classification of the failure (e.g. `"web.dom"`,
        /// `"web.iframe.cross_origin"`).
        kind: String,
        /// Optional inner error preserving the cause chain.
        #[source]
        cause: Option<Box<X2pError>>,
    },

    // -----------------------------------------------------------------
    // E0020 — schema_validation_failed
    // -----------------------------------------------------------------
    /// `x2p::E0020` — A bundle, fragment, manifest, or config failed
    /// JSON-Schema or invariant validation at parse time.
    ///
    /// Remediation: fix the offending field at the reported path or update
    /// the source to honor the schema rule.
    #[error("E0020 schema_validation_failed: path={path}, rule={rule}")]
    SchemaValidationFailed {
        /// JSON Pointer (RFC 6901) into the offending document.
        path: String,
        /// Identifier of the schema rule that failed
        /// (e.g. `"required"`, `"pattern"`, `"unique_items"`).
        rule: String,
    },

    // -----------------------------------------------------------------
    // E0030 — plugin_load_failed
    // -----------------------------------------------------------------
    /// `x2p::E0030` — The Plugin_System could not load a plugin (manifest
    /// invalid, signature check failed, ABI mismatch, …).
    ///
    /// Remediation: re-install the plugin and verify its signature,
    /// manifest, and `Compatible_Core_Range`.
    #[error("E0030 plugin_load_failed: name={name}")]
    PluginLoadFailed {
        /// `Plugin_Name` from the manifest, or the discovered file name if
        /// the manifest itself failed to parse.
        name: String,
        /// Optional inner error preserving the cause chain.
        #[source]
        cause: Option<Box<X2pError>>,
    },

    // -----------------------------------------------------------------
    // E0031 — capability_denied
    // -----------------------------------------------------------------
    /// `x2p::E0031` — A plugin attempted an operation outside its declared
    /// `Capability_Manifest` (or denied by the active Policy_Engine).
    ///
    /// Remediation: add the operation to the plugin's `Capability_Manifest`
    /// or revise the policy that gates it.
    #[error("E0031 capability_denied: plugin={plugin}, operation={operation}")]
    CapabilityDenied {
        /// `Plugin_Name` of the offending plugin.
        plugin: String,
        /// Operation identifier (e.g. `"fs.read:/etc/passwd"`,
        /// `"net.connect:example.com:443"`).
        operation: String,
    },

    // -----------------------------------------------------------------
    // E0032 — version_mismatch
    // -----------------------------------------------------------------
    /// `x2p::E0032` — A component's reported version does not satisfy a
    /// SemVer range required by the loader (Plugin_System,
    /// Compatible_Core_Range, etc.).
    ///
    /// Remediation: upgrade or pin the component so the loaded version
    /// satisfies the required SemVer range.
    #[error("E0032 version_mismatch: loaded={loaded}, required={required}")]
    VersionMismatch {
        /// Version the component reported.
        loaded: semver::Version,
        /// SemVer range the loader requires.
        required: semver::VersionReq,
    },

    // -----------------------------------------------------------------
    // E0040 — policy_denied
    // -----------------------------------------------------------------
    /// `x2p::E0040` — The Policy_Engine refused a capture, render, plugin
    /// load, or egress request.
    ///
    /// Remediation: adjust the policy or run with `--policy=<file>` to
    /// grant the operation.
    #[error("E0040 policy_denied: reason={reason}")]
    PolicyDenied {
        /// Human-readable reason from the policy decision (must also be
        /// recorded in the Audit_Log per Req. 19.4).
        reason: String,
    },

    // -----------------------------------------------------------------
    // E0050 — budget_exceeded
    // -----------------------------------------------------------------
    /// `x2p::E0050` — After all Compressor passes the rendered prompt still
    /// exceeds the configured `Token_Budget`. Names the smallest section
    /// that could not be reduced further (Req. 11.4).
    ///
    /// Remediation: increase the `Token_Budget` or mark fewer nodes as
    /// `must_include`.
    #[error("E0050 budget_exceeded: smallest_unreducible={section}")]
    BudgetExceeded {
        /// Identifier of the smallest section the Compressor could not
        /// reduce below the budget (template section name or node id).
        section: String,
    },

    // -----------------------------------------------------------------
    // E0051 — memory_budget_exceeded
    // -----------------------------------------------------------------
    /// `x2p::E0051` — The cap allocator observed peak RSS exceeding the
    /// configured `Memory_Budget` (Req. 6.4, 20.2).
    ///
    /// Remediation: increase `x2p.memory_budget_mb` or shrink the working
    /// set (close tabs, split bundle, lower per-stage parallelism).
    #[error("E0051 memory_budget_exceeded: peak_mb={peak}, budget_mb={budget}")]
    MemoryBudgetExceeded {
        /// Observed peak resident-set size in megabytes.
        peak: u64,
        /// Configured `Memory_Budget` in megabytes.
        budget: u64,
    },

    // -----------------------------------------------------------------
    // E0060 — timeout
    // -----------------------------------------------------------------
    /// `x2p::E0060` — An `Operation_Timeout` cancelled the in-flight
    /// operation (Req. 20.6, Property 30).
    ///
    /// Remediation: increase the operation timeout or split the workload
    /// into smaller stages.
    #[error("E0060 timeout: stage={stage}, elapsed_ms={elapsed}")]
    Timeout {
        /// Pipeline stage that was running when the deadline fired
        /// (e.g. `"capture.dom_walk"`, `"render.tokenize"`).
        stage: String,
        /// Elapsed wall time in milliseconds.
        elapsed: u64,
    },

    // -----------------------------------------------------------------
    // E0070 — storage_failed
    // -----------------------------------------------------------------
    /// `x2p::E0070` — The Storage_Cache could not service a `put`/`get`/
    /// alias/pin/quarantine operation (disk full, permission, sled IO,
    /// corrupted entry, …).
    ///
    /// Remediation: check disk space and permissions for the Storage_Cache
    /// directory; rerun with `--profile`.
    #[error("E0070 storage_failed: operation={operation}")]
    StorageFailed {
        /// Storage operation that failed (e.g. `"put"`, `"alias.resolve"`,
        /// `"quarantine"`).
        operation: String,
        /// Optional inner error preserving the cause chain.
        #[source]
        cause: Option<Box<X2pError>>,
    },

    // -----------------------------------------------------------------
    // E0080 — transport_failed
    // -----------------------------------------------------------------
    /// `x2p::E0080` — A transport (native messaging, MCP, JSON-RPC, …)
    /// failed to deliver a message.
    ///
    /// Remediation: verify the endpoint is reachable and that
    /// `local-only` / `Egress_Allowlist` policies permit it.
    #[error("E0080 transport_failed: endpoint={endpoint}")]
    TransportFailed {
        /// Logical transport endpoint identifier
        /// (e.g. `"native:io.x2p.host"`, `"mcp://127.0.0.1:7421"`).
        endpoint: String,
        /// Optional inner error preserving the cause chain.
        #[source]
        cause: Option<Box<X2pError>>,
    },

    // -----------------------------------------------------------------
    // E0081 — protocol_mismatch
    // -----------------------------------------------------------------
    /// `x2p::E0081` — The CLI and a peer (Browser_Extension, Agent client)
    /// reported incompatible `protocol_version` fields during the
    /// handshake (Req. 25.5, Property 34).
    ///
    /// Remediation: upgrade the CLI and the extension/peer to a matching
    /// `protocol_version`.
    #[error("E0081 protocol_mismatch: cli={cli}, peer={peer}")]
    ProtocolMismatch {
        /// Protocol version reported by the CLI side.
        cli: semver::Version,
        /// Protocol version reported by the peer.
        peer: semver::Version,
    },

    // -----------------------------------------------------------------
    // E0090 — egress_denied
    // -----------------------------------------------------------------
    /// `x2p::E0090` — An outbound transport attempted to reach a
    /// destination not on the `Egress_Allowlist` (Req. 18.4, 18.5,
    /// Property 26).
    ///
    /// Remediation: add the destination to the `Egress_Allowlist` or run
    /// in `local-only` mode.
    #[error("E0090 egress_denied: destination={destination}")]
    EgressDenied {
        /// Resolved destination string (`host:port`, IP+port, or socket
        /// identifier). Note: the resolution is over the resolved IP, not
        /// the hostname.
        destination: String,
    },

    // -----------------------------------------------------------------
    // E0091 — audit_chain_corrupt
    // -----------------------------------------------------------------
    /// `x2p::E0091` — Audit_Log validation found a hash-chain mismatch at
    /// the reported ordinal (Req. 19.5, 19.6, Property 28).
    ///
    /// Remediation: quarantine the audit log starting at the reported
    /// ordinal and restore from backup.
    #[error("E0091 audit_chain_corrupt: at_ordinal={ordinal}")]
    AuditChainCorrupt {
        /// Ordinal of the first record whose `prev_hash` did not match the
        /// previous record's `self_hash`.
        ordinal: u64,
    },

    // -----------------------------------------------------------------
    // E0100 — duplicate_node_id
    // -----------------------------------------------------------------
    /// `x2p::E0100` — A `Source_Adapter` emitted two nodes with the same
    /// `NodeId` inside the same `Context_Bundle` (Req. 2.7, Property 5).
    ///
    /// Remediation: ensure each `Source_Adapter` assigns a unique
    /// `NodeId`; regenerate identifiers if necessary.
    #[error("E0100 duplicate_node_id: id={id}")]
    DuplicateNodeId {
        /// The conflicting `NodeId`.
        id: NodeId,
    },

    // -----------------------------------------------------------------
    // E0110 — referential_integrity
    // -----------------------------------------------------------------
    /// `x2p::E0110` — A `Relationship` references a `NodeId` that is not
    /// declared in the same `Context_Bundle` (Req. 2.6, Property 4).
    ///
    /// Remediation: add the missing Node to the bundle or remove the
    /// Relationship that references it.
    #[error("E0110 referential_integrity: missing_node_id={id}")]
    ReferentialIntegrity {
        /// The unknown `NodeId` referenced by a Relationship.
        id: NodeId,
    },
}

impl X2pError {
    /// Stable error code prefixed `x2p::E####`.
    ///
    /// The returned string is `&'static` and is the canonical identifier
    /// for telemetry, scripts, audit logs, and external automation
    /// It MUST NOT change between releases.
    #[must_use]
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::SchemaVersionMismatch { .. } => "x2p::E0001",
            Self::CaptureFailed { .. } => "x2p::E0010",
            Self::SchemaValidationFailed { .. } => "x2p::E0020",
            Self::PluginLoadFailed { .. } => "x2p::E0030",
            Self::CapabilityDenied { .. } => "x2p::E0031",
            Self::VersionMismatch { .. } => "x2p::E0032",
            Self::PolicyDenied { .. } => "x2p::E0040",
            Self::BudgetExceeded { .. } => "x2p::E0050",
            Self::MemoryBudgetExceeded { .. } => "x2p::E0051",
            Self::Timeout { .. } => "x2p::E0060",
            Self::StorageFailed { .. } => "x2p::E0070",
            Self::TransportFailed { .. } => "x2p::E0080",
            Self::ProtocolMismatch { .. } => "x2p::E0081",
            Self::EgressDenied { .. } => "x2p::E0090",
            Self::AuditChainCorrupt { .. } => "x2p::E0091",
            Self::DuplicateNodeId { .. } => "x2p::E0100",
            Self::ReferentialIntegrity { .. } => "x2p::E0110",
        }
    }

    /// One-line remediation hint suitable for inclusion in the structured
    /// error payload printed to stderr (Req. 24.2).
    ///
    /// The returned string is `&'static`. The wording may be refined in
    /// future releases; the **meaning** is part of the platform contract.
    #[must_use]
    pub fn remediation(&self) -> &'static str {
        match self {
            Self::SchemaVersionMismatch { .. } => {
                "Upgrade x2p-core or regenerate the bundle so its schema_version satisfies the loader's required range."
            }
            Self::CaptureFailed { .. } => {
                "Inspect the inner cause; rerun with --profile to capture stage timings, then retry the operation."
            }
            Self::SchemaValidationFailed { .. } => {
                "Fix the offending field at the reported path or update the source to honor the schema rule."
            }
            Self::PluginLoadFailed { .. } => {
                "Reinstall the plugin and verify its signature, manifest, and Compatible_Core_Range."
            }
            Self::CapabilityDenied { .. } => {
                "Add the operation to the plugin's Capability_Manifest or revise the policy that gates it."
            }
            Self::VersionMismatch { .. } => {
                "Upgrade or pin the component so the loaded version satisfies the required SemVer range."
            }
            Self::PolicyDenied { .. } => {
                "Adjust the active policy or run with --policy=<file> to grant the operation."
            }
            Self::BudgetExceeded { .. } => {
                "Increase the Token_Budget or mark fewer nodes as must_include in the render configuration."
            }
            Self::MemoryBudgetExceeded { .. } => {
                "Increase x2p.memory_budget_mb or shrink the working set (close tabs, split the bundle)."
            }
            Self::Timeout { .. } => {
                "Increase the operation timeout or split the workload into smaller stages."
            }
            Self::StorageFailed { .. } => {
                "Check disk space and permissions for the Storage_Cache directory; rerun with --profile."
            }
            Self::TransportFailed { .. } => {
                "Verify the endpoint is reachable and that local-only / Egress_Allowlist policies permit it."
            }
            Self::ProtocolMismatch { .. } => {
                "Upgrade the CLI and the extension/peer to a matching protocol_version."
            }
            Self::EgressDenied { .. } => {
                "Add the destination to the Egress_Allowlist or run in local-only mode."
            }
            Self::AuditChainCorrupt { .. } => {
                "Quarantine the audit log starting at the reported ordinal and restore from backup."
            }
            Self::DuplicateNodeId { .. } => {
                "Ensure each Source_Adapter assigns a unique NodeId; regenerate identifiers if necessary."
            }
            Self::ReferentialIntegrity { .. } => {
                "Add the missing Node to the bundle or remove the Relationship that references it."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for the stable `error_code()` and `remediation()`
    //! accessors.
    //!
    //! These tests are the explicit acceptance criteria for Req. 24.1 and
    //! Req. 24.2 — they freeze the per-variant strings so that any
    //! accidental change shows up as a failing test on PR.
    //!
    //! Property-based tests (Property 24, "Stable Error Codes") live in
    //! task 2.2 of `tasks.md`; this module restricts itself to the
    //! mandatory unit coverage that task 2.1 calls out.

    use super::*;
    use crate::model::NodeId;
    use semver::{Version, VersionReq};
    use ulid::Ulid;

    fn dummy_node_id() -> NodeId {
        NodeId(Ulid::from_string("01HZA7TR5N7M0K3Z3Q1V2W4XJ0").expect("valid ULID"))
    }

    fn dummy_node_id2() -> NodeId {
        NodeId(Ulid::from_string("01HZA7TR5N7M0K3Z3Q1V2W4XJ1").expect("valid ULID"))
    }

    /// Construct a representative instance for every variant. New variants
    /// added under `#[non_exhaustive]` MUST be added to this list as well
    /// so the per-variant accessor tests cover them.
    fn one_of_each() -> Vec<X2pError> {
        let v = Version::new(1, 2, 3);
        let req = VersionReq::parse(">=1.0.0, <2.0.0").expect("valid VersionReq");
        vec![
            X2pError::SchemaVersionMismatch {
                loaded: v.clone(),
                required: req.clone(),
            },
            X2pError::CaptureFailed {
                kind: "web.dom".to_string(),
                cause: None,
            },
            X2pError::SchemaValidationFailed {
                path: "/nodes/0/kind".to_string(),
                rule: "required".to_string(),
            },
            X2pError::PluginLoadFailed {
                name: "x2p-pdf".to_string(),
                cause: None,
            },
            X2pError::CapabilityDenied {
                plugin: "x2p-pdf".to_string(),
                operation: "fs.read:/etc".to_string(),
            },
            X2pError::VersionMismatch {
                loaded: v.clone(),
                required: req.clone(),
            },
            X2pError::PolicyDenied {
                reason: "egress denied".to_string(),
            },
            X2pError::BudgetExceeded {
                section: "outline".to_string(),
            },
            X2pError::MemoryBudgetExceeded {
                peak: 512,
                budget: 256,
            },
            X2pError::Timeout {
                stage: "render.tokenize".to_string(),
                elapsed: 30_000,
            },
            X2pError::StorageFailed {
                operation: "put".to_string(),
                cause: None,
            },
            X2pError::TransportFailed {
                endpoint: "native:io.x2p.host".to_string(),
                cause: None,
            },
            X2pError::ProtocolMismatch {
                cli: v.clone(),
                peer: Version::new(0, 9, 0),
            },
            X2pError::EgressDenied {
                destination: "203.0.113.1:443".to_string(),
            },
            X2pError::AuditChainCorrupt { ordinal: 42 },
            X2pError::DuplicateNodeId {
                id: dummy_node_id(),
            },
            X2pError::ReferentialIntegrity {
                id: dummy_node_id2(),
            },
        ]
    }

    /// Every variant defined in `design.md` § Error Handling MUST be
    /// present. If this fails, either a variant was deleted (a breaking
    /// change that requires a SemVer-major bump and a coordinated update
    /// of `design.md`) or the test fixture forgot to add a newly added
    /// variant.
    #[test]
    fn every_variant_is_covered() {
        // 17 variants per design.md § Error Handling.
        assert_eq!(one_of_each().len(), 17);
    }

    /// Acceptance criterion for Req. 24.1: every error code is the
    /// stable `x2p::E####` form and is unique across variants.
    #[test]
    fn error_codes_are_stable_and_unique() {
        let codes: Vec<&'static str> = one_of_each().iter().map(X2pError::error_code).collect();

        for code in &codes {
            assert!(
                code.starts_with("x2p::E") && code.len() == "x2p::E".len() + 4,
                "error code must match x2p::E#### format, got {code:?}"
            );
            // The four characters after the `x2p::E` prefix must all be
            // ASCII digits, so the codespace is partitioned cleanly.
            let digits = &code["x2p::E".len()..];
            assert!(
                digits.chars().all(|c| c.is_ascii_digit()),
                "error code suffix must be 4 digits, got {digits:?}"
            );
        }

        // Uniqueness: no two variants share an error code.
        let mut sorted = codes.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            codes.len(),
            "duplicate error code detected in {codes:?}"
        );
    }

    /// Pin the exact codes from `design.md` so any drift fails CI loudly.
    /// Edits to these strings are breaking changes — see the stability
    /// contract in the module docs.
    #[test]
    #[allow(clippy::too_many_lines)] // exhaustive variant table is the point
    fn error_codes_match_design_doc() {
        let v = Version::new(1, 0, 0);
        let req = VersionReq::parse("^1").unwrap();
        let cases: &[(X2pError, &str)] = &[
            (
                X2pError::SchemaVersionMismatch {
                    loaded: v.clone(),
                    required: req.clone(),
                },
                "x2p::E0001",
            ),
            (
                X2pError::CaptureFailed {
                    kind: "k".into(),
                    cause: None,
                },
                "x2p::E0010",
            ),
            (
                X2pError::SchemaValidationFailed {
                    path: "/".into(),
                    rule: "r".into(),
                },
                "x2p::E0020",
            ),
            (
                X2pError::PluginLoadFailed {
                    name: "n".into(),
                    cause: None,
                },
                "x2p::E0030",
            ),
            (
                X2pError::CapabilityDenied {
                    plugin: "p".into(),
                    operation: "o".into(),
                },
                "x2p::E0031",
            ),
            (
                X2pError::VersionMismatch {
                    loaded: v.clone(),
                    required: req.clone(),
                },
                "x2p::E0032",
            ),
            (
                X2pError::PolicyDenied { reason: "r".into() },
                "x2p::E0040",
            ),
            (
                X2pError::BudgetExceeded { section: "s".into() },
                "x2p::E0050",
            ),
            (
                X2pError::MemoryBudgetExceeded {
                    peak: 1,
                    budget: 1,
                },
                "x2p::E0051",
            ),
            (
                X2pError::Timeout {
                    stage: "s".into(),
                    elapsed: 0,
                },
                "x2p::E0060",
            ),
            (
                X2pError::StorageFailed {
                    operation: "o".into(),
                    cause: None,
                },
                "x2p::E0070",
            ),
            (
                X2pError::TransportFailed {
                    endpoint: "e".into(),
                    cause: None,
                },
                "x2p::E0080",
            ),
            (
                X2pError::ProtocolMismatch {
                    cli: v.clone(),
                    peer: v.clone(),
                },
                "x2p::E0081",
            ),
            (
                X2pError::EgressDenied {
                    destination: "d".into(),
                },
                "x2p::E0090",
            ),
            (X2pError::AuditChainCorrupt { ordinal: 0 }, "x2p::E0091"),
            (
                X2pError::DuplicateNodeId { id: dummy_node_id() },
                "x2p::E0100",
            ),
            (
                X2pError::ReferentialIntegrity { id: dummy_node_id() },
                "x2p::E0110",
            ),
        ];

        for (err, expected) in cases {
            assert_eq!(err.error_code(), *expected, "for variant {err:?}");
        }
    }

    /// Acceptance criterion for Req. 24.2: every variant returns a
    /// non-empty, single-line remediation string.
    #[test]
    fn remediation_is_present_and_single_line() {
        for err in one_of_each() {
            let hint = err.remediation();
            assert!(
                !hint.is_empty(),
                "variant {err:?} has empty remediation hint"
            );
            assert!(
                !hint.contains('\n'),
                "remediation hint must be single-line for variant {err:?}, got {hint:?}"
            );
            // The hint should be a real sentence, not a placeholder.
            assert!(
                hint.len() >= 16,
                "remediation hint too short for variant {err:?}: {hint:?}"
            );
        }
    }

    /// `Display` MUST embed the stable error code (it is structured for
    /// human + grep consumption per design.md § Error Handling).
    #[test]
    fn display_embeds_error_code() {
        for err in one_of_each() {
            let code = err.error_code();
            // The Display strings start with the bare numeric code (e.g.
            // "E0001 schema_version_mismatch: …"), so the digit suffix of
            // the stable code is what we check for.
            let suffix = &code["x2p::".len()..]; // "E0001"
            let rendered = err.to_string();
            assert!(
                rendered.starts_with(suffix),
                "Display for {err:?} should start with {suffix:?}, got {rendered:?}"
            );
        }
    }

    /// The `#[source]` cause chain MUST surface the boxed inner error so
    /// that `tracing` and CLI formatters can walk the chain.
    #[test]
    fn source_chain_is_walked_for_cause_carrying_variants() {
        use std::error::Error as _;

        let inner = X2pError::StorageFailed {
            operation: "put".into(),
            cause: None,
        };
        let outer = X2pError::CaptureFailed {
            kind: "web.dom".into(),
            cause: Some(Box::new(inner)),
        };

        let src = outer.source().expect("outer must have a source");
        // Display of the boxed inner error must surface its stable code so
        // the CLI's chain-formatter can grep for it.
        let rendered = src.to_string();
        assert!(
            rendered.starts_with("E0070"),
            "inner Display should embed E0070, got {rendered:?}"
        );
        // And there must be no further nesting on this leaf.
        assert!(
            src.source().is_none(),
            "leaf cause should not chain further, got {:?}",
            src.source()
        );
    }

    /// Regression: a future contributor adding a variant without updating
    /// `error_code()` would either fall through a wildcard arm or fail to
    /// compile. We currently match exhaustively in `error_code()` and
    /// `remediation()`, so this test simply asserts that property by
    /// re-counting variant coverage in a single place.
    #[test]
    fn accessors_match_exhaustively() {
        // If a new variant is added, `one_of_each` will need updating
        // (exhaustive matches in `error_code` and `remediation` will
        // fail to compile until then). This test is the single
        // human-visible surface that records that contract.
        let count = one_of_each().len();
        assert_eq!(
            count, 17,
            "x2p-core error taxonomy currently defines exactly 17 variants \
             per design.md § Error Handling. If this number is intentional \
             change, update one_of_each() and this assertion together."
        );
    }
}
