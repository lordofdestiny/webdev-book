use crate::types::{NextId, QuestionId};
use serde::{Deserialize, Serialize};
use std::{io::ErrorKind, str::FromStr, sync::atomic::AtomicUsize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Answer {
    pub id: AnswerId,
    pub content: String,
    pub question_id: QuestionId,
}
