use std::collections::HashMap;

use warp::{filters::BoxedFilter, Filter, Reply};

use crate::filters::{store_filter, with_trace};
use crate::questions::*;
use crate::store::Store;
use crate::types::QuestionId;

/// GET /questions?start={}&limit={}
///
/// Handler arguments:
/// 1. `HashMap<String, String>` - Extracted from the query string
pub fn get_questions(store: Store) -> BoxedFilter<(impl Reply,)> {
    store_filter(store)
        .and(warp::get())
        .and(warp::path!("questions"))
        .and(warp::query::<HashMap<String, String>>())
        .and_then(handlers::get_questions)
        .with(with_trace!("get_questions request"))
        .boxed()
}

/// GET /questions/{id}
///
/// Handler arguments:
/// 1. `QuestionId` - Extracted from the URL path
pub fn get_question(store: Store) -> BoxedFilter<(impl Reply,)> {
    store_filter(store)
        .and(warp::get())
        .and(warp::path!("questions"))
        .and(warp::query::<QuestionId>())
        .and_then(handlers::get_question)
        .with(with_trace!("get_question request"))
        .boxed()
}

/// POST /questions
///
/// Handler arguments:
/// 1. `Question` - Extracted from the request body as JSON
pub fn add_question(store: Store) -> BoxedFilter<(impl Reply,)> {
    store_filter(store)
        .and(warp::post())
        .and(warp::path!("questions"))
        .and(warp::body::json())
        .and_then(handlers::add_question)
        .with(with_trace!("add_question request"))
        .boxed()
}

/// PUT /questions/{id}
///
/// Handler arguments:
/// 1. `QuestionId` - Extracted from the URL path
/// 2. `Question` - Extracted from the request body as JSON
pub fn update_question(store: Store) -> BoxedFilter<(impl Reply,)> {
    store_filter(store)
        .and(warp::put())
        .and(warp::path!("questions" / QuestionId))
        .and(warp::body::json())
        .and_then(handlers::update_question)
        .with(with_trace!("update_questions request"))
        .boxed()
}

/// DELETE /questions/{id}
///
/// Handler arguments:
/// 1. `QuestionId` - Extracted from the URL path
pub fn delete_question(store: Store) -> BoxedFilter<(impl Reply,)> {
    store_filter(store)
        .and(warp::delete())
        .and(warp::path!("questions" / QuestionId))
        .and_then(handlers::delete_question)
        .with(with_trace!("delete_question request"))
        .boxed()
}
