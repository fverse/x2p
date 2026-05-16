use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::ids::{Annotation, NodeId};

/// The eight canonical relationship kinds from Req. 2.2.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipKind {
    /// Structural parent/child containment.
    Contains,
    /// Page A navigates to page B via a link or redirect.
    NavigatesTo,
    /// A MenuItem opens a Menu; an Action opens a Dialog.
    Triggers,
    /// An Action depends on a Form for its input.
    DependsOn,
    /// A compressor-collapsed reference node points to its canonical subtree.
    References,
    /// WorkflowStep B follows WorkflowStep A in sequence.
    Follows,
    /// An updated-state node supersedes a prior node.
    Supersedes,
    /// A compressor-introduced summary node is derived from its source subtree.
    DerivedFrom,
}

/// A directed edge between two nodes in the Context_Model graph.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Relationship {
    pub from: NodeId,
    pub to: NodeId,
    pub kind: RelationshipKind,
    pub annotations: Vec<Annotation>,
}
