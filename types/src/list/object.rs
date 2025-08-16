use crate::owner::Owner;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Object {
    pub last_modified: String,
    pub e_tag: Option<String>,
    pub storage_class: Option<String>,
    pub key: String,
    pub owner: Option<Owner>,
    pub size: u64,
}
