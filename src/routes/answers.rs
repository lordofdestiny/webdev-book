use crate::{routes::store_filter, store::Store, types::QuestionId};
use std::collections::HashMap;
use warp::{filters::BoxedFilter, Filter};

pub fn add_answer() -> BoxedFilter<(QuestionId, HashMap<String, String>, Store)> {
    warp::path!("questions" / QuestionId / "answers")
        .and(warp::post())
        .and(warp::body::form())
        .and(store_filter())
        .boxed()
}
