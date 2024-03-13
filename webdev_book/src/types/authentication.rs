use chrono::{DateTime, Utc};
use macros::DbObjectId;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::Row;

/// Represents an answer id.
///
/// `AccountId` is a wrapper around a i32. It represents the id of an account.
#[derive(DbObjectId, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountId(pub i32);

/// Represents an account.
///
/// `Account` is a struct that represents an account. It contains the id, email, and password of the account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// The id of the account.
    ///
    /// Is an `Option` because the id is not known when creating a new account.
    pub id: Option<AccountId>,
    /// The email of the account.
    pub email: String,
    /// The password of the account.
    ///
    /// Password can be plain text or hashed,
    /// depending if the account is being created or retrieved from the database.
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

/// Represents a session.
///
/// `Session` is a struct that represents a session.
/// It contains the expiration date, not before date, and the account id of the session.   
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// The expiration date of the session.
    pub exp: DateTime<Utc>,
    /// The not before date of the session.
    pub nbf: DateTime<Utc>,
    /// The account id associated with the session.
    pub account_id: AccountId,
}
