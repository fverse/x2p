//! Context_Model schema
//!
//! Public re-exports flatten the sub-module hierarchy for callers:
//! `use x2p_core::model::{ContextBundle, NodeId, Node, ...}`.

pub mod bundle;
pub mod ids;
pub mod node;
pub mod relationship;

// Flat re-exports — ids
pub use ids::{
    Annotation, AssetId, ContentKey, NodeId, Rect, SchemaVersion, SessionId, SessionStatus,
    StepId, Timestamp, Viewport, compatible_bundle_range, current_schema_version,
};

// Flat re-exports — relationship
pub use relationship::{Relationship, RelationshipKind};

// Flat re-exports — node
pub use node::{
    AriaRole, ComponentType, Delta, FieldType, NodeKind, NodePatch, RedactionState,
    RegionStatus, Selector, SourceLocator, WorkflowEventKind, Node,
};

// Flat re-exports — bundle
pub use bundle::{AssetRef, BundleSource, ContextBundle, RedactionRecord, Session};
