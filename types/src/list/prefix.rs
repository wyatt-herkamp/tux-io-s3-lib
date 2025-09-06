use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CommonPrefixes {
    pub prefix: Vec<String>,
}
impl Deref for CommonPrefixes {
    type Target = Vec<String>;
    fn deref(&self) -> &Self::Target {
        &self.prefix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_prefixes() {
        let common_prefixes = CommonPrefixes {
            prefix: vec!["prefix1/".to_string(), "prefix2/".to_string()],
        };
        let serialize = quick_xml::se::to_string(&common_prefixes).unwrap();
        println!("Serialized CommonPrefixes: \n{}", serialize);
    }
}
