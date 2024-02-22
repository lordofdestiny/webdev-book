use warp::{filters::BoxedFilter, Filter};

use crate::types::{NewAnswer, QuestionId};

/// POST /questions/{id}/answers
///
/// Handler arguments:
/// 1. `QuestionId` - Extracted from the URL path
/// 2. `NewAnswer` - Extracted from the request body json
pub fn add_answer() -> BoxedFilter<(QuestionId, NewAnswer)> {
    warp::path!("questions" / QuestionId / "answers")
        .and(warp::post())
        .and(warp::body::json())
        .boxed()
}
