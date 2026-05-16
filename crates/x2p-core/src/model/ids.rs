use schemars::JsonSchema;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// Unique, stable, time-ordered identity for a Node within a Context_Bundle.
/// 128-bit ULID; lexicographically sortable by creation time.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeId(pub Ulid);

impl JsonSchema for NodeId {
    fn schema_name() -> String {
        "NodeId".to_owned()
    }
    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        gen.subschema_for::<String>()
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Identity for a WorkflowStep; distinct newtype from NodeId for type safety.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StepId(pub Ulid);

impl JsonSchema for StepId {
    fn schema_name() -> String {
        "StepId".to_owned()
    }
    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        gen.subschema_for::<String>()
    }
}

impl std::fmt::Display for StepId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Identity for a Session; time-ordered, lexicographically sortable.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(pub Ulid);

impl JsonSchema for SessionId {
    fn schema_name() -> String {
        "SessionId".to_owned()
    }
    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        gen.subschema_for::<String>()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Identity for an Asset (binary attachment).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AssetId(pub Ulid);

impl JsonSchema for AssetId {
    fn schema_name() -> String {
        "AssetId".to_owned()
    }
    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        gen.subschema_for::<String>()
    }
}

impl std::fmt::Display for AssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// BLAKE3-256 content-addressed key. Wire-encoded as 64 lowercase hex chars.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ContentKey(pub [u8; 32]);

impl ContentKey {
    /// All-zero key used as a placeholder when computing `bundle_id`.
    pub const ZERO: Self = Self([0u8; 32]);
}

impl Serialize for ContentKey {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(&hex::encode(self.0))
    }
}

impl<'de> Deserialize<'de> for ContentKey {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let s = String::deserialize(de)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| serde::de::Error::custom("ContentKey must be 32 bytes (64 hex chars)"))?;
        Ok(Self(arr))
    }
}

impl JsonSchema for ContentKey {
    fn schema_name() -> String {
        "ContentKey".to_owned()
    }
    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        gen.subschema_for::<String>()
    }
}

impl std::fmt::Display for ContentKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

/// SemVer-tagged schema version carried on every Context_Bundle.
/// Serialized as a plain version string: `"1.0.0"`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaVersion(pub Version);

impl Serialize for SchemaVersion {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for SchemaVersion {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let s = String::deserialize(de)?;
        let v = Version::parse(&s).map_err(serde::de::Error::custom)?;
        Ok(Self(v))
    }
}

impl JsonSchema for SchemaVersion {
    fn schema_name() -> String {
        "SchemaVersion".to_owned()
    }
    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        gen.subschema_for::<String>()
    }
}

/// A key/value annotation attached to a Node, Relationship, or bundle.
///
/// `value` is an arbitrary JSON payload; callers MUST use string values for
/// human-readable annotations and structured objects for machine-readable ones.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Annotation {
    pub key: String,
    pub value: serde_json::Value,
}

/// UTC epoch milliseconds as a 64-bit signed integer. Floats are forbidden.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize, JsonSchema,
)]
#[serde(transparent)]
pub struct Timestamp(pub i64);

/// Pixel dimensions of the browser viewport at capture time.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Viewport {
    pub w: u32,
    pub h: u32,
}

/// Pixel-space bounding rectangle (origin at top-left, inclusive).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

/// Lifecycle state of a Session.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
    Completed,
    TimedOut,
    Aborted,
}

/// Schema version this build of `x2p-core` produces in new bundles.
pub fn current_schema_version() -> Version {
    Version::new(1, 0, 0)
}

/// SemVer range of `schema_version`s this build of `x2p-core` accepts.
/// Rejects any bundle whose `schema_version` is outside this range with
/// `X2pError::SchemaVersionMismatch` (Req. 2.4, Property 6).
pub fn compatible_bundle_range() -> VersionReq {
    VersionReq::parse("^1").expect("hard-coded VersionReq is valid")
}
