use crate::types::NextId;
use serde::{Deserialize, Serialize};
use std::{io::ErrorKind, str::FromStr, sync::atomic::AtomicUsize};

/// Represents a question id.
///
/// QuestionId is a wrapper around a String. It implements the NextId trait, which
/// allows us to generate a new id for each question.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuestionId(String);

impl NextId for QuestionId {
    fn counter() -> &'static AtomicUsize {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        &COUNTER
    }
}

impl FromStr for QuestionId {
    type Err = std::io::Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        match id.is_empty() {
            false => Ok(QuestionId(id.to_string())),
            true => Err(Self::Err::new(ErrorKind::InvalidInput, "No id provided")),
        }
    }
}

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
