use warp::{filters::BoxedFilter, Filter, Reply};

use crate::answers::handlers;
use crate::filters::{store_filter, with_trace};
use crate::store::Store;
use crate::types::question::QuestionId;

/// POST /questions/{id}/answers
///
/// Creates a filter for a route that handles addition of new answers to a question.
/// The filter expects a JSON payload containing the answer content.
///
/// # Parameters
/// - `store` - [Store] object available to the route handler
pub fn add_answer(store: Store) -> BoxedFilter<(impl Reply,)> {
    store_filter(store)
        .and(warp::post())
        .and(warp::path!("questions" / QuestionId / "answers"))
        .and(warp::body::json())
        .and_then(handlers::add_answer)
        .with(with_trace!("add_answer request"))
        .boxed()
}
