use crate::{DataExtract, list::v1::ListBucketResult};

impl DataExtract for ListBucketResult {
    fn extract_data<R: std::io::BufRead>(reader: &mut R) -> Result<Self, crate::S3ContentError> {
        let result: ListBucketResult = quick_xml::de::from_reader(reader)?;
        Ok(result)
    }
}
