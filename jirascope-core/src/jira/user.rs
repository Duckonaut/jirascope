use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub active: bool,
    #[serde(rename = "accountId")]
    pub account_id: String,
    #[serde(rename = "accountType")]
    pub account_type: String,
}
