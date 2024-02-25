//! This module contains the error handling for the API.

use tracing::{instrument, warn};
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
    #[error("invalid pagination parameters: {0}")]
    PaginationError(#[from] std::num::ParseIntError),

    #[error("question {0} not found")]
    QuestionNotFound(#[from] MissingQuestion),

    #[error("database query error: {0}")]
    DatabaseQueryError(#[from] sqlx::Error),

    #[error("external API error: {0}")]
    ExternalAPIError(#[from] reqwest::Error),

    #[error("external client error: {0}")]
    ClientError(APILayerError),

    #[error("external server error: {0}")]
    ServerError(APILayerError),
}

impl ServiceError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ServiceError::PaginationError(_) => StatusCode::BAD_REQUEST,
            ServiceError::QuestionNotFound(_) => StatusCode::NOT_FOUND,
            ServiceError::DatabaseQueryError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::ExternalAPIError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::ClientError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::ServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl Reject for ServiceError {}

macro_rules! tracing_event {
    ($error:expr) => {
        tracing::event!(target: "webdev_book", tracing::Level::ERROR, "{}", $error);
    };
}

#[instrument]
pub async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(service_error) = r.find::<ServiceError>() {
        tracing_event!(service_error);
        Ok(warp::reply::with_status(
            service_error.to_string(),
            service_error.status_code(),
        ))
    } else if let Some(error) = r.find::<CorsForbidden>() {
        tracing_event!(error);
        Ok(warp::reply::with_status(error.to_string(), StatusCode::FORBIDDEN))
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        tracing_event!(error);
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else {
        warn!(target: "webdev_book", "Request route not found: {:?}", r);
        Ok(warp::reply::with_status(
            "route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}
