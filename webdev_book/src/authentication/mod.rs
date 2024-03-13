//! Module for authentication
//!
//! This module contains the following submodules:
//! - `handlers`- Contains the handlers for the `Authentication` resource
//! - `routes`- Contains the routes for the `Authentication` resource
use std::future;

use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

use crate::error::ServiceError;
use crate::store::Store;
use crate::types::authentication::Session;

/// Handlers for the `Authentication` resource.
mod handlers;
/// Routes for the `Authentication` resource.
mod routes;

/// Filter for the `Authentication` resource.
///
/// Creates a filter that handles requests for the `Authentication` resource.
///
/// The filter combines the following filters:
/// - `register`, for handling `POST /register`
///
/// # Parameters
/// - `store` - The [Store] to use for handling requests.
pub fn filter(store: &Store) -> BoxedFilter<(impl Reply,)> {
    routes::register(store.clone()).or(routes::login(store.clone())).boxed()
}

/// Verifies a password using Argon2.
///
/// Verifies a password using Argon2 and returns a [`Session`]
///
/// If the password is correct, otherwise,
/// it returns a [`ServiceError::CannotDecrpytToken`](ServiceError::CannotDecryptToken).
pub fn verify_token(token: String) -> Result<Session, ServiceError> {
    println!("token: {}", token);
    let token = paseto::tokens::validate_local_token(
        &token,
        None,
        &"RANDOM WORDS WINTER MACINTOSH PC".as_bytes(),
        &paseto::tokens::TimeBackend::Chrono,
    )
    .map_err(|_| ServiceError::CannotDecryptToken)?;
    println!("token: {}", token);

    serde_json::from_value::<Session>(token).map_err(|_| ServiceError::CannotDecryptToken)
}

/// Filter for authenticating requests.
///
/// Creates a filter that authenticates requests using the `Authorization` header.
///
/// The filter extracts a `Session` if the request is authenticated,
/// otherwise it rejects the request.
pub fn auth() -> impl Filter<Extract = (Session,), Error = warp::Rejection> + Clone {
    warp::header("Authorization").and_then(|token| {
        future::ready(match verify_token(token) {
            Ok(session) => Ok(session),
            Err(error) => Err(warp::reject::custom(error)),
        })
    })
}
