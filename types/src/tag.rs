mod tag_type;
use http::{HeaderName, HeaderValue, header::InvalidHeaderValue};
pub use tag_type::*;
mod validation;
pub use validation::*;
#[cfg(feature = "headers")]
pub mod header;
use crate::S3ContentError;
mod extractor;
pub const TAGGING_HEADER: HeaderName = HeaderName::from_static("x-amz-tagging");
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum AnyTaggingSet<'a> {
    Borrowed(BorrowedTaggingSet<'a>),
    Cow(CowTaggingSet<'a>),
    Owned(OwnedTaggingSet),
}
impl<'a> From<BorrowedTaggingSet<'a>> for AnyTaggingSet<'a> {
    fn from(value: BorrowedTaggingSet<'a>) -> Self {
        AnyTaggingSet::Borrowed(value)
    }
}
impl<'a> From<CowTaggingSet<'a>> for AnyTaggingSet<'a> {
    fn from(value: CowTaggingSet<'a>) -> Self {
        AnyTaggingSet::Cow(value)
    }
}
impl From<OwnedTaggingSet> for AnyTaggingSet<'_> {
    fn from(value: OwnedTaggingSet) -> Self {
        AnyTaggingSet::Owned(value)
    }
}
impl<'a> AnyTaggingSet<'a> {
    pub fn to_header_value(&self) -> Result<HeaderValue, InvalidHeaderValue> {
        match self {
            AnyTaggingSet::Borrowed(tags) => tags.to_header_value(),
            AnyTaggingSet::Cow(tags) => tags.to_header_value(),
            AnyTaggingSet::Owned(tags) => tags.to_header_value(),
        }
    }
    pub fn to_xml_string(&self) -> Result<String, S3ContentError> {
        match self {
            AnyTaggingSet::Borrowed(tags) => Ok(quick_xml::se::to_string(tags)?),
            AnyTaggingSet::Cow(tags) => Ok(quick_xml::se::to_string(tags)?),
            AnyTaggingSet::Owned(tags) => Ok(quick_xml::se::to_string(tags)?),
        }
    }
}
pub type BorrowedTaggingSet<'a> = Tagging<BorrowedTag<'a>>;
pub type CowTaggingSet<'a> = Tagging<CowTag<'a>>;
pub type OwnedTaggingSet = Tagging<OwnedTag>;
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Tagging<Tag: TagType> {
    #[serde(rename = "TagSet")]
    pub tag_set: TagSet<Tag>,
}

impl<Tag: TagType> From<Vec<Tag>> for Tagging<Tag> {
    fn from(tags: Vec<Tag>) -> Self {
        Self {
            tag_set: TagSet::from(tags),
        }
    }
}
impl<Tag: TagType> From<Tagging<Tag>> for Vec<Tag> {
    fn from(tagging: Tagging<Tag>) -> Self {
        tagging.tag_set.tags
    }
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TagSet<Tag: TagType> {
    #[serde(rename = "Tag")]
    pub tags: Vec<Tag>,
}
impl<Tag: TagType> From<Vec<Tag>> for TagSet<Tag> {
    fn from(tags: Vec<Tag>) -> Self {
        Self { tags }
    }
}
impl<Tag: TagType> Tagging<Tag> {
    pub fn new(tag_set: Vec<Tag>) -> Self {
        Self {
            tag_set: TagSet::from(tag_set),
        }
    }

    pub fn add_tag(&mut self, tag: impl Into<Tag>) {
        let tag: Tag = tag.into();
        if let Some(index) = self.tag_set.tags.iter().position(|t| t.key() == tag.key()) {
            self.tag_set.tags[index] = tag;
        } else {
            self.tag_set.tags.push(tag);
        }
    }

    pub fn remove_tag(&mut self, key: &str) {
        self.tag_set.tags.retain(|tag| tag.key() != key);
    }
    pub fn get_tag(&self, key: &str) -> Option<&Tag> {
        self.tag_set.tags.iter().find(|tag| tag.key() == key)
    }
    pub fn has_tag(&self, key: &str) -> bool {
        self.tag_set.tags.iter().any(|tag| tag.key() == key)
    }
    pub fn to_header_value(&self) -> Result<HeaderValue, InvalidHeaderValue> {
        let length_estimate: usize = self
            .tag_set
            .tags
            .iter()
            .map(|tag| tag.key().len() + tag.value().len() + 2)
            .sum();
        let mut serializer =
            form_urlencoded::Serializer::new(String::with_capacity(length_estimate));
        for tag in &self.tag_set.tags {
            tag.append_pair(&mut serializer);
        }
        let result = serializer.finish();
        HeaderValue::from_str(&result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_serde_round_trip() {
        let tag = OwnedTag {
            key: "exampleKey".to_string(),
            value: "exampleValue".to_string(),
        };
        let tagging = OwnedTaggingSet {
            tag_set: TagSet { tags: vec![tag] },
        };

        let serialized = quick_xml::se::to_string(&tagging).unwrap();

        println!("Serialized Tagging: {}", serialized);
        let deserialized: OwnedTaggingSet = quick_xml::de::from_str(&serialized).unwrap();
        assert_eq!(deserialized.tag_set.tags.len(), 1);
        assert_eq!(deserialized.tag_set.tags[0].key, "exampleKey");
        assert_eq!(deserialized.tag_set.tags[0].value, "exampleValue");
    }
    #[test]
    fn test_from_amazon_example() {
        let xml_data = r#"
            <Tagging>
                <TagSet>
                    <Tag>
                        <Key>string</Key>
                        <Value>string</Value>
                    </Tag>
                </TagSet>
            </Tagging>
        "#;

        let tagging: OwnedTaggingSet = quick_xml::de::from_str(xml_data).unwrap();
        assert_eq!(tagging.tag_set.tags.len(), 1);
        assert_eq!(tagging.tag_set.tags[0].key, "string");
        assert_eq!(tagging.tag_set.tags[0].value, "string");
    }
}
