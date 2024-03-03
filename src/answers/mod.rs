use warp::{Filter, Reply};
use warp::filters::BoxedFilter;

use crate::store::Store;

mod handlers;
mod routes;

/// This function returns the combined filter for the "answers" resource.
///
/// The filter combines the following filters:
/// - `POST /questions/{id}/answers -> add_answer`p
pub fn filter(store: &Store) -> BoxedFilter<(impl Reply, )> {
    routes::add_answer(store.clone()).boxed()
}
