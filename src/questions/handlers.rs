use std::collections::HashMap;

use tracing::{error, event, info, instrument, Level};
use warp::{http::StatusCode, Rejection, Reply};

use crate::types::NewQuestion;
use crate::{
    error::ServiceError,
    store::Store,
    types::{Pagination, Question, QuestionId},
};

/// Handler for `GET /questions?start={}&limit={}`
///
/// Query parameters:
/// - offset: usize - default `0`
/// - limit: usize - default `usize::MAX`
///
/// Returns a list no more than `limit` questions, starting from `offset`.
///
/// Returns `200 OK` on success \
/// Returns `400 Bad Request` if the query parameters are invalid
#[instrument]
pub async fn get_questions(store: Store, params: HashMap<String, String>) -> Result<impl Reply, Rejection> {
    event!(target: "webdev_book", Level::INFO, "querying questions");

    // Extract the pagination parameters from the query
    let pag = Pagination::extract(&params)?;

    info!(target: "webdev_book", pagination = !pag.is_default());

    // Read the questions from the store
    match store.get_questions(pag).await {
        Ok(questions) => Ok(warp::reply::json(&questions)),
        Err(e) => {
            error!(target: "webdev_book", "Failed to get questions: {:?}", e);
            Err(warp::reject::custom(ServiceError::DatabaseQueryError(e)))
        }
    }
}

/// Handler for `GET /questions/{id}`
///
/// Returns the question with the given id
///
/// Returns `200 OK` on success \
/// Returns `404 Not Found` if the question does not exist
#[instrument]
pub async fn get_question(store: Store, id: QuestionId) -> Result<impl Reply, Rejection> {
    event!(target: "webdev_book", Level::INFO, "querying question_id = {id}");

    let question = store.get_question(id.0).await;
    info!(target: "webdev_book", question_found = question.is_ok());

    match question {
        Ok(q) => Ok(warp::reply::json(&q)),
        Err(_) => Err(warp::reject::custom(ServiceError::QuestionNotFound(id.into()))),
    }
}

/// Handler for `POST /questions`
///
/// Creates a new question
///
/// Returns `201 Created` on success \
/// Returns `400 Bad Request` if the question is invalid
#[instrument]
pub async fn add_question(store: Store, new_question: NewQuestion) -> Result<impl Reply, Rejection> {
    event!(target: "webdev_book", Level::INFO, "adding a new question");

    info!(target: "webdev_book", "censoring content...");
    let bad_words = store
        .bad_words_api
        .lock()
        .await
        .censor(new_question.content.clone())
        .await?;

    let new_question = NewQuestion {
        content: bad_words.censored_content,
        ..new_question
    };

    match store.add_question(new_question).await {
        Ok(question) => {
            info!(target: "webdev_book", "created a question with question_id = {}", question.id);
            Ok(warp::reply::json(&question))
        }
        Err(e) => Err(warp::reject::custom(ServiceError::DatabaseQueryError(e))),
    }
}

/// Handler for `PUT /questions/{id}`
///
/// Updates the question with the given id
///
/// Returns `200 OK` on success \
/// Returns `404 Not Found` if the question does not exist
pub async fn update_question(store: Store, id: QuestionId, question: Question) -> Result<impl Reply, Rejection> {
    event!(target: "webdev_book", Level::INFO, "updating the question with question_id = {id}");

    match store
        .update_question(question, id.0)
        .await
        .map_err(ServiceError::DatabaseQueryError)
    {
        Ok(_) => {
            info!(target: "webdev_book", "updated question with question_id = {id}");
            Ok(warp::reply::with_status("Question updated", StatusCode::OK))
        }
        Err(e) => Err(warp::reject::custom(e)),
    }
}

/// Handler for `DELETE /questions/{id}`
///
/// Deletes the question with the given id
///
/// Returns `200 OK` on success \
/// Returns `404 Not Found` if the question does not exist
pub async fn delete_question(store: Store, id: QuestionId) -> Result<impl Reply, Rejection> {
    event!(target: "webdev_book", Level::INFO, "deleting the question with question_id = {id}");

    match store.delete_question(id.0).await {
        Ok(true) => {
            info!(target: "webdev_book", "deleted question with question_id = {id}");
            Ok(warp::reply::with_status("Question deleted", StatusCode::OK))
        }
        Ok(false) => Err(warp::reject::custom(ServiceError::QuestionNotFound(id.into()))),
        Err(e) => Err(warp::reject::custom(ServiceError::DatabaseQueryError(e))),
    }
}
