use tracing::{debug, info, instrument, trace};
use warp::http::StatusCode;
use warp::reply::with_status;
use warp::{Rejection, Reply};

use crate::error::ServiceError;
use crate::store::Store;
use crate::types::answer::Answer;
use crate::types::authentication::Session;
use crate::types::question::QuestionId;

/// Handler for `POST /questions/{id}/answers`
///
/// Adds a new answer to the store for the given question.
///
/// # Parameters
/// - `store` - [Store] instance
/// - `question_id` - [QuestionId] for the question the answer is associated with
/// - `new_answer` - [Answer] object containing answer content
#[instrument(target = "webdev_book::answers", skip(store))]
pub async fn add_answer(
    store: Store,
    question_id: QuestionId,
    new_answer: Answer,
    session: Session,
) -> Result<impl Reply, Rejection> {
    let Session { account_id, .. } = session;
    trace!("checking if the account is the owner of the question");
    if !store.is_question_owner(question_id, account_id).await? {
        return Err(ServiceError::Unauthorized.into());
    }

    trace!("adding an answer for the question with question_id = {question_id:?}");
    // Check if the question exists

    trace!("censoring the answer content");
    let content = store.bad_words_api.censor(new_answer.content).await?;
    debug!("censored content: {content}");

    match store.add_answer(session.account_id, question_id, content).await {
        Ok(answer) => {
            info!("created the answer for the question with question_id = {question_id:?}");
            debug!("created the answer: {:?}", answer);
            Ok(with_status("Answer created", StatusCode::CREATED))
        }
        Err(error) => Err(warp::reject::custom(error)),
    }
}
