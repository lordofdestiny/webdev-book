use warp::{filters::BoxedFilter, Filter};

use crate::filters::store_filter;
use crate::store::Store;
use crate::types::{NewAnswer, QuestionId};

/// POST /questions/{id}/answers
///
/// Handler arguments:
/// 1. `QuestionId` - Extracted from the URL path
/// 2. `NewAnswer` - Extracted from the request body json
pub fn add_answer(store: Store) -> BoxedFilter<(Store, QuestionId, NewAnswer)> {
    store_filter(store.clone())
        .and(warp::post())
        .and(warp::path!("questions" / QuestionId / "answers"))
        .and(warp::body::json())
        .boxed()
}
