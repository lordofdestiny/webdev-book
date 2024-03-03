use tracing::{debug, info, instrument, trace};
use warp::{Rejection, Reply};
use warp::http::StatusCode;
use warp::reply::with_status;

use crate::{error::ServiceError, store::Store, types::QuestionId};
use crate::types::NewAnswer;

/// Handler for `POST /questions/{id}/answers`
///
/// Adds a new answer to the store for the given question.
///
/// Returns `201 Created` on success. \
/// Returns `404 Not Found` if the question does not exist.
#[instrument(target = "webdev_book::answers", skip(store))]
pub async fn add_answer(store: Store, question_id: QuestionId, new_answer: NewAnswer) -> Result<impl Reply, Rejection> {
    trace!("adding an answer for the question with question_id = {question_id}");
    // Check if the question exists

    trace!("censoring the answer content");
    let content = store.bad_words_api.censor(new_answer.content).await?;
    debug!("censored content: {content}");

    match store.add_answer(question_id.0, content).await {
        Ok(answer) => {
            info!("created the answer for the question with question_id = {question_id}");
            debug!("created the answer: {:?}", answer);
            Ok(with_status("Answer created", StatusCode::CREATED))
        }
        Err(e) => Err(warp::reject::custom(ServiceError::DatabaseQueryError(e))),
    }
}
