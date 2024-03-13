use std::collections::HashMap;

use tracing::{debug, info, instrument, trace};
use warp::http::StatusCode;
use warp::reply::{json, with_status};
use warp::{Rejection, Reply};

use crate::types::authentication::Session;
use crate::{
    error::ServiceError,
    store::Store,
    types::{pagination::Pagination, question::*},
};

/// Handler for `GET /questions?start={i32}&limit={i32}`
///
/// Returns a list of questions, paginated according to the query parameters
///
/// Query parameters are consumed from the request and used to paginate the results.
/// If no query parameters are provided, the default values are used.
///
/// The default values are:
/// - `start` - 0
/// - `limit` - no limit
///
/// Pagination logic is implemented in the [Pagination] struct.
///
/// # Parameters
/// - `store` - [Store] instance
/// - `params` - HashMap of query parameters
///   - `start` - The starting index for the paginated results
///   - `limit` - The maximum number of results to return
#[instrument(target = "webdev_book::questions", skip(store))]
pub async fn get_questions(store: Store, params: HashMap<String, String>) -> Result<impl Reply, Rejection> {
    trace!("querying questions");

    // Extract the pagination parameters from the query
    let pag = Pagination::extract(&params)?;

    debug!(pagination = ?pag);

    // Read the questions from the store
    match store.get_questions(pag).await {
        Ok(questions) => {
            debug!(questions_found = questions.len());
            info!("returning all questions");
            Ok(json(&questions))
        }
        Err(e) => Err(e.into()),
    }
}

/// Handler for `GET /questions/{id}`
///
/// Returns the question with the given id.
///
/// # Parameters
/// - `store` - [Store] instance
/// - `id` - [QuestionId] for the question to retrieve
#[instrument(target = "webdev_book::questions", skip(store))]
pub async fn get_question(store: Store, question_id: QuestionId) -> Result<impl Reply, Rejection> {
    trace!("querying question_id = {question_id:?}");

    let question = store.get_question(question_id).await?;
    debug!(question_found = question.is_some());

    match question {
        Some(question) => {
            info!("returning question with question_id = {question_id:?}");
            Ok(json(&question))
        }
        None => Err(ServiceError::QuestionNotFound(question_id.into()).into()),
    }
}

/// Handler for `POST /questions`
///
/// Creates a new question
///
/// # Parameters
/// - `store` - [Store] instance
/// - `question` - [Question] object containing question details
#[instrument(target = "webdev_book::questions", skip(store))]
pub async fn add_question(store: Store, question: Question, session: Session) -> Result<impl Reply, Rejection> {
    trace!("adding a new question");
    let Question {
        title, content, tags, ..
    } = question;

    trace!("censoring title and content...");
    let (title, content) = tokio::try_join!(store.bad_words_api.censor(title), store.bad_words_api.censor(content))?;

    debug!("censored title: {title}");
    debug!("censored content: {content}");

    match store
        .add_question(
            session.account_id,
            Question {
                id: None,
                title,
                content,
                tags,
            },
        )
        .await
    {
        Ok(question) => {
            info!("created a question with question_id = {:?}", question.id);
            Ok(with_status(json(&question), StatusCode::CREATED))
        }
        Err(error) => Err(error.into()),
    }
}

/// Handler for `PUT /questions/{id}`
///
/// Updates the question with the given id
///
/// # Parameters
/// - `store` - [Store] instance
/// - `question_id` - [QuestionId] for the question to update
/// - `question` - [Question] object containing updated question details
#[instrument(target = "webdev_book::questions", skip(store))]
pub async fn update_question(
    store: Store,
    question_id: QuestionId,
    question: Question,
    session: Session,
) -> Result<impl Reply, Rejection> {
    let Session { account_id, .. } = session;
    trace!("checking if the account is the owner of the question");
    if !store.is_question_owner(question_id, account_id).await? {
        return Err(ServiceError::Unauthorized.into());
    }

    trace!("updating the question with question_id = {}", question_id.0);
    let Question {
        title, content, tags, ..
    } = question;

    trace!("censoring title and content...");
    let (title, content) = tokio::try_join!(store.bad_words_api.censor(title), store.bad_words_api.censor(content))?;

    debug!("censored title: {title}");
    debug!("censored content: {content}");

    let censored_question = Question {
        id: Some(question_id),
        title,
        content,
        tags,
    };

    match store
        .update_question(session.account_id, censored_question, question_id)
        .await
    {
        Ok(question) => {
            info!("updated question with question_id = {}", question_id.0);
            debug!(updated_question = ?question);
            Ok(with_status("Question updated", StatusCode::OK))
        }
        Err(error) => Err(error.into()),
    }
}

/// Handler for `DELETE /questions/{id}`
///
/// Deletes the question with the given id
///
/// # Parameters
/// - `store` - [Store] instance
/// - `question_id` - [QuestionId] for the question to delete
#[instrument(target = "webdev_book::questions", skip(store))]
pub async fn delete_question(store: Store, question_id: QuestionId, session: Session) -> Result<impl Reply, Rejection> {
    let Session { account_id, .. } = session;
    trace!("checking if the account is the owner of the question");
    if !store.is_question_owner(question_id, account_id).await? {
        return Err(ServiceError::Unauthorized.into());
    }

    trace!("deleting the question with question_id = {}", question_id.0);
    match store.delete_question(session.account_id, question_id).await {
        Ok(true) => {
            info!("deleted question with question_id = {}", question_id.0);
            Ok(with_status("Question deleted", StatusCode::OK))
        }
        Ok(false) => Err(ServiceError::QuestionNotFound(question_id.into()).into()),
        Err(error) => Err(error.into()),
    }
}
