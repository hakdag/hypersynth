use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountType {
    Personal,
    Company,
}

impl AccountType {
    pub fn as_db_value(self) -> &'static str {
        match self {
            AccountType::Personal => "personal",
            AccountType::Company => "company",
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
