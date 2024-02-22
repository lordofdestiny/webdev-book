//! This module contains the error handling for the API.

use sqlx::Error as SqlxError;
use warp::{
    filters::{body::BodyDeserializeError, cors::CorsForbidden},
    http::StatusCode,
    reject::Reject,
    Rejection, Reply,
};

/// This error is returned when the sends a malformed pagination parameter.
#[derive(Debug)]
pub struct PaginationError(pub std::num::ParseIntError);

impl std::fmt::Display for PaginationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Cannot parse parameter: {}", self.0)
    }
}

impl Reject for PaginationError {}

/// This error is returned when the user tries to access a question that does not exist.
#[derive(Debug)]
pub struct QuestionNotFound;

impl std::fmt::Display for QuestionNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Question not found")
    }
}

impl Reject for QuestionNotFound {}

/// This error is returned any time the database query fails.
#[derive(Debug)]
pub struct DatabaseQueryError(pub SqlxError);

impl std::fmt::Display for DatabaseQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Database query error: Query could not be executed: {}", self.0)
    }
}

impl Reject for DatabaseQueryError {}

/// This macro implements the `return_error` function.
///
/// Macro is used to avoid code duplication associated with calling `reject.find::<ErrorType>()`.
/// The macro takes a list of types and their corresponding status code as pairs separated by a
/// colon.
///
/// The macro then generates a function that takes a rejection and returns a reply with the
/// corresponding status code and the error message.
macro_rules! impl_return_error {
        ($($type:ty : ($status_code:expr, $message_fmt:literal),)*) => {
            /// This function returns the error message associated with the rejection.
            ///
            /// If the rejection is not associated with any error, it returns a `404 Not Found`.
            /// If the rejection is associated with an error, it returns the error message with the
            /// corresponding status code and the error message.
            pub async fn return_error(rej: Rejection) -> Result<impl Reply, Rejection> {
                $(if let Some(error) = rej.find::<$type>() {
                    tracing::event!(target: "webdev_book", tracing::Level::ERROR, $message_fmt, error);
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
    CorsForbidden : (StatusCode::FORBIDDEN, "CORS policy rejected the request: {}"),
    QuestionNotFound : (StatusCode::NOT_FOUND, "{}"),
    PaginationError : (StatusCode::BAD_REQUEST, "Invalid pagination parameters: {}"),
    BodyDeserializeError : (StatusCode::UNPROCESSABLE_ENTITY, "Invalid request body (expected JSON): {}"),
    DatabaseQueryError : (StatusCode::INTERNAL_SERVER_ERROR, "{}"),
);
