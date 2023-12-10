use crate::types::{NextId, QuestionId};
use serde::{Deserialize, Serialize};
use std::{io::ErrorKind, str::FromStr, sync::atomic::AtomicUsize};

/// Represents an answer id.
///
/// `AnswerId` is a wrapper around a String. It implements the `NextId` trait, which
/// allows us to generate a new id for each answer.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Id(String);

impl NextId for Id {
    fn counter() -> &'static AtomicUsize {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        &COUNTER
    }
}

impl FromStr for Id {
    type Err = std::io::Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        if id.is_empty() {
            Err(Self::Err::new(ErrorKind::InvalidInput, "No id provided"))
        } else {
            Ok(Id(id.to_string()))
        }
    }
}

/// Represents an answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Answer {
    /// The id of the answer.
    pub id: Id,
    /// The content of the answer.
    pub content: String,
    /// The id of the question this answer is associated with.
    pub question_id: QuestionId,
}

impl Answer {
    pub fn new(content: String, question_id: QuestionId) -> Self {
        Self {
            id: Id::next(),
            content,
            question_id,
        }
    }
}
