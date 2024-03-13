//! Module for authentication
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

use crate::store::Store;

mod handlers;
mod routes;

/// Filter for the `Authentication` resource.
///
/// Creates a filter that handles requests for the `Authentication` resource.
///
/// The filter combines the following filters:
/// - `register`, for handling `POST /register`
///
/// # Parameters
/// - `store` - The [Store] to use for handling requests.
pub fn filter(store: &Store) -> BoxedFilter<(impl Reply,)> {
    routes::register(store.clone()).or(routes::login(store.clone())).boxed()
}
