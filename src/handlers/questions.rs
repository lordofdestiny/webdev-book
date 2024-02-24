use std::collections::HashMap;

use tracing::{error, event, info, instrument, Level};
use warp::{http::StatusCode, Rejection, Reply};

use crate::types::NewQuestion;
use crate::{
    error,
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
pub async fn get_all(store: Store, params: HashMap<String, String>) -> Result<impl Reply, Rejection> {
    event!(target: "webdev_book", Level::INFO, "querying questions");

    // Extract the pagination parameters from the query
    let pag = Pagination::extract(&params)?;

    info!(target: "webdev_book", pagination = !pag.is_default());

    // Read the questions from the store
    match store.get_questions(pag).await {
        Ok(questions) => Ok(warp::reply::json(&questions)),
        Err(e) => {
            error!(target: "webdev_book", "Failed to get questions: {:?}", e);
            Err(warp::reject::custom(error::DatabaseQueryError(e)))
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
        Err(_) => Err(warp::reject::custom(error::QuestionNotFound)),
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

    match store.add_question(new_question).await {
        Ok(question) => {
            info!(target: "webdev_book", "Created a question with question_id = {}", question.id);
            Ok(warp::reply::with_status("Question created", StatusCode::CREATED))
        }
        Err(e) => {
            error!(target: "webdev_book", "Failed to create a question: {:?}", e);
            Err(warp::reject::custom(error::DatabaseQueryError(e)))
        }
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

    match store.update_question(question, id.0).await {
        Ok(_) => {
            info!(target: "webdev_book", "updated question with question_id = {id}");
            Ok(warp::reply::with_status("Question updated", StatusCode::OK))
        }
        Err(e) => {
            error!(target: "webdev_book", "Failed to update a question: {:?}", e);
            Err(warp::reject::custom(error::DatabaseQueryError(e)))
        }
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
        Ok(false) => {
            error!(target: "webdev_book", "Failed to delete a question: question_id = {id}, because it does not exist");
            Err(warp::reject::custom(error::QuestionNotFound))
        }
        Err(e) => {
            error!(target: "webdev_book", "Failed to update a question: {:?}", e);
            Err(warp::reject::custom(error::DatabaseQueryError(e)))
        }
    }
}
