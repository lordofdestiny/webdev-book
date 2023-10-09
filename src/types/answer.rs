use crate::types::{NextId, QuestionId};
use serde::{Deserialize, Serialize};
use std::{io::ErrorKind, str::FromStr, sync::atomic::AtomicUsize};

/// Represents an answer id.
///
/// AnswerId is a wrapper around a String. It implements the NextId trait, which
/// allows us to generate a new id for each answer.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnswerId(String);

impl NextId for AnswerId {
    fn counter() -> &'static AtomicUsize {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        &COUNTER
    }
}

impl FromStr for AnswerId {
    type Err = std::io::Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        match id.is_empty() {
            false => Ok(AnswerId(id.to_string())),
            true => Err(Self::Err::new(ErrorKind::InvalidInput, "No id provided")),
        }
    }
}

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

impl Answer {
    pub fn new(content: String, question_id: QuestionId) -> Self {
        Self {
            id: AnswerId::next(),
            content,
            question_id,
        }
    }
}
