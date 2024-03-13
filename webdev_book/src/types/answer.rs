use macros::DbObjectId;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::Row;

use crate::types::question::QuestionId;

/// Represents an answer id.
///
/// `AnswerId` is a wrapper around an i32. It represents the id of an answer.
#[derive(DbObjectId, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnswerId(pub i32);

/// Represents an answer.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Answer {
    /// The id of the answer.
    pub id: Option<AnswerId>,
    /// The content of the answer.
    pub content: String,
    /// The id of the question this answer is associated with.
    pub question_id: Option<QuestionId>,
}

impl TryFrom<PgRow> for Answer {
    type Error = sqlx::Error;

    fn try_from(row: PgRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Some(AnswerId(row.try_get("id")?)),
            content: row.try_get("content")?,
            question_id: Some(QuestionId(row.try_get("question_id")?)),
        })
    }
}
