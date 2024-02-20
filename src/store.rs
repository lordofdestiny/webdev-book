//! This module contains the store, which is a simple in-memory database.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::types::{Answer, Id, Question, QuestionId};

/// This struct represents the store, which is a simple in-memory database.
///
/// The store contains two maps: one for questions and one for answers.
/// The maps are wrapped in an `Arc` and a `RwLock` to allow for concurrent access.
#[derive(Debug, Clone)]
pub struct Store {
    /// This map contains all the questions. The key is the question
    /// ID and the value is the question.
    pub questions: Arc<RwLock<HashMap<QuestionId, Question>>>,
    /// This map contains all the answers. The key is the answer ID
    /// and the value is the answer.
    pub answers: Arc<RwLock<HashMap<Id, Answer>>>,
}

impl Store {
    /// This function creates a new store.
    ///
    /// The store is initialized with the questions from the `questions.json` file
    /// in the root of the project by calling the `init` function.
    pub fn new() -> Store {
        Store {
            questions: Arc::new(RwLock::new(Self::init())),
            answers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// This function initializes the store with the questions from the `questions.json` file.
    /// File is located in the root of the project.
    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/questions.json"));
        serde_json::from_str(file).expect("Failed to parse questions.json")
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}
