use std::collections::HashSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::ids::{
    Annotation, AssetId, ContentKey, NodeId, SchemaVersion, SessionId, SessionStatus, Timestamp,
    compatible_bundle_range,
};
use super::node::{Node, NodeKind};
use super::relationship::Relationship;
use crate::error::X2pError;

// ── AssetRef ─────────────────────────────────────────────────────────────────

/// A reference to a binary asset stored in the Content_Addressed_Store.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AssetRef {
    pub id: AssetId,
    pub mime: String,
    pub size_bytes: u64,
    /// BLAKE3-256 content key; the actual bytes live in the Storage_Cache.
    pub content_key: ContentKey,
}

// ── Session ───────────────────────────────────────────────────────────────────

/// Top-level session metadata carried in `ContextBundle.sessions`.
/// Sorted by `SessionId` for canonicalization.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Session {
    pub id: SessionId,
    pub started_at: Timestamp,
    pub ended_at: Option<Timestamp>,
    pub status: SessionStatus,
}

// ── BundleSource ─────────────────────────────────────────────────────────────

/// Identifies the adapter and plugin version that produced a bundle.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BundleSource {
    /// Adapter identifier, e.g. `"web"`, `"pdf"`.
    pub adapter: String,
    /// SemVer string of the plugin that performed the capture.
    pub plugin_version: String,
    /// UTC epoch milliseconds when the capture was initiated.
    pub captured_at: Timestamp,
}

// ── RedactionRecord ───────────────────────────────────────────────────────────

/// A record of one redaction applied to a node's field.
/// `ContextBundle.redactions` is sorted by `node_id`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RedactionRecord {
    /// The node whose field was redacted.
    pub node_id: NodeId,
    /// JSON field path within the node (e.g. `"kind.value"`).
    pub field: String,
    /// Detector kind that flagged the content (e.g. `"aws_key"`, `"email"`).
    pub detector_kind: String,
    /// The placeholder inserted in place of the original value
    /// (e.g. `"<REDACTED:aws_key>"`).
    pub placeholder: String,
}

// ── ContextBundle ─────────────────────────────────────────────────────────────

/// The wire-level root of a Context_Model serialization.
///
/// Field ordering matches the canonical JSON key order (lexicographic by
/// Unicode code-point per Req. 23.1). Arrays are sorted as specified per field.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ContextBundle {
    /// Bundle-level annotations from adapters or plugins.
    pub annotations: Vec<Annotation>,
    /// Binary assets referenced by nodes; sorted by `AssetId`.
    pub assets: Vec<AssetRef>,
    /// Self-attesting BLAKE3-256 of the canonical bundle bytes.
    /// Computed by zeroing this field, canonicalizing, hashing, then substituting.
    pub bundle_id: ContentKey,
    /// Nodes in the graph; sorted by `NodeId` for canonicalization.
    pub nodes: Vec<Node>,
    /// Edges in the graph; sorted by `(from, to, kind)`.
    pub relationships: Vec<Relationship>,
    /// Redaction records for every placeholder inserted; sorted by `node_id`.
    pub redactions: Vec<RedactionRecord>,
    /// Schema version of this bundle. Checked against `compatible_bundle_range()`.
    pub schema_version: SchemaVersion,
    /// Session metadata; sorted by `SessionId`.
    pub sessions: Vec<Session>,
    /// Provenance of this bundle (adapter, plugin version, capture timestamp).
    pub source: BundleSource,
}

impl ContextBundle {
    /// Run all parse-time invariant checks (Req. 2.4, 2.6, 2.7, 5.6).
    ///
    /// Returns the first violation found, in this order:
    /// 1. Schema version compatibility (`SchemaVersionMismatch`)
    /// 2. Unique `NodeId` within bundle (`DuplicateNodeId`)
    /// 3. Referential integrity of relationships (`ReferentialIntegrity`)
    /// 4. WorkflowStep total ordering and unique ordinals within each Workflow
    pub fn validate(&self) -> Result<(), X2pError> {
        self.check_schema_version()?;
        self.check_unique_node_ids()?;
        self.check_referential_integrity()?;
        self.check_workflow_step_ordering()?;
        Ok(())
    }

    fn check_schema_version(&self) -> Result<(), X2pError> {
        let range = compatible_bundle_range();
        if !range.matches(&self.schema_version.0) {
            return Err(X2pError::SchemaVersionMismatch {
                loaded: self.schema_version.0.clone(),
                required: range,
            });
        }
        Ok(())
    }

    fn check_unique_node_ids(&self) -> Result<(), X2pError> {
        let mut seen: HashSet<NodeId> = HashSet::with_capacity(self.nodes.len());
        for node in &self.nodes {
            if !seen.insert(node.id) {
                return Err(X2pError::DuplicateNodeId { id: node.id });
            }
        }
        Ok(())
    }

    fn check_referential_integrity(&self) -> Result<(), X2pError> {
        let ids: HashSet<NodeId> = self.nodes.iter().map(|n| n.id).collect();
        for rel in &self.relationships {
            if !ids.contains(&rel.from) {
                return Err(X2pError::ReferentialIntegrity { id: rel.from });
            }
            if !ids.contains(&rel.to) {
                return Err(X2pError::ReferentialIntegrity { id: rel.to });
            }
        }
        Ok(())
    }

    /// For each Workflow node, collect its WorkflowStep children (via `Contains`
    /// relationships), then verify the steps are totally ordered by `ts` and
    /// that no two steps share the same ordinal within a workflow.
    fn check_workflow_step_ordering(&self) -> Result<(), X2pError> {
        use super::relationship::RelationshipKind;

        // Build a map from NodeId → &Node for fast lookup.
        let node_map: std::collections::HashMap<NodeId, &Node> =
            self.nodes.iter().map(|n| (n.id, n)).collect();

        for node in &self.nodes {
            if !matches!(node.kind, NodeKind::Workflow { .. }) {
                continue;
            }

            // Collect WorkflowStep children in Contains-relationship order.
            let mut steps: Vec<(u32, i64)> = self
                .relationships
                .iter()
                .filter(|r| r.from == node.id && r.kind == RelationshipKind::Contains)
                .filter_map(|r| node_map.get(&r.to))
                .filter_map(|child| {
                    if let NodeKind::WorkflowStep { ordinal, ts, .. } = &child.kind {
                        Some((*ordinal, ts.0))
                    } else {
                        None
                    }
                })
                .collect();

            // Sort by ordinal for validation.
            steps.sort_by_key(|(ord, _)| *ord);

            // Verify: no duplicate ordinals, timestamps are non-decreasing.
            let mut prev_ord: Option<u32> = None;
            let mut prev_ts: Option<i64> = None;
            for (ord, ts) in &steps {
                if prev_ord == Some(*ord) {
                    // Duplicate ordinal within this workflow is a schema violation.
                    // We surface this as SchemaValidationFailed since StepId-level
                    // uniqueness is expressed via ordinals, not NodeIds.
                    return Err(X2pError::SchemaValidationFailed {
                        path: format!("/nodes[workflow={}]/ordinal", node.id),
                        rule: "unique_workflow_step_ordinal".to_owned(),
                    });
                }
                if let Some(p) = prev_ts {
                    if *ts < p {
                        return Err(X2pError::SchemaValidationFailed {
                            path: format!("/nodes[workflow={}]/ts", node.id),
                            rule: "workflow_steps_timestamp_order".to_owned(),
                        });
                    }
                }
                prev_ord = Some(*ord);
                prev_ts = Some(*ts);
            }
        }

        Ok(())
    }
}
