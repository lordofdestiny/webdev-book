use crate::routes::store_filter;
use crate::store::Store;
use crate::types::{Question, QuestionId};
use std::collections::HashMap;
use warp::{filters::BoxedFilter, Filter};

pub fn get_questions() -> BoxedFilter<(HashMap<String, String>, Store)> {
    warp::path!("questions")
        .and(warp::get())
        .and(warp::query::<HashMap<String, String>>())
        .and(store_filter())
        .boxed()
}

pub fn get_question() -> BoxedFilter<(QuestionId, Store)> {
    warp::path!("questions" / QuestionId)
        .and(warp::get())
        .and(store_filter())
        .boxed()
}

pub fn add_question() -> BoxedFilter<(Question, Store)> {
    warp::path!("questions")
        .and(warp::post())
        .and(warp::body::json())
        .and(store_filter())
        .boxed()
}

pub fn update_question() -> BoxedFilter<(QuestionId, Question, Store)> {
    warp::path!("questions" / QuestionId)
        .and(warp::put())
        .and(warp::body::json())
        .and(store_filter())
        .boxed()
}

pub fn delete_question() -> BoxedFilter<(QuestionId, Store)> {
    warp::path!("questions" / QuestionId)
        .and(warp::delete())
        .and(store_filter())
        .boxed()
}
