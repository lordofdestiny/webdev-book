use crate::{
    error,
    store::Store,
    types::{NextId, Pagination, Question, QuestionId},
};
use std::collections::HashMap;
use warp::{http::StatusCode, Rejection, Reply};

pub async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl Reply, Rejection> {
    let Pagination { start, limit } = Pagination::extract(params)?;
    let res: Vec<Question> = store
        .questions
        .read()
        .await
        .values()
        .cloned()
        .skip(start)
        .take(limit)
        .collect();
    Ok(warp::reply::json(&res))
}

pub async fn get_question(id: QuestionId, store: Store) -> Result<impl Reply, Rejection> {
    match store.questions.read().await.get(&id) {
        Some(q) => Ok(warp::reply::json(q)),
        None => Err(warp::reject::custom(error::QuestionNotFound)),
    }
}

pub async fn add_question(data: Question, store: Store) -> Result<impl Reply, Rejection> {
    let id = QuestionId::next();
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

pub async fn delete_question(id: QuestionId, store: Store) -> Result<impl Reply, Rejection> {
    match store.questions.write().await.remove(&id) {
        Some(_) => Ok(warp::reply::with_status("Question deleted", StatusCode::OK)),
        None => Err(warp::reject::custom(error::QuestionNotFound)),
    }
}
