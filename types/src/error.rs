use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize, ser::SerializeMap};
#[derive(Debug)]
pub struct Error {
    pub code: String,
    pub message: Option<String>,
    pub request_id: Option<String>,
    pub host_id: Option<String>,
    pub attributes: HashMap<String, String>,
}
impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let len = self.attributes.len()
            + 1
            + if self.message.is_some() { 1 } else { 0 }
            + if self.request_id.is_some() { 1 } else { 0 }
            + if self.host_id.is_some() { 1 } else { 0 };
        let mut state = serializer.serialize_map(Some(len))?;
        state.serialize_entry("Code", &self.code)?;
        if let Some(ref message) = self.message {
            state.serialize_entry("Message", message)?;
        }
        if let Some(ref request_id) = self.request_id {
            state.serialize_entry("RequestId", request_id)?;
        }
        if let Some(ref host_id) = self.host_id {
            state.serialize_entry("HostId", host_id)?;
        }
        for (key, value) in &self.attributes {
            state.serialize_entry(key, value)?;
        }
        state.end()
    }
}
struct ErrorVisitor;
impl<'de> serde::de::Visitor<'de> for ErrorVisitor {
    type Value = Error;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an S3 error response")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut code = None;
        let mut message = None;
        let mut request_id = None;
        let mut host_id = None;
        let mut attributes = HashMap::new();
        while let Some((key, value)) = map.next_entry::<String, String>()? {
            match key.as_str() {
                "Code" => {
                    code = Some(value);
                }
                "Message" => {
                    message = Some(value);
                }
                "RequestId" => {
                    request_id = Some(value);
                }
                "HostId" => {
                    host_id = Some(value);
                }
                _ => {
                    attributes.insert(key, value);
                }
            }
        }
        Ok(Error {
            code: code.ok_or_else(|| serde::de::Error::missing_field("Code"))?,
            message,
            request_id,
            host_id,
            attributes,
        })
    }
}

impl<'de> Deserialize<'de> for Error {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ErrorVisitor)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorAttribute {
    #[serde(rename = "$text")]
    pub key: String,
    #[serde(rename = "$value")]
    pub value: String,
}
#[cfg(test)]
mod tests {
    #[test]
    fn deserialize_error() {
        let xml = r#"
<?xml version="1.0" encoding="UTF-8"?>
<Error>
	<Code>NoSuchBucket</Code>
	<Message>The specified bucket does not exist</Message>
	<Key>path/to/object</Key>
	<BucketName>tests</BucketName>
	<Resource>/tests/path/to/object</Resource>
	<RequestId>18788A1CB29086D9</RequestId>
	<HostId>dd9025bab4ad464b049177c95eb6ebf374d3b3fd1af9251148b658df7ac2e3e8</HostId>
</Error>"#;

        let error: super::Error = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(error.code, "NoSuchBucket");
    }
}
