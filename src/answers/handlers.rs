use tracing::{event, info, instrument, Level};
use warp::{Rejection, Reply};

use crate::types::NewAnswer;
use crate::{error::ServiceError, store::Store, types::QuestionId};

/// Handler for `POST /questions/{id}/answers`
///
/// Adds a new answer to the store for the given question.
///
/// Returns `201 Created` on success. \
/// Returns `404 Not Found` if the question does not exist.
#[instrument]
pub async fn add_answer(store: Store, question_id: QuestionId, new_answer: NewAnswer) -> Result<impl Reply, Rejection> {
    event!(target: "webdev_book", Level::INFO, "adding an answer for the question with question_id = {question_id}");
    // Check if the question exists

    match store.add_answer(question_id.0, new_answer.content).await {
        Ok(_) => {
            info!(target: "webdev_book", "created the answer for the question with question_id = {question_id}");
            Ok(warp::reply::with_status(
                "Answer created",
                warp::http::StatusCode::CREATED,
            ))
        }
        Err(e) => {
            info!(target: "webdev_book", "error adding an answer: {:?}", e);
            Err(warp::reject::custom(ServiceError::DatabaseQueryError(e)))
        }
    }
}
