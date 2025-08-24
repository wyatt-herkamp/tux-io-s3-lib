use std::fmt::Display;

use crate::owner::Owner;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Technically Every element is not required and that is confusing??
///
///
/// [S3 Object](https://docs.aws.amazon.com/AmazonS3/latest/API/API_Object.html)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Object {
    /// It is marked as not required the S3 Docs?
    pub key: String,
    pub last_modified: DateTime<FixedOffset>,
    pub size: u64,
    pub e_tag: Option<String>,
    pub storage_class: Option<StorageClass>,
    pub owner: Option<Owner>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageClass {
    Standard,
    ReducedRedundancy,
    Glacier,
    StandardIa,
    OnezoneIa,
    IntelligentTiering,
    DeepArchive,
    Outposts,
    GlacierIr,
    Snow,
    ExpressOnezone,
    FsxOpenZfs,
    Other(String),
}
macro_rules! storage_class {
    (
        $(
            $name:ident => $value:literal
        ),*
    ) => {
        impl Display for StorageClass{
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self{
                    $(
                        StorageClass::$name => write!(f, "{}", $value),
                    )*
                    StorageClass::Other(v) => write!(f, "{}", v)
                }
            }
        }
        impl From<&str> for StorageClass{
            fn from(value: &str) -> Self {
                match value{
                    $(
                        $value => StorageClass::$name,
                    )*
                    other => StorageClass::Other(other.to_string())
                }
            }
        }
        impl From<String> for StorageClass{
            fn from(value: String) -> Self {
                match value.as_str(){
                    $(
                        $value => StorageClass::$name,
                    )*
                    _ => StorageClass::Other(value)
                }
            }
        }
    };
}
storage_class! {
    Standard => "STANDARD",
    ReducedRedundancy => "REDUCED_REDUNDANCY",
    Glacier => "GLACIER",
    StandardIa => "STANDARD_IA",
    OnezoneIa => "ONEZONE_IA",
    IntelligentTiering => "INTELLIGENT_TIERING",
    DeepArchive => "DEEP_ARCHIVE",
    Outposts => "OUTPOSTS",
    GlacierIr => "GLACIER_IR",
    Snow => "SNOW",
    ExpressOnezone => "EXPRESS_ONEZONE",
    FsxOpenZfs => "FSX_OPENZFS"
}
impl<'de> Deserialize<'de> for StorageClass {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = StorageClass;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid storage class string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(StorageClass::from(value))
            }
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(StorageClass::from(v.as_str()))
            }
        }
        deserializer.deserialize_string(Visitor)
    }
}

impl Serialize for StorageClass {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
