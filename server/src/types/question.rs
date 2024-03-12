use macros::DbObjectId;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::Row;

/// Represents a question id.
///
/// `QuestionId` is a wrapper around an i32. It represents the id of a question.
#[derive(DbObjectId, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuestionId(pub i32);

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents a question.
///
pub struct Question {
    /// The id of the question. It is an `Option<QuestionId>` because we want to be able to
    /// create a question by parsing a JSON object that doesn't have an id field.
    pub id: Option<QuestionId>,
    /// The title of the question.
    pub title: String,
    /// The content of the question.
    pub content: String,
    /// The tags of the question.
    pub tags: Option<Vec<String>>,
}

impl TryFrom<PgRow> for Question {
    type Error = sqlx::Error;
    fn try_from(value: PgRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Some(QuestionId(value.try_get("id")?)),
            title: value.try_get("title")?,
            content: value.try_get("content")?,
            tags: value.try_get("tags")?,
        })
    }
}
