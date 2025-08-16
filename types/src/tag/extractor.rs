use crate::{DataExtract, tag::OwnedTaggingSet};

impl DataExtract for OwnedTaggingSet {
    fn extract_data<R: std::io::BufRead>(reader: &mut R) -> Result<Self, crate::ContentParseError> {
        let tagging: OwnedTaggingSet = quick_xml::de::from_reader(reader)?;
        Ok(tagging)
    }
}
