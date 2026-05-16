use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::ids::{
    Annotation, AssetId, ContentKey, NodeId, Rect, SessionId, SessionStatus, Timestamp, Viewport,
};
use super::relationship::Relationship;

// ── Helper types for NodeKind variants ───────────────────────────────────────

/// ARIA landmark and widget roles relevant to the Web_Adapter.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AriaRole {
    Banner,
    Complementary,
    Contentinfo,
    Form,
    Main,
    Navigation,
    Region,
    Search,
    Alert,
    Article,
    Cell,
    Columnheader,
    Definition,
    Figure,
    Group,
    Heading,
    Img,
    List,
    Listitem,
    Math,
    Note,
    Rowgroup,
    Rowheader,
    Separator,
    Table,
    Term,
    Toolbar,
    Tooltip,
    Tree,
    Treegrid,
}

/// Visibility / interactivity state of a `Region` node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RegionStatus {
    Visible,
    Hidden,
    Inert,
    Inaccessible,
}

/// Interactive widget types that map to `Component` nodes.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ComponentType {
    Button,
    Checkbox,
    Radio,
    Select,
    Slider,
    Switch,
    TextInput,
    DatePicker,
    FileUpload,
    Link,
    Image,
    Icon,
    Badge,
    Tag,
    Chip,
    Avatar,
    Spinner,
    Progress,
    Tooltip,
    Popover,
    Other,
}

/// HTML input types and textarea that map to `Field` nodes.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    Text,
    Email,
    Password,
    Number,
    Tel,
    Url,
    Search,
    Date,
    Time,
    DatetimeLocal,
    Month,
    Week,
    Color,
    File,
    Range,
    Hidden,
    Textarea,
    Select,
}

/// An opaque CSS/XPath selector string that stably addresses a UI element.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct Selector(pub String);

/// Redaction state of a node's text-bearing fields.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum RedactionState {
    /// No redactable content detected.
    Clean,
    /// Content was redacted; `count` placeholder tokens were inserted.
    Redacted { count: u32 },
    /// Pending scan (transient; bundles written to disk are never `Pending`).
    Pending,
}

/// The nine event kinds that can originate a WorkflowStep.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowEventKind {
    Navigation,
    RouteChange,
    DialogOpen,
    DialogClose,
    FormSubmit,
    TabChange,
    MenuOpen,
    MenuClose,
    Custom,
}

/// An opaque node-field patch, represented as a JSON Merge Patch (RFC 7396).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct NodePatch(pub serde_json::Value);

/// Graph diff between two consecutive WorkflowStep snapshots.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Delta {
    pub added: Vec<NodeId>,
    pub removed: Vec<NodeId>,
    pub changed: Vec<(NodeId, NodePatch)>,
    pub rels_added: Vec<Relationship>,
    pub rels_removed: Vec<Relationship>,
}

// ── SourceLocator ─────────────────────────────────────────────────────────────

/// Where a node originated. Internally tagged with `"modality"`.
/// `#[non_exhaustive]` allows new modalities under future MINOR bumps.
///
/// Requirements: 2.3, 22.2, 22.3
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "modality", rename_all = "snake_case")]
pub enum SourceLocator {
    Web {
        url: String,
        dom_path: Vec<u32>,
        captured_at: Timestamp,
        viewport: Viewport,
    },
    Code {
        repo_root: String,
        path: String,
        line_start: u32,
        line_end: u32,
        commit: Option<String>,
    },
    Pdf {
        path: String,
        page: u32,
        rect: Option<Rect>,
    },
    Image {
        path: String,
        rect: Option<Rect>,
    },
    Video {
        path: String,
        t_start_ms: u64,
        t_end_ms: u64,
    },
    Term {
        session: String,
        t_start_ms: u64,
        t_end_ms: u64,
    },
    Log {
        source: String,
        t_ms: u64,
    },
    Other {
        kind: String,
        fields: serde_json::Value,
    },
}

// ── NodeKind ─────────────────────────────────────────────────────────────────

/// The 20 node kinds from Req. 2.1. Internally tagged with `"kind"`.
///
/// `#[non_exhaustive]` allows MINOR schema bumps to add new kinds. Readers
/// that encounter an unknown kind SHOULD emit `Annotation { key:
/// "x2p.unknown_kind", value: <raw json> }` and continue.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NodeKind {
    Page {
        url: String,
        title: Option<String>,
        route: Option<String>,
    },
    Region {
        role: Option<AriaRole>,
        label: Option<String>,
        status: RegionStatus,
    },
    Component {
        component_type: ComponentType,
        label: Option<String>,
        value: Option<String>,
        selected: Option<bool>,
    },
    Form {
        name: Option<String>,
    },
    Field {
        field_type: FieldType,
        label: Option<String>,
        value: Option<String>,
        required: bool,
    },
    Action {
        name: String,
        target: Option<NodeId>,
    },
    Table {
        caption: Option<String>,
        columns: Vec<String>,
    },
    Row {
        ordinal: u32,
    },
    Cell {
        column: u32,
        value: String,
    },
    Dialog {
        title: Option<String>,
        modal: bool,
        active: bool,
    },
    Tab {
        label: String,
        active: bool,
    },
    Accordion {
        label: String,
        expanded: bool,
    },
    Menu {
        label: Option<String>,
        open: bool,
    },
    MenuItem {
        label: String,
        value: Option<String>,
    },
    Breadcrumb {
        trail: Vec<String>,
    },
    Workflow {
        session_id: SessionId,
    },
    WorkflowStep {
        ordinal: u32,
        /// The workflow event that triggered this step.
        /// Named `event_kind` in Rust to avoid conflict with the `"kind"` serde
        /// discriminant tag; serialized as `"event_kind"` on the wire.
        event_kind: WorkflowEventKind,
        ts: Timestamp,
        focused: Option<Selector>,
        delta: Delta,
    },
    Session {
        id: SessionId,
        started_at: Timestamp,
        ended_at: Option<Timestamp>,
        status: SessionStatus,
    },
    Asset {
        asset_id: AssetId,
        mime: String,
        size_bytes: u64,
        content_key: ContentKey,
    },
    Annotation {
        name: String,
        value: serde_json::Value,
    },
}

// ── Node ─────────────────────────────────────────────────────────────────────

/// A single vertex in the Context_Model graph.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Node {
    /// Stable, unique identity within the Context_Bundle.
    pub id: NodeId,
    pub kind: NodeKind,
    /// Origin of this node (modality, coordinates, capture timestamp).
    pub source: SourceLocator,
    /// Out-of-band annotations from adapters, plugins, or the Compressor.
    pub annotations: Vec<Annotation>,
    /// Redaction state of any text-bearing fields on this node.
    pub redaction: RedactionState,
    /// `true` → Compressor MUST NOT remove or truncate (Req. 12.4, 10.7).
    pub must_include: bool,
    /// `true` → node carries no semantic content; Compressor MAY drop unless
    /// `must_include` overrides (Req. 10.3).
    pub decorative_only: bool,
}
