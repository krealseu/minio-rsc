//! Data types

mod select_object_content;

pub use select_object_content::*;

use serde::{Deserialize, Serialize};

use crate::time::UtcTime;

#[derive(Clone, Debug, PartialEq)]
pub struct Region(pub String);

trait XmlSelf {}

macro_rules! impl_xmlself {
    ($($name:tt )*) => {
        $(
            impl XmlSelf for $name{}
        )*
    };
}

impl_xmlself!(
    CommonPrefix
    LegalHold
    VersioningConfiguration
    Retention
    CompleteMultipartUpload
    CompleteMultipartUploadResult
    InitiateMultipartUploadResult
    ListMultipartUploadsResult
    CopyPartResult
    ListPartsResult
    ListAllMyBucketsResult
    ListBucketResult
);

pub trait ToXml {
    /// try get xml string
    fn to_xml(&self) -> crate::error::Result<String>;
}

impl<T: Serialize + XmlSelf> ToXml for T {
    fn to_xml(&self) -> crate::error::Result<String> {
        crate::xml::ser::to_string(&self).map_err(Into::into)
    }
}

pub trait FromXml: Sized {
    /// try from xml string
    fn from_xml(v: String) -> crate::error::Result<Self>;
}

impl<'de, T: Deserialize<'de> + XmlSelf> FromXml for T {
    fn from_xml(v: String) -> crate::error::Result<Self> {
        crate::xml::de::from_string(v).map_err(Into::into)
    }
}

