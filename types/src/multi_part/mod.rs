use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitiateMultipartUploadResult {
    #[serde(rename = "Bucket")]
    pub bucket: Option<String>,
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "UploadId")]
    pub upload_id: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Part {
    #[serde(rename = "PartNumber")]
    pub number: u32,
    #[serde(rename = "ETag")]
    pub etag: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompleteMultipartUpload {
    #[serde(rename = "Part")]
    pub parts: Vec<Part>,
}

#[cfg(test)]
mod tests {
    use crate::multi_part::{CompleteMultipartUpload, Part};
    #[test]
    pub fn complete_serialize_deserialize() {
        let part = Part {
            number: 1,
            etag: "etag1".into(),
        };
        let part_two = Part {
            number: 2,
            etag: "etag2".into(),
        };
        let upload = CompleteMultipartUpload {
            parts: vec![part, part_two],
        };
        let serialized = quick_xml::se::to_string(&upload).unwrap();
        println!("Serialized: \n {}", serialized);
        let deserialized: CompleteMultipartUpload = quick_xml::de::from_str(&serialized).unwrap();
        assert_eq!(upload, deserialized);
    }
}
