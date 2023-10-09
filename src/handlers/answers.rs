use std::collections::HashMap;
use warp::{http::StatusCode, Rejection, Reply};

use crate::{
    error,
    store::Store,
    types::{Answer, QuestionId},
};

/// Handler for `POST /questions/{id}/answers`
///
/// Adds a new answer to the store for the given question.
///
/// Returns `201 Created` on success. \
/// Returns `404 Not Found` if the question does not exist.
pub async fn add_answer(
    question_id: QuestionId,
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl Reply, Rejection> {
    // Check if the question exists
    if !store.questions.read().await.contains_key(&question_id) {
        return Err(warp::reject::custom(error::QuestionNotFound));
    }

    // Extract the content from the form
    let content = params
        .get("content")
        .ok_or(error::AnswerContentMissing)?
        .clone();

    // Create the answer
    let answer = Answer::new(content, question_id);

    // Add the answer to the store
    store
        .answers
        .write()
        .await
        .insert(answer.id.clone(), answer);

    Ok(warp::reply::with_status(
        "Answer added",
        StatusCode::CREATED,
    ))
}