impl Region {
    pub fn from<S>(region: S) -> Self
    where
        S: Into<String>,
    {
        return Self(region.into());
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Bucket {
    /// The name of the bucket.
    pub name: String,
    /// Date the bucket was created. This date can change when making changes to your bucket, such as editing its bucket policy.
    pub creation_date: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Buckets {
    #[serde(default)]
    pub bucket: Vec<Bucket>,
}

/// Container for all (if there are any) keys between Prefix and the next occurrence of the string specified by a delimiter.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CommonPrefix {
    pub prefix: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CompleteMultipartUpload {
    #[serde(default, rename="Part")]
    pub parts: Vec<Part>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CompleteMultipartUploadResult {
    pub bucket: String,
    pub key: String,
    pub e_tag: String,
    pub location: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CopyPartResult {
    pub e_tag: String,
}

/// The container element for specifying the default Object Lock retention settings
/// for new objects placed in the specified bucket.
///
/// **Note**
/// - The DefaultRetention settings require **both** a `mode` and a `period`.
/// - The DefaultRetention period can be either Days or Years but you must select one.
///   You cannot specify Days and Years at the same time.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct DefaultRetention {
    pub days: Option<usize>,
    pub mode: RetentionMode,
    pub years: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct InitiateMultipartUploadResult {
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
}

/// Container element that identifies who initiated the multipart upload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Initiator {
    pub display_name: String,
    #[serde(rename = "ID")]
    pub id: String,
}

/// A legal hold configuration for an object.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct LegalHold {
    pub status: LegalHoldStatus,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListAllMyBucketsResult {
    #[serde(default)]
    pub buckets: Buckets,
    pub owner: Owner,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListBucketResult {
    pub name: String,
    pub prefix: String,
    pub key_count: usize,
    pub max_keys: usize,
    #[serde(default)]
    pub delimiter: String,
    pub is_truncated: bool,
    pub start_after: Option<String>,
    #[serde(default)]
    pub contents: Vec<Object>,
    #[serde(default)]
    pub common_prefixes: Vec<CommonPrefix>,
    #[serde(default)]
    pub next_continuation_token: String,
    #[serde(default)]
    pub continuation_token: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListMultipartUploadsResult {
    pub bucket: String,
    pub key_marker: String,
    pub upload_id_marker: String,
    pub next_key_marker: String,
    pub prefix: String,
    pub delimiter: String,
    pub next_upload_id_marker: String,
    pub max_uploads: usize,
    pub is_truncated: bool,
    #[serde(default, rename = "Upload")]
    pub uploads: Vec<MultipartUpload>,
    #[serde(default)]
    pub common_prefixes: Vec<CommonPrefix>,
    pub encoding_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListPartsResult {
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
    pub part_number_marker: usize,
    pub max_parts: usize,
    pub next_part_number_marker: usize,
    pub is_truncated: bool,
    #[serde(default, rename = "Part")]
    pub parts: Vec<Part>,
    pub storage_class: String,
    pub checksum_algorithm: String,
    pub initiator: Initiator,
    pub owner: Owner,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MultipartUpload {
    pub checksum_algorithm: String,
    pub upload_id: String,
    pub storage_class: String,
    pub key: String,
    pub initiated: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Object {
    pub key: String,
    pub last_modified: String,
    pub e_tag: String,
    pub size: u64,
    pub storage_class: String,
    pub owner: Option<Owner>,
    pub checksum_algorithm: Option<String>,
}

/// The container element for an Object Lock rule.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectLockRule {
    pub default_retention: DefaultRetention,
}

/// Object representation of
/// - request XML of `put_object_lock_configuration` API
/// - response XML of `get_object_lock_configuration` API.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectLockConfiguration {
    /// Indicates whether this bucket has an Object Lock configuration enabled.
    /// Enable ObjectLockEnabled when you apply ObjectLockConfiguration to a bucket.
    ///
    /// Valid Values: `Enabled`
    /// Required: No
    pub object_lock_enabled: String,
    pub rule: Option<ObjectLockRule>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Owner {
    pub display_name: String,
    #[serde(rename = "ID")]
    pub id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Part {
    pub e_tag: String,
    pub part_number: usize,
}

/// This data type contains information about progress of an operation.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Progress {
    pub bytes_processed: u64,
    pub bytes_returned: u64,
    pub bytes_scanned: u64,
}

/// A container for replication rules. You can add up to 1,000 rules. The maximum size of a replication configuration is 2 MB.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReplicationConfiguration {
    pub role: String,
    #[serde(rename = "Rule", default)]
    pub rules: Vec<ReplicationRule>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReplicationRule {
    pub role: String,
}

/// Object representation of request XML of `put_object_retention` API
/// and response XML of `get_object_retention` API.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Retention {
    /// Valid Values: GOVERNANCE | COMPLIANCE
    pub mode: RetentionMode,
    /// The date on which this Object Lock Retention will expire.
    #[serde(deserialize_with = "crate::time::deserialize_with_str")]
    pub retain_until_date: UtcTime,
}

/// Container for the stats details.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Stats {
    pub bytes_processed: u64,
    pub bytes_returned: u64,
    pub bytes_scanned: u64,
}

/// A container of a key value name pair.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Tag {
    pub key: String,
    pub value: String,
}

/// A collection for a set of tags
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct TagSet {
    #[serde(rename = "Tag", default)]
    pub tags: Vec<Tag>,
}

/// Container for TagSet elements.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Tagging {
    pub tag_set: TagSet,
}

/// Describes the versioning state of an Amazon S3 bucket.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct VersioningConfiguration {
    /// Specifies whether MFA delete is enabled in the bucket versioning configuration.
    /// This element is only returned if the bucket has been configured with MFA delete.
    /// If the bucket has never been so configured, this element is not returned.
    ///
    /// Valid Values: Enabled | Disabled
    pub mfa_delete: Option<MFADelete>,

    /// The versioning state of the bucket.
    ///
    /// Valid Values: Enabled | Suspended
    pub status: Option<VersioningStatus>,
}

//////////////////  Enum Type

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum ChecksumAlgorithm {
    CRC32,
    CRC32C,
    SHA1,
    SHA256,
}

/// Specifies whether MFA delete is enabled in the bucket versioning configuration.
/// This element is only returned if the bucket has been configured with MFA delete.
/// If the bucket has never been so configured, this element is not returned.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum MFADelete {
    Enabled,
    Disabled,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum LegalHoldStatus {
    ON,
    OFF,
}

/// Retention mode, Valid Values: `GOVERNANCE | COMPLIANCE`
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub enum RetentionMode {
    #[default]
    GOVERNANCE,
    COMPLIANCE,
}

/// The versioning state of the bucket.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum VersioningStatus {
    Enabled,
    Suspended,
}
