//! This module contains the store, which is a simple in-memory database.

use std::fmt::Display;
use std::sync::Arc;

use sqlx::postgres::{PgPool, PgPoolOptions};
use tokio::sync::Mutex;

use crate::api::bad_words::BadWordsAPI;
use crate::types::{Answer, NewQuestion, Pagination, Question};

/// This struct represents the store, which is a simple in-memory database.
///
/// The store contains two maps: one for questions and one for answers.
/// The maps are wrapped in an `Arc` and a `RwLock` to allow for concurrent access.
#[derive(Clone)]
pub struct Store {
    pub connection: PgPool,
    pub bad_words_api: Arc<Mutex<BadWordsAPI>>,
}

impl std::fmt::Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Store").field("connection", &self.connection).finish()
    }
}

impl Store {
    /// This function creates a new store.
    ///
    /// # Arguments
    /// - `db_url`: A string slice that contains the URL for the database.
    ///
    /// # Returns
    /// - A store if the database connection was established successfully.
    ///
    /// # Panics
    /// - If the database connection cannot be established
    pub async fn new(db_url: &str) -> Self {
        let db_pool = match PgPoolOptions::new().max_connections(5).connect(db_url).await {
            Ok(pool) => pool,
            Err(e) => panic!("Couldn't establish DB connection: {}", e),
        };

        let api_layer_key = std::env!("API_LAYER_KEY", "API_LAYER_KEY not found");
        let bad_words_api = BadWordsAPI::build(api_layer_key, '*').expect("Couldn't build BadWordsAPI");

        Store {
            connection: db_pool,
            bad_words_api: Arc::new(Mutex::new(bad_words_api)),
        }
    }

    /// This function returns all questions from the table `questions`.
    ///
    /// # Arguments
    /// - `pag`: A `Pagination` struct that contains the offset and limit for the query.
    ///
    /// # Returns
    /// - A vector of questions if the questions were found successfully.
    /// - An error if the questions could not be found.
    pub async fn get_questions(&self, pag: Pagination) -> Result<Vec<Question>, sqlx::Error> {
        let Pagination { offset, limit } = pag;

        match sqlx::query("SELECT * FROM questions LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .map(Question::try_from)
            .fetch_all(&self.connection)
            .await?
            .into_iter()
            .collect()
        {
            Ok(questions) => Ok(questions),
            Err(e) => {
                tracing::event!(target:"webdev_book", tracing::Level::ERROR, "{:?}", e);
                Err(e)
            }
        }
    }

    /// This function returns a question from the table `questions` by its ID.
    ///
    /// # Arguments
    /// - `id`: An integer that represents the ID of the question.
    ///
    /// # Returns
    /// - A Question if the question was found successfully.
    /// - An error if the question could not be found.
    pub async fn get_question(&self, id: i32) -> Result<Question, sqlx::Error> {
        match sqlx::query("SELECT * FROM questions WHERE id = $1")
            .bind(id)
            .map(Question::try_from)
            .fetch_one(&self.connection)
            .await?
        {
            Ok(question) => Ok(question),
            Err(e) => {
                tracing::event!(target:"webdev_book", tracing::Level::ERROR, "{:?}", e);
                Err(e)
            }
        }
    }

    /// This function will insert a question into the table `questions`
    ///
    /// # Arguments
    /// - `question_id`: An integer that represents the ID of the question.
    ///
    /// # Returns
    /// - A new Question if the question was added successfully.
    /// - An error if the question could not be added.
    pub async fn add_question(&self, new_question: NewQuestion) -> Result<Question, sqlx::Error> {
        let NewQuestion { title, content, tags } = new_question;

        match sqlx::query(
            "INSERT INTO questions (title, content, tags)\
            VALUES ($1, $2, $3)\
            RETURNING *",
        )
        .bind(title)
        .bind(content)
        .bind(tags)
        .map(Question::try_from)
        .fetch_one(&self.connection)
        .await?
        {
            Ok(question) => Ok(question),
            Err(e) => {
                tracing::event!(target:"webdev_book", tracing::Level::ERROR, "{:?}", e);
                Err(e)
            }
        }
    }

    /// This function will update a question in the table `questions` by its ID
    ///
    /// # Arguments
    /// - `question`: A `Question` struct that contains the new data for the question.
    /// - `question_id`: An integer that represents the ID of the question.
    ///
    /// # Returns
    /// - An updated Question if the question was updated successfully.
    /// - An error if the question could not be updated.
    pub async fn update_question(&self, question: Question, question_id: i32) -> Result<Question, sqlx::Error> {
        let Question {
            title, content, tags, ..
        } = question;
        match sqlx::query(
            "UPDATE questions \
            SET title = $1,\
                content = $2,\
                tags = $3\
            WHERE id = $4\
            RETURNING *",
        )
        .bind(title)
        .bind(content)
        .bind(tags)
        .bind(question_id)
        .map(Question::try_from)
        .fetch_one(&self.connection)
        .await?
        {
            Ok(question) => Ok(question),
            Err(e) => {
                tracing::event!(target:"webdev_book", tracing::Level::ERROR, "{:?}", e);
                Err(e)
            }
        }
    }

    /// This function will delete a question from the table `questions` by its ID
    ///
    /// # Arguments
    /// - `question_id`: An integer that represents the ID of the question.
    ///
    /// # Returns
    /// - An Ok(true) if the question was deleted successfully.
    /// - An Ok(false) if the question was not found.
    /// - An error if the question could not be deleted.
    pub async fn delete_question(&self, question_id: i32) -> Result<bool, sqlx::Error> {
        match sqlx::query("DELETE FROM questions WHERE id = $1")
            .bind(question_id)
            .execute(&self.connection)
            .await
        {
            Ok(res) => Ok(res.rows_affected() != 0),
            Err(e) => {
                tracing::event!(target:"webdev_book", tracing::Level::ERROR, "hello: {:?}", e);
                Err(e)
            }
        }
    }

    /// This function adds an answer to the table `answers` for a given question ID.
    ///
    /// # Arguments
    /// - `question_id`: An integer that represents the ID of the question.
    /// - `content`: A string slice that contains the content of the answer.
    ///
    /// # Returns
    /// - An Answer if the answer was added successfully.
    /// - An error if the answer could not be added.
    pub async fn add_answer(&self, question_id: i32, content: String) -> Result<Answer, sqlx::Error> {
        match sqlx::query(
            "INSERT INTO answers (content, question_id)\
            VALUES ($1, $2)\
            RETURNING *",
        )
        .bind(content)
        .bind(question_id)
        .map(Answer::try_from)
        .fetch_one(&self.connection)
        .await?
        {
            Ok(answer) => Ok(answer),
            Err(e) => {
                tracing::event!(target:"webdev_book", tracing::Level::ERROR, "{:?}", e);
                Err(e)
            }
        }
    }
}

impl Display for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
