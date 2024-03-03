//! This module contains the error handling for the API.

use reqwest::Error as ReqwestError;
use reqwest_middleware::Error as ReqwestMiddlewareError;
use tracing::{error, instrument, warn};
use warp::{
    filters::{body::BodyDeserializeError, cors::CorsForbidden},
    http::StatusCode,
    reject::Reject,
    Rejection, Reply,
};

use crate::api;
use crate::types::QuestionId;

#[derive(thiserror::Error, Debug)]
#[error("{0}")]
pub struct MissingQuestion(pub QuestionId);

impl From<QuestionId> for MissingQuestion {
    fn from(id: QuestionId) -> Self {
        MissingQuestion(id)
    }
}

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

#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
    #[error("invalid pagination parameters")]
    PaginationError(#[from] std::num::ParseIntError),

    #[error("question {0} not found")]
    QuestionNotFound(#[from] MissingQuestion),

    #[error("cannot update, invalid data")]
    DatabaseQueryError(#[from] sqlx::Error),

    #[error("external API error:")]
    ReqwestAPIError(#[from] ReqwestError),
    #[error("external API error")]
    MiddlewareReqwestAPIError(#[from] ReqwestMiddlewareError),

    #[error("external client error")]
    ClientError(APILayerError),

    #[error("external server error")]
    ServerError(APILayerError),
}

impl ServiceError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ServiceError::PaginationError(_) => StatusCode::BAD_REQUEST,
            ServiceError::QuestionNotFound(_) => StatusCode::NOT_FOUND,
            ServiceError::DatabaseQueryError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::ReqwestAPIError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::MiddlewareReqwestAPIError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::ClientError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::ServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl Reject for ServiceError {}

#[instrument(target = "webdev_book::errors", skip_all)]
pub async fn return_error(rejection: Rejection) -> Result<impl Reply, Rejection> {
    use warp::reply::with_status;

    if let Some(service_error) = rejection.find::<ServiceError>() {
        error!("{service_error}");
        Ok(with_status(service_error.to_string(), service_error.status_code()))
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
