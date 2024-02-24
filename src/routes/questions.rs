use std::collections::HashMap;

use warp::{filters::BoxedFilter, Filter};

use crate::filters::store_filter;
use crate::store::Store;
use crate::types::{NewQuestion, Question, QuestionId};

/// GET /questions?start={}&limit={}
///
/// Handler arguments:
/// 1. `HashMap<String, String>` - Extracted from the query string
pub fn get_questions(store: Store) -> BoxedFilter<(Store, HashMap<String, String>)> {
    store_filter(store)
        .and(warp::get())
        .and(warp::path!("questions"))
        .and(warp::query::<HashMap<String, String>>())
        .boxed()
}

/// GET /questions/{id}
///
/// Handler arguments:
/// 1. `QuestionId` - Extracted from the URL path
pub fn get_question(store: Store) -> BoxedFilter<(Store, QuestionId)> {
    store_filter(store)
        .and(warp::get())
        .and(warp::path!("questions"))
        .and(warp::query::<QuestionId>())
        .boxed()
}

/// POST /questions
///
/// Handler arguments:
/// 1. `Question` - Extracted from the request body as JSON
pub fn add_question(store: Store) -> BoxedFilter<(Store, NewQuestion)> {
    store_filter(store)
        .and(warp::post())
        .and(warp::path!("questions"))
        .and(warp::body::json())
        .boxed()
}

/// PUT /questions/{id}
///
/// Handler arguments:
/// 1. `QuestionId` - Extracted from the URL path
/// 2. `Question` - Extracted from the request body as JSON
pub fn update_question(store: Store) -> BoxedFilter<(Store, QuestionId, Question)> {
    store_filter(store)
        .and(warp::put())
        .and(warp::path!("questions" / QuestionId))
        .and(warp::body::json())
        .boxed()
}

/// DELETE /questions/{id}
///
/// Handler arguments:
/// 1. `QuestionId` - Extracted from the URL path
pub fn delete_question(store: Store) -> BoxedFilter<(Store, QuestionId)> {
    store_filter(store)
        .and(warp::delete())
        .and(warp::path!("questions" / QuestionId))
        .boxed()
}
