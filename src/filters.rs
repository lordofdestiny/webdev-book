//! This module contains the combined filters for the application resources.

use warp::{filters::BoxedFilter, http::Method, Filter, Reply};

use crate::store::Store;

/// This function returns the CORS filter for the application.
///
/// The CORS filter allows requests from any origin and allows the following methods:
/// - GET
/// - POST
/// - PUT
/// - PATCH
/// - DELETE
///
/// It also allows the `content-type` header.
pub fn cors() -> warp::cors::Builder {
    warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
}

/// This function returns a filter that associates the store with the request.
///
/// The filter takes a store and returns a boxed filter that takes no arguments and returns the
/// store. This is useful for handlers that need access to the store.
pub fn store_filter(store: Store) -> BoxedFilter<(Store,)> {
    warp::any().map(move || store.clone()).boxed()
}

/// This macro creates a warp trace filter with the given text
macro_rules! with_trace {
    ($what: literal) => {
        warp::trace(|info| {
            tracing::info_span!(
                $what,
                method = %info.method(),
                path = %info.path(),
                id = %uuid::Uuid::new_v4(),
            )
        })
    };
}

/// This function returns the combined filter for the questions resource.
///
/// The filter combines the following filters:
/// - `GET /questions -> get_questions`
/// - `GET /questions/{id} -> get_question`
/// - `POST /questions -> add_question`
/// - `PUT /questions/{id} -> update_question`
/// - `DELETE /questions/{id} -> delete_question`
pub fn questions_filter(store: &Store) -> BoxedFilter<(impl Reply,)> {
    use crate::handlers::questions as handlers;
    use crate::routes::questions as routes;

    routes::get_questions()
        .and(store_filter(store.clone()))
        .and_then(handlers::get_all)
        .with(with_trace!("get_questions request"))
        .or(routes::get_question()
            .and(store_filter(store.clone()))
            .and_then(handlers::get_question)
            .with(with_trace!("get_question request")))
        .or(routes::add_question()
            .and(store_filter(store.clone()))
            .and_then(handlers::add_question)
            .with(with_trace!("add_question request")))
        .or(routes::update_question()
            .and(store_filter(store.clone()))
            .and_then(handlers::update_question)
            .with(with_trace!("update_questions request")))
        .or(routes::delete_question()
            .and(store_filter(store.clone()))
            .and_then(handlers::delete_question)
            .with(with_trace!("delete_question request")))
        .boxed()
}

/// This function returns the combined filter for the answers resource.
///
/// The filter combines the following filters:
/// - `POST /questions/{id}/answers -> add_answer`
pub fn answers_filter(store: &Store) -> BoxedFilter<(impl Reply,)> {
    use crate::handlers::answers as handlers;
    use crate::routes::answers as routes;

    routes::add_answer()
        .and(store_filter(store.clone()))
        .and_then(handlers::add_answer)
        .with(with_trace!("add_answer request"))
        .boxed()
}
