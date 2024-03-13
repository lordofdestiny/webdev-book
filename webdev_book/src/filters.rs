//! Module containing filters that are used to process requests.

use warp::{filters::BoxedFilter, http::Method, Filter};

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
        .allow_methods(&[Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE])
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

pub(crate) use with_trace;
