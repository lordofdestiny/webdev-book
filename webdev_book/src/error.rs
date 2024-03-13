//! Module that implements the error handling for the API.
pub use argon2::Error as ArgonError;
pub use reqwest::Error as ReqwestError;
pub use reqwest_middleware::Error as ReqwestMiddlewareError;
pub use sqlx::Error as SqlxError;
use tracing::{error, instrument, warn};
use warp::{
    filters::{body::BodyDeserializeError, cors::CorsForbidden},
    http::StatusCode,
    reject::{MissingHeader, Reject},
    Rejection, Reply,
};

use crate::api;
use crate::types::question::QuestionId;

/// Error type for missing questions
///
/// This error is used when a question is not found in the database.
#[derive(thiserror::Error, Debug)]
#[error("{0:?}")]
pub struct MissingQuestion(pub QuestionId);

impl From<QuestionId> for MissingQuestion {
    fn from(id: QuestionId) -> Self {
        MissingQuestion(id)
    }
}

/// Error type for the API layer
///
/// This error is used when the API layer returns an error.
/// API layer errors are errors that are returned from the external API.
#[derive(thiserror::Error, Debug, Clone)]
#[error("status: {status}, message: {message}")]
pub struct APILayerError {
    pub status: StatusCode,
    pub message: String,
}

impl Reject for APILayerError {}

impl APILayerError {
    pub async fn transform_error(res: reqwest::Response) -> Self {
        Self {
            status: res.status(),
            message: res.json::<api::APIResponse>().await.unwrap().message,
        }
    }
}

/// Error type for all errors returned by the service
#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
    /// Error for invalid pagination parameters
    #[error("invalid pagination parameters")]
    PaginationError(#[from] std::num::ParseIntError),
    /// Error for missing questions, used when a question is not found in the database
    #[error("question {0} not found")]
    QuestionNotFound(#[from] MissingQuestion),
    /// Error for invalid database queries
    #[error("cannot update, invalid data")]
    DatabaseQueryError(#[from] SqlxError),
    /// Error returned by the Argon2 hashing library
    #[error("argon2 error")]
    ArgonLibraryError(#[from] ArgonError),
    /// Error for Reqwest errors
    #[error("external API error:")]
    ReqwestAPIError(#[from] ReqwestError),
    /// Error for Reqwest middleware errors
    #[error("external API error")]
    MiddlewareReqwestAPIError(#[from] ReqwestMiddlewareError),
    /// Error for client errors
    #[error("external client error")]
    ClientError(APILayerError),
    /// Error for server errors
    #[error("external server error")]
    ServerError(APILayerError),
    #[error("wrong credentials combination")]
    WrongPassword,
    #[error("auth token could not be decyphered")]
    CannotDecryptToken,
    #[error("unauthorized, no premission to modify the resource")]
    Unauthorized,
}

impl ServiceError {
    /// Returns the status code for the error
    ///
    /// This function returns the status code for the error, based on the error type.
    ///
    /// # Returns
    /// - `StatusCode`: The status code for the error
    ///     - `StatusCode::BAD_REQUEST`: For `PaginationError`
    ///     - `StatusCode::NOT_FOUND`: For `QuestionNotFound`
    ///     - `StatusCode::INTERNAL_SERVER_ERROR`: For all other errors
    pub fn status_code(&self) -> StatusCode {
        use ServiceError::*;
        match self {
            PaginationError(_) => StatusCode::BAD_REQUEST,
            QuestionNotFound(_) => StatusCode::NOT_FOUND,
            DatabaseQueryError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ArgonLibraryError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ReqwestAPIError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            MiddlewareReqwestAPIError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ClientError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            WrongPassword => StatusCode::UNAUTHORIZED,
            CannotDecryptToken => StatusCode::UNAUTHORIZED,
            Unauthorized => StatusCode::UNAUTHORIZED,
        }
    }
}

impl Reject for ServiceError {}

/// Error codes for PostgreSQL
pub mod pg_error_codes {
    pub const UNIQUE_VIOLATION: &str = "23505";
    pub const CHECK_VIOLATION: &str = "23514";

    /// Returns the default error message for the error code
    pub fn default_error_message(code: &str) -> &'static str {
        match code {
            UNIQUE_VIOLATION => "duplicate data",
            CHECK_VIOLATION => "invalid data: constraint violation",
            _ => "cannot update data",
        }
    }
}

/// Error handler for the API
///
/// This function handles the errors returned by the API, when handlers return a `Result` with an `Err`
/// variants that implement the `Reject` trait.
///
/// Errors are logged and a response is returned with the appropriate status code.
///
/// # Parameters
/// - `rejection`: The rejection returned by the handler
///
/// # Panics
/// - If error is a [DatabaseQueryError](ServiceError::DatabaseQueryError) and the error code is not recognized
#[instrument(target = "webdev_book::errors", skip_all)]
pub async fn return_error(rejection: Rejection) -> Result<impl Reply, Rejection> {
    use warp::reply::with_status;
    if let Some(ServiceError::DatabaseQueryError(error)) = rejection.find() {
        let message = match error {
            sqlx::Error::Database(err) => {
                let code = err.code().unwrap();
                pg_error_codes::default_error_message(code.as_ref())
            }
            _ => "cannot update data",
        };
        error!("{message}");
        Ok(with_status(message.to_string(), StatusCode::UNPROCESSABLE_ENTITY))
    } else if let Some(service_error) = rejection.find::<ServiceError>() {
        error!("{service_error}");
        Ok(with_status(service_error.to_string(), service_error.status_code()))
    } else if let Some(error) = rejection.find::<MissingHeader>() {
        error!("{error}");
        Ok(with_status(
            format!("missing request header: \"{}\"", error.name()),
            StatusCode::BAD_REQUEST,
        ))
    } else if let Some(error) = rejection.find::<CorsForbidden>() {
        error!("{error}");
        Ok(with_status(error.to_string(), StatusCode::FORBIDDEN))
    } else if let Some(error) = rejection.find::<BodyDeserializeError>() {
        error!("{error}");
        Ok(with_status(error.to_string(), StatusCode::UNPROCESSABLE_ENTITY))
    } else {
        warn!("request route not found: {rejection:?}");
        Ok(with_status("route not found".to_string(), StatusCode::NOT_FOUND))
    }
}
