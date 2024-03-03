use std::{io::ErrorKind, str::FromStr};
use std::fmt::Display;

use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::Row;

/// Represents a question id.
///
/// `QuestionId` is a wrapper around a String. It implements the `NextId` trait, which
/// allows us to generate a new id for each question.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuestionId(pub i32);

impl Display for QuestionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents a question.
///
pub struct Question {
    /// The id of the question. It is an `Option<QuestionId>` because we want to be able to
    /// create a question by parsing a JSON object that doesn't have an id field.
    pub id: QuestionId,
    /// The title of the question.
    pub title: String,
    /// The content of the question.
    pub content: String,
    /// The tags of the question.
    pub tags: Option<Vec<String>>,
}

impl FromStr for QuestionId {
    type Err = std::io::Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        if id.is_empty() {
            Err(Self::Err::new(ErrorKind::InvalidInput, "No id provided"))
        } else {
            match id.parse() {
                Ok(id) => Ok(QuestionId(id)),
                Err(_) => Err(Self::Err::new(ErrorKind::InvalidInput, "Invalid id format")),
            }
        }
    }
}

impl TryFrom<PgRow> for Question {
    type Error = sqlx::Error;
    fn try_from(value: PgRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: QuestionId(value.try_get("id")?),
            title: value.try_get("title")?,
            content: value.try_get("content")?,
            tags: value.try_get("tags")?,
        })
    }
}

/// Represents a new question.
///
/// This struct is used to create a new question. It is used in the `POST /questions` route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewQuestion {
    pub title: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
}
