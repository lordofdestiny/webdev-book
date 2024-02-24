//! This module contains the error handling for the API.

use sqlx::Error as SqlxError;
use warp::{
    filters::{body::BodyDeserializeError, cors::CorsForbidden},
    http::StatusCode,
    reject::Reject,
    Rejection, Reply,
};

/// This error is returned when the sends a malformed pagination parameter.
#[derive(thiserror::Error, Debug)]
#[error("invalid pagination parameters: {0}")]
pub struct PaginationError(#[from] pub std::num::ParseIntError);

impl Reject for PaginationError {}

/// This error is returned when the user tries to access a question that does not exist.
#[derive(thiserror::Error, Debug)]
#[error("question not found")]
pub struct QuestionNotFound;

impl Reject for QuestionNotFound {}

/// This error is returned any time the database query fails.
#[derive(thiserror::Error, Debug)]
#[error("database query failed: {0}")]
pub struct DatabaseQueryError(pub SqlxError);

impl Reject for DatabaseQueryError {}

macro_rules! tracing_event {
    ($error:ident, $type:ty) => {
        tracing::event!(target: "webdev_book", tracing::Level::ERROR, "{}", $error);
    };
    ($error:ident, $type:ty, $message_fmt:literal)=> {
        tracing::event!(target: "webdev_book", tracing::Level::ERROR, $message_fmt, $error);
    }
}

/// This macro implements the `return_error` function.
///
/// Macro is used to avoid code duplication associated with calling `reject.find::<ErrorType>()`.
/// The macro takes a list of types and their corresponding status code as pairs separated by a
/// colon.
///
/// The macro then generates a function that takes a rejection and returns a reply with the
/// corresponding status code and the error message.
macro_rules! impl_return_error {
        ($(($type:ty, $status_code:expr  $(, $message_fmt:literal)?),)*) => {
            /// This function returns the error message associated with the rejection.
            ///
            /// If the rejection is not associated with any error, it returns a `404 Not Found`.
            /// If the rejection is associated with an error, it returns the error message with the
            /// corresponding status code and the error message.
            pub async fn return_error(rej: Rejection) -> Result<impl Reply, Rejection> {
                $(if let Some(error) = rej.find::<$type>() {
                    tracing_event!(error, $type $(, $message_fmt)?);
                    Ok(warp::reply::with_status(
                        error.to_string(),
                        $status_code,
                    ))
                } else)* {
                    Err(warp::reject::not_found())
                }
            }
        };
    }

impl_return_error!(
    (
        CorsForbidden,
        StatusCode::FORBIDDEN,
        "CORS policy rejected the request: {}"
    ),
    (
        BodyDeserializeError,
        StatusCode::UNPROCESSABLE_ENTITY,
        "invalid request body (expected JSON): {}"
    ),
    (QuestionNotFound, StatusCode::NOT_FOUND),
    (PaginationError, StatusCode::BAD_REQUEST),
    (DatabaseQueryError, StatusCode::INTERNAL_SERVER_ERROR),
);
