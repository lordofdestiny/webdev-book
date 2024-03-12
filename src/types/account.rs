use std::io::ErrorKind;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use macros::DbObjectId;

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

