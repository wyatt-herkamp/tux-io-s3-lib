use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListAllMyBuckets {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<String>,
    pub buckets: Buckets,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Buckets {
    #[serde(rename = "Bucket")]
    pub buckets: Vec<Bucket>,
}
impl From<Vec<Bucket>> for Buckets {
    fn from(buckets: Vec<Bucket>) -> Self {
        Self { buckets }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Bucket {
    pub creation_date: String,
    pub name: String,
    #[serde(rename = "BucketRegion", skip_serializing_if = "Option::is_none")]
    pub bucket_region: Option<String>,
}
#[cfg(test)]
mod tests {
    use crate::list::buckets::{Bucket, Buckets, ListAllMyBuckets};

    #[test]
    fn tests() {
        let buckets = vec![
            Bucket {
                creation_date: "2022-01-01".into(),
                name: "bucket1".into(),
                bucket_region: Some("us-east-1".into()),
            },
            Bucket {
                creation_date: "2022-01-02".into(),
                name: "bucket2".into(),
                bucket_region: Some("us-west-1".into()),
            },
        ];
        let buckets = Buckets::from(buckets);
        let list_buckets = ListAllMyBuckets {
            continuation_token: None,
            buckets,
            prefix: None,
        };
        let to_xml = quick_xml::se::to_string(&list_buckets).unwrap();
        println!("{}", to_xml);
        let from_xml: ListAllMyBuckets = quick_xml::de::from_str(&to_xml).unwrap();
        assert_eq!(list_buckets, from_xml);
    }
}
