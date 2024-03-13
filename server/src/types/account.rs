use macros::DbObjectId;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::Row;

/// Represents an answer id.
///
/// `AccountId` is a wrapper around a i32. It represents the id of an account.
#[derive(DbObjectId, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountId(pub i32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Option<AccountId>,
    pub email: String,
    pub password: String,
}

impl TryFrom<PgRow> for Account {
    type Error = sqlx::Error;

    fn try_from(row: PgRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Some(AccountId(row.try_get("id")?)),
            email: row.try_get("email")?,
            password: row.try_get("password")?,
        })
    }
}
