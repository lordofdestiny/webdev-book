use std::collections::HashMap;
use warp::{http::StatusCode, Rejection, Reply};

use crate::{
    error,
    store::Store,
    types::{Answer, AnswerId, NextId, QuestionId},
};

pub async fn add_answer(
    question_id: QuestionId,
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl Reply, Rejection> {
    if !store.questions.read().await.contains_key(&question_id) {
        return Err(warp::reject::custom(error::QuestionNotFound));
    }

    let answer = Answer {
        id: AnswerId::next(),
        content: params
            .get("content")
            .ok_or(error::AnswerContentMissing)?
            .to_string(),
        question_id,
    };

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
