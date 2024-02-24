use warp::{filters::BoxedFilter, Filter, Reply};

use crate::answers::handlers;
use crate::filters::{store_filter, with_trace};
use crate::store::Store;
use crate::types::QuestionId;

/// POST /questions/{id}/answers
///
/// Handler arguments:
/// 1. `QuestionId` - Extracted from the URL path
/// 2. `NewAnswer` - Extracted from the request body json
pub fn add_answer(store: Store) -> BoxedFilter<(impl Reply,)> {
    store_filter(store.clone())
        .and(warp::post())
        .and(warp::path!("questions" / QuestionId / "answers"))
        .and(warp::body::json())
        .and_then(handlers::add_answer)
        .with(with_trace!("add_answer request"))
        .boxed()
}
