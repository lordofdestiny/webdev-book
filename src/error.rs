//! This module contains the error handling for the API.

use warp::{
    filters::{body::BodyDeserializeError, cors::CorsForbidden},
    http::StatusCode,
    reject::Reject,
    Rejection, Reply,
};

/// This error is returned when the user tries to create an answer without content.
#[derive(Debug)]
pub struct AnswerContentMissing;
impl std::fmt::Display for AnswerContentMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing answer content")
    }
}
impl Reject for AnswerContentMissing {}

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

/// This macro implements the `return_error` function.
///
/// Macro is used to avoid code duplication associated with calling `reject.find::<ErrorType>()`.
/// The macro takes a list of types and their corresponding status code as pairs separated by a
/// colon.
///
/// The macro then generates a function that takes a rejection and returns a reply with the
/// corresponding status code and the error message.
macro_rules! impl_return_error {
        ($($type:ty : $status_code:expr,)*) => {
            /// This function returns the error message associated with the rejection.
            ///
            /// If the rejection is not associated with any error, it returns a `404 Not Found`.
            /// If the rejection is associated with an error, it returns the error message with the
            /// corresponding status code and the error message.
            pub async fn return_error(rej: Rejection) -> Result<impl Reply, Rejection> {
                $(if let Some(error) = rej.find::<$type>() {
                    Ok(warp::reply::with_status(
                        error.to_string(),
                        $status_code
                    ))
                } else)* {
                    Err(warp::reject::not_found())
                }
            }
        };
    }

impl_return_error!(
    CorsForbidden : StatusCode::FORBIDDEN,
    QuestionNotFound : StatusCode::NOT_FOUND,
    PaginationError : StatusCode::BAD_REQUEST,
    AnswerContentMissing : StatusCode::UNPROCESSABLE_ENTITY,
    BodyDeserializeError : StatusCode::UNPROCESSABLE_ENTITY,
);
