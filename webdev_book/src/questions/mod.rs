//! Module for handling requests for the `Questions` resource.
//!
//! This module contains the following submodules:
//! - `handlers` - Contains the request handlers for the `Questions` resource.
//! - `routes` - Contains the filters for the `Questions` resource.
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

use crate::store::Store;

/// Handlers for the `Questions` resource.
mod handlers;
/// Routes for the `Questions` resource.
mod routes;

/// Filter for `Questions` module
///
/// Creates a filter that handles requests for the `Questions` resource.
///
/// The filter combines the following filters:
/// - `get_questions` for handling `GET /questions`
/// - `get_question` for handling `GET /questions/{id}`
/// - `add_question` for handling `POST /questions`
/// - `update_question` for handling `PUT /questions/{id}`
/// - `delete_question` for handling `DELETE /questions/{id}`
///
/// # Parameters
/// - `store` - The [Store] to use for handling requests.
pub fn filter(store: &Store) -> BoxedFilter<(impl Reply,)> {
    routes::get_questions(store.clone())
        .or(routes::get_question(store.clone()))
        .or(routes::add_question(store.clone()))
        .or(routes::update_question(store.clone()))
        .or(routes::delete_question(store.clone()))
        .boxed()
}
