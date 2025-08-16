use std::borrow::Cow;

use form_urlencoded::{Serializer as UrlSerializer, Target};
use serde::{Serialize, ser::SerializeStruct};
pub trait TagType: Serialize {
    /// Returns the key of the tag.
    fn key(&self) -> &str;
    /// Returns the value of the tag.
    fn value(&self) -> &str;

    fn append_pair<T: Target>(&self, serializer: &mut UrlSerializer<'_, T>) {
        serializer.append_pair(self.key(), self.value());
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct OwnedTag {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Value")]
    pub value: String,
}
impl Serialize for OwnedTag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Tag", 2)?;
        state.serialize_field("Key", &self.key)?;
        state.serialize_field("Value", &self.value)?;
        state.end()
    }
}
impl TagType for OwnedTag {
    fn key(&self) -> &str {
        &self.key
    }

    fn value(&self) -> &str {
        &self.value
    }
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct CowTag<'a> {
    #[serde(rename = "Key")]
    pub key: Cow<'a, str>,
    #[serde(rename = "Value")]
    pub value: Cow<'a, str>,
}
impl Serialize for CowTag<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Tag", 2)?;
        state.serialize_field("Key", &self.key)?;
        state.serialize_field("Value", &self.value)?;
        state.end()
    }
}
impl<'a> TagType for CowTag<'a> {
    fn key(&self) -> &str {
        &self.key
    }

    fn value(&self) -> &str {
        &self.value
    }
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct BorrowedTag<'a> {
    pub key: &'a str,
    pub value: &'a str,
}
impl<'a> BorrowedTag<'a> {
    pub fn new(key: &'a str, value: &'a str) -> Self {
        Self { key, value }
    }
}
impl Serialize for BorrowedTag<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Tag", 2)?;
        state.serialize_field("Key", &self.key)?;
        state.serialize_field("Value", &self.value)?;
        state.end()
    }
}
impl<'a> TagType for BorrowedTag<'a> {
    fn key(&self) -> &str {
        self.key
    }
    fn value(&self) -> &str {
        self.value
    }
}
impl<'a> From<(&'a str, &'a str)> for BorrowedTag<'a> {
    fn from((key, value): (&'a str, &'a str)) -> Self {
        Self { key, value }
    }
}
impl<'a> From<(&'a str, &'a str)> for CowTag<'a> {
    fn from((key, value): (&'a str, &'a str)) -> Self {
        Self {
            key: Cow::Borrowed(key),
            value: Cow::Borrowed(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_owned_tag() {
        let tag = OwnedTag {
            key: "exampleKey".to_string(),
            value: "exampleValue".to_string(),
        };
        assert_eq!(tag.key(), "exampleKey");
        assert_eq!(tag.value(), "exampleValue");
    }

    #[test]
    fn test_cow_tag() {
        let tag = CowTag {
            key: Cow::Borrowed("exampleKey"),
            value: Cow::Borrowed("exampleValue"),
        };
        assert_eq!(tag.key(), "exampleKey");
        assert_eq!(tag.value(), "exampleValue");
    }

    #[test]
    fn test_borrowed_tag() {
        let tag = BorrowedTag {
            key: "exampleKey",
            value: "exampleValue",
        };
        let serialized = quick_xml::se::to_string(&tag).unwrap();
        println!("Serialized BorrowedTag: {}", serialized);
        let deserialized: BorrowedTag = quick_xml::de::from_str(&serialized).unwrap();
        assert_eq!(deserialized.key, "exampleKey");
        assert_eq!(deserialized.value, "exampleValue");
    }
}
