use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    Personal,
    Company,
    #[serde(rename = "system_admin")]
    SystemAdmin,
}

impl AccountType {
    pub fn as_db_value(self) -> &'static str {
        match self {
            AccountType::Personal => "personal",
            AccountType::Company => "company",
            AccountType::SystemAdmin => {
                panic!("system_admin is session-only and must not be stored in users.account_type")
            }
        }
    }

    pub fn from_db_value(value: &str) -> Option<Self> {
        match value {
            "personal" => Some(AccountType::Personal),
            "company" => Some(AccountType::Company),
            _ => None,
        }
    }
}
