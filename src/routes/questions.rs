use crate::types::{Question, QuestionId};
use std::collections::HashMap;
use warp::{filters::BoxedFilter, Filter};

/// GET /questions?start={}&limit={}
///
/// Handler arguments:
/// 1. `HashMap<String, String>` - Extracted from the query string
pub fn get_questions() -> BoxedFilter<(HashMap<String, String>,)> {
    warp::path!("questions")
        .and(warp::get())
        .and(warp::query::<HashMap<String, String>>())
        .boxed()
}

/// GET /questions/{id}
///
/// Handler arguments:
/// 1. `QuestionId` - Extracted from the URL path
pub fn get_question() -> BoxedFilter<(QuestionId,)> {
    warp::path!("questions" / QuestionId)
        .and(warp::get())
        .boxed()
}

/// POST /questions
///
/// Handler arguments:
/// 1. `Question` - Extracted from the request body as JSON
pub fn add_question() -> BoxedFilter<(Question,)> {
    warp::path!("questions")
        .and(warp::post())
        .and(warp::body::json())
        .boxed()
}

/// PUT /questions/{id}
///
/// Handler arguments:
/// 1. `QuestionId` - Extracted from the URL path
/// 2. `Question` - Extracted from the request body as JSON
pub fn update_question() -> BoxedFilter<(QuestionId, Question)> {
    warp::path!("questions" / QuestionId)
        .and(warp::put())
        .and(warp::body::json())
        .boxed()
}

/// DELETE /questions/{id}
///
/// Handler arguments:
/// 1. `QuestionId` - Extracted from the URL path
pub fn delete_question() -> BoxedFilter<(QuestionId,)> {
    warp::path!("questions" / QuestionId)
        .and(warp::delete())
        .boxed()
}
