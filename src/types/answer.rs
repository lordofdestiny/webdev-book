use std::{io::ErrorKind, str::FromStr};

use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::Row;

use crate::types::QuestionId;

/// Represents an answer id.
///
/// `AnswerId` is a wrapper around a String. It implements the `NextId` trait, which
/// allows us to generate a new id for each answer.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnswerId(pub i32);

/// Represents an answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Answer {
    /// The id of the answer.
    pub id: AnswerId,
    /// The content of the answer.
    pub content: String,
    /// The id of the question this answer is associated with.
    pub question_id: QuestionId,
}

impl FromStr for AnswerId {
    type Err = std::io::Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        if id.is_empty() {
            Err(Self::Err::new(ErrorKind::InvalidInput, "No id provided"))
        } else {
            match id.parse() {
                Ok(id) => Ok(AnswerId(id)),
                Err(_) => Err(Self::Err::new(ErrorKind::InvalidInput, "Invalid id format")),
            }
        }
    }
}

impl TryFrom<PgRow> for Answer {
    type Error = sqlx::Error;

    fn try_from(row: PgRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: AnswerId(row.try_get("id")?),
            content: row.try_get("content")?,
            question_id: QuestionId(row.try_get("question_id")?),
        })
    }
}

/// Represents a new answer.
///
/// This struct is used to create a new answer. It is used in the `POST /questions/:id/answers` route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAnswer {
    pub content: String,
}
