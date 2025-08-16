use serde::{Deserialize, Serialize};

use crate::list::{object::Object, prefix::CommonPrefixes};
mod extractor;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListBucketResult {
    pub is_truncated: bool,
    pub max_keys: Option<i32>,
    pub name: String,
    pub next_continuation_token: Option<String>,
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub start_after: Option<String>,
    pub contents: Option<Vec<Object>>,
    pub common_prefixes: Option<CommonPrefixes>,
}
