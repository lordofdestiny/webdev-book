//! Module for the `Answer` resource.
//!
//! This module contains the following submodules:
//! - `handlers` - Contains the request handlers for the `Answer` resource.
//! - `routes` - Contains the filters for the `Answer` resource.
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

use crate::store::Store;

mod handlers;
mod routes;

/// Filter for the `Answer` resource.
///
/// Creates a filter that handles requests for the `Answer` resource.
///
/// The filter combines the following filters:
/// - `add_answer`, for handling `POST /questions/{id}/answers`
///
/// # Parameters
/// - `store` - The [Store] to use for handling requests.
pub fn filter(store: &Store) -> BoxedFilter<(impl Reply, )> {
    routes::add_answer(store.clone()).boxed()
}
