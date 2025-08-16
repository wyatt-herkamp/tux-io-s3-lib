use crate::region::{RegionType, S3Implementation};
use std::fmt::Display;
use std::str::FromStr;
use url::Url;
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Invalid region: {0}")]
pub struct InvalidRegionError(pub String);
impl From<String> for InvalidRegionError {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl From<&str> for InvalidRegionError {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}
macro_rules! official_region {
    (
        $(
            $(#[$docs:meta])*
            $name:ident{
                endpoint: $endpoint:literal,
                key: $key:literal
            }
        ),*
    ) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum OfficialRegion {
            $(
                $(#[$docs])*
                $name,
            )*
        }
        impl RegionType for OfficialRegion {
            fn name(&self) -> &str {
                match self {
                    $(OfficialRegion::$name => $key,)*
                }
            }
            fn schema(&self) -> &str {
                return "https";
            }
            fn endpoint(&self) -> &str {
                match self {
                    $(OfficialRegion::$name => $endpoint,)*
                }
            }
            fn endpoint_url(&self) -> Url {
                match self {
                    $(OfficialRegion::$name => Url::parse(concat!("https://",$endpoint)).unwrap(),)*
                }
            }
            fn implementation(&self) -> S3Implementation {
                S3Implementation::AWS
            }
        }
        impl FromStr for OfficialRegion {
            type Err = InvalidRegionError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        $key | $endpoint | concat!("https://", $endpoint) => Ok(OfficialRegion::$name),
                    )*
                    _ => Err(InvalidRegionError::from(s)),
                }
            }
        }
        impl Display for OfficialRegion {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(OfficialRegion::$name => write!(f, "{}", $key),)*
                }
            }
        }
    };
}

official_region!(
    /// The US East (N. Virginia) region.
    UsEast1 {
        endpoint: "s3.amazonaws.com",
        key: "us-east-1"
    }
);
#[allow(clippy::derivable_impls)]
impl Default for OfficialRegion {
    fn default() -> Self {
        OfficialRegion::UsEast1
    }
}
struct OfficialRegionVisitor;
impl<'de> serde::de::Visitor<'de> for OfficialRegionVisitor {
    type Value = OfficialRegion;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid official region name or endpoint")
    }
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        OfficialRegion::from_str(v).map_err(E::custom)
    }
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        OfficialRegion::from_str(&v).map_err(E::custom)
    }
    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        OfficialRegion::from_str(v).map_err(E::custom)
    }
}

impl<'de> serde::Deserialize<'de> for OfficialRegion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(OfficialRegionVisitor)
    }
}

impl serde::Serialize for OfficialRegion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.name())
    }
}
