use crate::types::NextId;
use serde::{Deserialize, Serialize};
use std::{io::ErrorKind, str::FromStr, sync::atomic::AtomicUsize};

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
pub struct Question {
    pub id: Option<QuestionId>,
    pub title: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
}
