use crate::{
    error,
    store::Store,
    types::{NextId, Pagination, Question, QuestionId},
};
use std::collections::HashMap;
use warp::{http::StatusCode, Rejection, Reply};

/// Handler for `GET /questions?start={}&limit={}`
///
/// Query parameters:
/// - start: usize - default `0`
/// - limit: usize - default `usize::MAX`
///
/// Returns a list no more than `limit` questions, starting from `start`.
///
/// Returns `200 OK` on success \
/// Returns `400 Bad Request` if the query parameters are invalid
pub async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl Reply, Rejection> {
    // Extract the pagination parameters from the query
    let Pagination { start, limit } = Pagination::extract(params)?;
    // Read the questions from the store
    let questions = store.questions.read().await;
    // Collect the questions into a vector
    let res: Vec<_> = questions.values().skip(start).take(limit).collect();
    Ok(warp::reply::json(&res))
}

/// Handler for `GET /questions/{id}`
///
/// Returns the question with the given id
///
/// Returns `200 OK` on success \
/// Returns `404 Not Found` if the question does not exist
pub async fn get_question(id: QuestionId, store: Store) -> Result<impl Reply, Rejection> {
    match store.questions.read().await.get(&id) {
        Some(q) => Ok(warp::reply::json(q)),
        None => Err(warp::reject::custom(error::QuestionNotFound)),
    }
}

/// Handler for `POST /questions`
///
/// Creates a new question
///
/// Returns `201 Created` on success \
/// Returns `400 Bad Request` if the question is invalid
pub async fn add_question(data: Question, store: Store) -> Result<impl Reply, Rejection> {
    // Generate a new id for the question
    let id = QuestionId::next();
    // Insert the question into the store
    store.questions.write().await.insert(
        id.clone(),
        Question {
            id: Some(id),
            ..data
        },
    );

    Ok(warp::reply::with_status(
        "Question added",
        StatusCode::CREATED,
    ))
}

/// Handler for `PUT /questions/{id}`
///
/// Updates the question with the given id
///
/// Returns `200 OK` on success \
/// Returns `404 Not Found` if the question does not exist
pub async fn update_question(
    id: QuestionId,
    question: Question,
    store: Store,
) -> Result<impl Reply, Rejection> {
    match store.questions.write().await.get_mut(&id) {
        Some(q) => {
            *q = Question {
                id: Some(id),
                ..question
            };
            Ok(warp::reply::with_status("Question updated", StatusCode::OK))
        }
        None => Err(warp::reject::custom(error::QuestionNotFound)),
    }
}

/// Handler for `DELETE /questions/{id}`
///
/// Deletes the question with the given id
///
/// Returns `200 OK` on success \
/// Returns `404 Not Found` if the question does not exist
pub async fn delete_question(id: QuestionId, store: Store) -> Result<impl Reply, Rejection> {
    match store.questions.write().await.remove(&id) {
        Some(_) => Ok(warp::reply::with_status("Question deleted", StatusCode::OK)),
        None => Err(warp::reject::custom(error::QuestionNotFound)),
    }
}
