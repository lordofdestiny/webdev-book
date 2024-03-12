use std::collections::HashMap;

use warp::{filters::BoxedFilter, Filter, Reply};

use crate::filters::{store_filter, with_trace};
use crate::questions::*;
use crate::store::Store;
use crate::types::question::QuestionId;

/// GET /questions?start={i32}&limit={i32}
///
/// Creates a filter for a route that handles fetching a list of questions.
///
/// The filter parses the query parameters, and passes them to the handler.
///
/// # Parameters
/// - `store` - [Store] object available to the route handler
pub fn get_questions(store: Store) -> BoxedFilter<(impl Reply, )> {
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
/// Creates a filter for a route that handles fetching a single question.
/// The filter extracts the `QuestionId` from the URL path and passes it to the handler.
///
/// # Parameters
/// - `store` - [Store] object available to the route handler
pub fn get_question(store: Store) -> BoxedFilter<(impl Reply, )> {
    store_filter(store)
        .and(warp::get())
        .and(warp::path!("questions" / QuestionId))
        .and_then(handlers::get_question)
        .with(with_trace!("get_question request"))
        .boxed()
}

/// POST /questions
///
/// Creates a filter for a route that handles creating a new question.
///
/// The filter extracts the `Question` from the request body as JSON and passes it to the handler.
///
/// # Parameters
/// - `store` - [Store] object available to the route handler
pub fn add_question(store: Store) -> BoxedFilter<(impl Reply, )> {
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
/// Creates a filter for a route that handles updating a question.
///
/// The filter extracts the `QuestionId` from the URL path and the `Question` from the request body as JSON and passes them to the handler.
///
/// # Parameters
/// - `store` - [Store] object available to the route handler
pub fn update_question(store: Store) -> BoxedFilter<(impl Reply, )> {
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
/// Creates a filter for a route that handles deleting a question.
///
/// The filter extracts the `QuestionId` from the URL path and passes it to the handler.
///
/// # Parameters
/// - `store` - [Store] object available to the route handler
pub fn delete_question(store: Store) -> BoxedFilter<(impl Reply, )> {
    store_filter(store)
        .and(warp::delete())
        .and(warp::path!("questions" / QuestionId))
        .and_then(handlers::delete_question)
        .with(with_trace!("delete_question request"))
        .boxed()
}
