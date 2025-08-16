use std::{str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};
use url::Url;
mod official;
pub use official::OfficialRegion;

use crate::signature::SignatureVersions;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum S3Implementation {
    AWS,
    GenericCustom,
}
pub trait RegionType {
    /// Returns the name of the region.
    fn name(&self) -> &str;
    /// Returns the endpoint of the region.
    fn endpoint(&self) -> &str;
    /// Returns the URL for the region.
    fn endpoint_url(&self) -> Url;
    /// Returns the schema of the region (e.g., "https").
    fn schema(&self) -> &str {
        "https"
    }
    fn supported_signature_versions(&self) -> Vec<SignatureVersions> {
        vec![SignatureVersions::V4]
    }
    fn implementation(&self) -> S3Implementation {
        S3Implementation::AWS
    }
}
impl<R: RegionType> RegionType for &R {
    fn name(&self) -> &str {
        R::name(self)
    }
    fn endpoint(&self) -> &str {
        R::endpoint(self)
    }
    fn endpoint_url(&self) -> Url {
        R::endpoint_url(self)
    }
    fn schema(&self) -> &str {
        R::schema(self)
    }
    fn supported_signature_versions(&self) -> Vec<SignatureVersions> {
        R::supported_signature_versions(self)
    }
    fn implementation(&self) -> S3Implementation {
        R::implementation(self)
    }
}
impl<R: RegionType> RegionType for Arc<R> {
    fn name(&self) -> &str {
        R::name(self)
    }
    fn endpoint(&self) -> &str {
        R::endpoint(self)
    }
    fn endpoint_url(&self) -> Url {
        R::endpoint_url(self)
    }
    fn schema(&self) -> &str {
        R::schema(self)
    }
    fn supported_signature_versions(&self) -> Vec<SignatureVersions> {
        R::supported_signature_versions(self)
    }
    fn implementation(&self) -> S3Implementation {
        R::implementation(self)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct CustomRegion {
    pub endpoint: Url,
    pub name: Option<String>,
}
impl RegionType for CustomRegion {
    fn name(&self) -> &str {
        self.name.as_deref().unwrap_or_else(|| {
            self.endpoint
                .host_str()
                .unwrap_or_else(|| self.endpoint.as_str())
        })
    }

    fn endpoint(&self) -> &str {
        self.endpoint
            .host_str()
            .unwrap_or_else(|| self.endpoint.as_str())
    }
    fn schema(&self) -> &str {
        self.endpoint.scheme()
    }
    fn endpoint_url(&self) -> Url {
        self.endpoint.clone()
    }

    fn supported_signature_versions(&self) -> Vec<SignatureVersions> {
        vec![SignatureVersions::V4]
    }
}
impl FromStr for CustomRegion {
    type Err = url::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let endpoint = Url::parse(s)?;
        Ok(Self {
            endpoint,
            name: None,
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum S3Region {
    Official(OfficialRegion),
    Custom(CustomRegion),
}
impl Default for S3Region {
    fn default() -> Self {
        S3Region::Official(OfficialRegion::default())
    }
}
impl RegionType for S3Region {
    fn name(&self) -> &str {
        match self {
            S3Region::Official(region) => region.name(),
            S3Region::Custom(region) => region.name(),
        }
    }

    fn endpoint(&self) -> &str {
        match self {
            S3Region::Official(region) => region.endpoint(),
            S3Region::Custom(region) => region.endpoint(),
        }
    }
    fn schema(&self) -> &str {
        match self {
            S3Region::Official(region) => region.endpoint().split("://").next().unwrap_or("https"),
            S3Region::Custom(region) => region.endpoint().split("://").next().unwrap_or("https"),
        }
    }

    fn endpoint_url(&self) -> Url {
        match self {
            S3Region::Official(region) => region.endpoint_url(),
            S3Region::Custom(region) => region.endpoint_url(),
        }
    }

    fn supported_signature_versions(&self) -> Vec<SignatureVersions> {
        match self {
            S3Region::Official(region) => region.supported_signature_versions(),
            S3Region::Custom(region) => region.supported_signature_versions(),
        }
    }
}

struct S3RegionVisitor;
impl<'de> serde::de::Visitor<'de> for S3RegionVisitor {
    type Value = S3Region;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid S3 region")
    }
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Ok(region) = OfficialRegion::from_str(v) {
            Ok(S3Region::Official(region))
        } else if let Ok(region) = CustomRegion::from_str(v) {
            Ok(S3Region::Custom(region))
        } else {
            Err(E::custom(format!("invalid S3 region: {}", v)))
        }
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut endpoint = None;
        let mut name = None;
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "endpoint" => {
                    if endpoint.is_some() {
                        return Err(serde::de::Error::duplicate_field("endpoint"));
                    }
                    endpoint = Some(map.next_value()?);
                }
                "name" => {
                    if name.is_some() {
                        return Err(serde::de::Error::duplicate_field("name"));
                    }
                    name = Some(map.next_value()?);
                }
                _ => return Err(serde::de::Error::unknown_field(&key, &["endpoint", "name"])),
            }
        }
        let region = match (endpoint, name) {
            (Some(endpoint), Some(name)) => Ok(S3Region::Custom(CustomRegion {
                endpoint,
                name: Some(name),
            })),
            (Some(endpoint), None) => Ok(S3Region::Custom(CustomRegion {
                endpoint,
                name: None,
            })),
            (None, Some(name)) => {
                let region = OfficialRegion::from_str(&name);
                match region {
                    Ok(region) => Ok(S3Region::Official(region)),
                    Err(_) => Err(serde::de::Error::custom(format!(
                        "invalid official region name: {} or please provide an endpoint for custom a region",
                        name
                    ))),
                }
            }
            _ => Err(serde::de::Error::missing_field("endpoint")),
        }?;

        Ok(region)
    }
}
impl<'de> serde::Deserialize<'de> for S3Region {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(S3RegionVisitor)
    }
}
impl serde::Serialize for S3Region {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            S3Region::Official(region) => region.serialize(serializer),
            S3Region::Custom(region) => region.serialize(serializer),
        }
    }
}
