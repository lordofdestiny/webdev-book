use crate::store::Store;
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

mod handlers;
mod routes;

/// This function returns the combined filter for the questions resource.
///
/// The filter combines the following filters:
/// - `GET /questions
/// - `GET /questions/{id}`
/// - `POST /questions`
/// - `PUT /questions/{id}`
/// - `DELETE /questions/{id}`
pub fn filter(store: &Store) -> BoxedFilter<(impl Reply,)> {
    routes::get_questions(store.clone())
        .or(routes::get_question(store.clone()))
        .or(routes::add_question(store.clone()))
        .or(routes::update_question(store.clone()))
        .or(routes::delete_question(store.clone()))
        .boxed()
}
