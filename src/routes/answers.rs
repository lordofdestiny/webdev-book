use crate::types::QuestionId;
use std::collections::HashMap;
use warp::{filters::BoxedFilter, Filter};

/// POST /questions/{id}/answers
///
/// Handler arguments:
/// 1. `QuestionId` - Extracted from the URL path
/// 2. `HashMap<String, String>` - Extracted from the request body x-www-form-urlencoded
pub fn add_answer() -> BoxedFilter<(QuestionId, HashMap<String, String>)> {
    warp::path!("questions" / QuestionId / "answers")
        .and(warp::post())
        .and(warp::body::form())
        .boxed()
}
