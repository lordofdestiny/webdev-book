//! Module that implements the [Store], a shared state for the application.

use std::sync::Arc;

use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing::{error, instrument, trace};

use crate::api::bad_words::BadWordsAPI;
use crate::error::ServiceError;
use crate::types::account::Account;
use crate::types::{answer::Answer, pagination::Pagination, question::Question};

/// This struct represents the store, which is a simple in-memory database.
///
/// The store contains two maps: one for questions and one for answers.
/// The maps are wrapped in an `Arc` and a `RwLock` to allow for concurrent access.
#[derive(Clone)]
pub struct Store {
    pub connection: PgPool,
    pub bad_words_api: Arc<BadWordsAPI>,
}

impl std::fmt::Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Store").field("connection", &self.connection).finish()
    }
}

impl std::fmt::Display for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
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
    #[instrument(target = "store", level = "debug", skip(db_url))]
    pub async fn new(db_url: &str) -> Self {
        trace!("creating store object");

        trace!("creating connection pool to ${db_url}");
        let db_pool = match PgPoolOptions::new().max_connections(5).connect(db_url).await {
            Ok(pool) => pool,
            Err(e) => panic!("Couldn't establish DB connection: {}", e),
        };

        trace!("building BadWordsAPI object");
        const API_LAYER_KEY: &str = env!("API_LAYER_KEY", "API_LAYER_KEY not found");
        let bad_words_api = BadWordsAPI::build(API_LAYER_KEY, '*').expect("Couldn't build BadWordsAPI");

        trace!("store object created successfully");
        Store {
            connection: db_pool,
            bad_words_api: Arc::new(bad_words_api),
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
    #[instrument(target = "store", level = "debug", skip(self))]
    pub async fn get_questions(&self, pag: Pagination) -> Result<Vec<Question>, ServiceError> {
        let Pagination { offset, limit } = pag;

        trace!("fetching questions from the database");
        match sqlx::query("SELECT * FROM questions LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .map(Question::try_from)
            .fetch_all(&self.connection)
            .await?
            .into_iter()
            .collect()
        {
            Ok(questions) => {
                trace!("questions fetched successfully");
                Ok(questions)
            }
            Err(error) => {
                error!("{error}");
                Err(ServiceError::DatabaseQueryError(error))
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
    pub async fn get_question(&self, id: i32) -> Result<Option<Question>, ServiceError> {
        let pg_row = sqlx::query("SELECT * FROM questions WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.connection)
            .await?;

        let Some(pg_row) = pg_row else {
            trace!("question not found");
            return Ok(None);
        };

        match Question::try_from(pg_row) {
            Ok(question) => Ok(Some(question)),
            Err(error) => {
                tracing::event!(target:"webdev_book", tracing::Level::ERROR, "{:?}", error);
                Err(ServiceError::DatabaseQueryError(error))
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
    #[instrument(target = "store", skip(self))]
    pub async fn add_question(
        &self,
        Question {
            title, content, tags, ..
        }: Question,
    ) -> Result<Question, ServiceError> {
        trace!("adding a question to the database");

        let res = sqlx::query(
            "INSERT INTO questions (title, content, tags)\
            VALUES ($1, $2, $3)\
            RETURNING *",
        )
        .bind(title)
        .bind(content)
        .bind(tags)
        .map(Question::try_from)
        .fetch_one(&self.connection)
        .await?;

        match res {
            Ok(question) => {
                trace!("question added successfully with id={:?}", question.id);
                Ok(question)
            }
            Err(error) => {
                error!("{error}");
                Err(ServiceError::DatabaseQueryError(error))
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
    #[instrument(target = "store", skip(self))]
    pub async fn update_question(&self, question: Question, question_id: i32) -> Result<Question, ServiceError> {
        trace!("updating question in the database; id={question_id}");
        let Question {
            title, content, tags, ..
        } = question;

        match sqlx::query(
            "UPDATE questions \
            SET title = $1,\
                content = $2,\
                tags = $3 \
            WHERE id = $4 \
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
            Ok(question) => {
                trace!("question updated successfully");
                Ok(question)
            }
            Err(error) => {
                error!("{error}");
                Err(ServiceError::DatabaseQueryError(error))
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
    #[instrument(target = "store", skip(self))]
    pub async fn delete_question(&self, question_id: i32) -> Result<bool, ServiceError> {
        trace!("deleting question from the database; id={question_id}");
        match sqlx::query("DELETE FROM questions WHERE id = $1")
            .bind(question_id)
            .execute(&self.connection)
            .await
        {
            Ok(res) => {
                if res.rows_affected() == 0 {
                    trace!("question not found");
                    Ok(false)
                } else {
                    trace!("question deleted successfully");
                    Ok(true)
                }
            }
            Err(error) => {
                error!("{error}");
                Err(ServiceError::DatabaseQueryError(error))
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
    #[instrument(target = "store", skip(self))]
    pub async fn add_answer(&self, question_id: i32, content: String) -> Result<Answer, ServiceError> {
        trace!("adding an answer for the question with id={question_id}");
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
            Ok(answer) => {
                trace!("answer added successfully with id={:?}", answer.id);
                Ok(answer)
            }
            Err(error) => {
                error!("{error}");
                Err(ServiceError::DatabaseQueryError(error))
            }
        }
    }

    /// This function creates a new account in the table `accounts`.
    ///
    /// It is expected that the password is already hashed before calling this function.
    ///
    /// # Arguments
    /// - `account`: An `Account` struct that contains the email and password of the account.
    #[instrument(target = "store", skip(self))]
    pub async fn add_account(self, account: Account) -> Result<bool, ServiceError> {
        match sqlx::query("INSERT INTO accounts (email, password) VALUES ($1, $2)")
            .bind(account.email)
            .bind(account.password)
            .execute(&self.connection)
            .await
        {
            Ok(_) => Ok(true),
            Err(error) => {
                let db_error = error.as_database_error().unwrap();
                error!(
                    code = db_error.code().unwrap().parse::<i32>().unwrap(),
                    db_message = db_error.message(),
                    constraint = db_error.constraint().unwrap(),
                );
                Err(ServiceError::DatabaseQueryError(error))
            }
        }
    }

    /// Get an account from the table `accounts` by its email.
    ///
    /// # Arguments
    /// - `email`: A string slice that contains the email of the account.
    ///
    /// # Returns
    /// - An `Account` if the account was found successfully.
    #[instrument(target = "store", skip(self))]
    pub async fn get_account(&self, email: &str) -> Result<Account, ServiceError> {
        let pg_row = sqlx::query("SELECT * FROM accounts WHERE email = $1")
            .bind(email)
            .fetch_one(&self.connection)
            .await?;

        match Account::try_from(pg_row) {
            Ok(account) => Ok(account),
            Err(error) => {
                tracing::event!(target:"webdev_book", tracing::Level::ERROR, "{:?}", error);
                Err(ServiceError::DatabaseQueryError(error))
            }
        }
    }
}
