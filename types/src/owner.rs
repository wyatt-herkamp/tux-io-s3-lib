use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Owner {
    pub display_name: Option<String>,
    #[serde(rename = "ID")]
    pub id: String,
}
