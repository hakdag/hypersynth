use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
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
}
