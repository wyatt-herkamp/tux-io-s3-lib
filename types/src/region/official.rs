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
    },
        /// us-east-2
    UsEast2{
        endpoint: "s3.us-east-2.amazonaws.com",
        key: "us-east-2"
    },
    /// us-west-1
    UsWest1{
        endpoint: "s3.us-west-1.amazonaws.com",
        key: "us-west-1"
    },
    /// us-west-2
    UsWest2{
        endpoint: "s3.us-west-2.amazonaws.com",
        key: "us-west-2"
    },
    /// ca-central-1
    CaCentral1{
        endpoint: "s3.ca-central-1.amazonaws.com",
        key: "ca-central-1"
    },
    /// af-south-1
    AfSouth1{
        endpoint: "s3.af-south-1.amazonaws.com",
        key: "af-south-1"
    },
    /// ap-east-1
    ApEast1{
        endpoint: "s3.ap-east-1.amazonaws.com",
        key: "ap-east-1"
    },
    /// ap-south-1
    ApSouth1{
        endpoint: "s3.ap-south-1.amazonaws.com",
        key: "ap-south-1"
    },
    /// ap-northeast-1
    ApNortheast1{
        endpoint: "s3.ap-northeast-1.amazonaws.com",
        key: "ap-northeast-1"
    },
    /// ap-northeast-2
    ApNortheast2{
        endpoint: "s3.ap-northeast-2.amazonaws.com",
        key: "ap-northeast-2"
    },
    /// ap-northeast-3
    ApNortheast3{
        endpoint: "s3.ap-northeast-3.amazonaws.com",
        key: "ap-northeast-3"
    },
    /// ap-southeast-1
    ApSoutheast1{
        endpoint: "s3.ap-southeast-1.amazonaws.com",
        key: "ap-southeast-1"
    },
    /// ap-southeast-2
    ApSoutheast2{
        endpoint: "s3.ap-southeast-2.amazonaws.com",
        key: "ap-southeast-2"
    },
    /// cn-north-1
    CnNorth1{
        endpoint: "s3.cn-north-1.amazonaws.com",
        key: "cn-north-1"
    },
    /// cn-northwest-1
    CnNorthwest1{
        endpoint: "s3.cn-northwest-1.amazonaws.com",
        key: "cn-northwest-1"
    },
    /// eu-north-1
    EuNorth1{
        endpoint: "s3.eu-north-1.amazonaws.com",
        key: "eu-north-1"
    },
    /// eu-central-1
    EuCentral1{
        endpoint: "s3.eu-central-1.amazonaws.com",
        key: "eu-central-1"
    },
    /// eu-central-2
    EuCentral2{
        endpoint: "s3.eu-central-2.amazonaws.com",
        key: "eu-central-2"
    },
    /// eu-west-1
    EuWest1{
        endpoint: "s3.eu-west-1.amazonaws.com",
        key: "eu-west-1"
    },
    /// eu-west-2
    EuWest2{
        endpoint: "s3.eu-west-2.amazonaws.com",
        key: "eu-west-2"
    },
    /// eu-west-3
    EuWest3{
        endpoint: "s3.eu-west-3.amazonaws.com",
        key: "eu-west-3"
    },
    /// il-central-1
    IlCentral1{
        endpoint: "s3.il-central-1.amazonaws.com",
        key: "il-central-1"
    },
    /// me-south-1
    MeSouth1{
        endpoint: "s3.me-south-1.amazonaws.com",
        key: "me-south-1"
    },
    /// sa-east-1
    SaEast1{
        endpoint: "s3.sa-east-1.amazonaws.com",
        key: "sa-east-1"
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
