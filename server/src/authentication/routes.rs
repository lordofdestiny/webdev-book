use crate::authentication::handlers;
use crate::filters::store_filter;
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

use crate::store::Store;

/// POST /register
///
/// Creates a filter for a route that handles user registration.
///
/// # Parameters
/// - `store` - [Store] object available to the route handler
pub fn register(store: Store) -> BoxedFilter<(impl Reply, )> {
    store_filter(store)
        .and(warp::post())
        .and(warp::path("register"))
        .and(warp::body::json())
        .and_then(handlers::register)
        .boxed()
}
