use std::collections::HashMap;

use tracing::{debug, info, instrument, trace};
use warp::{Rejection, Reply};
use warp::http::StatusCode;
use warp::reply::{json, with_status};

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
        Err(e) => Err(ServiceError::DatabaseQueryError(e).into()),
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
pub async fn get_question(store: Store, id: QuestionId) -> Result<impl Reply, Rejection> {
    trace!("querying question_id = {id:?}");

    let question = store
        .get_question(id.0)
        .await
        .map_err(ServiceError::DatabaseQueryError)?;
    debug!(question_found = question.is_some());

    match question {
        Some(question) => {
            info!("returning question with question_id = {id:?}");
            Ok(json(&question))
        },
        None => Err(ServiceError::QuestionNotFound(id.into()).into()),
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
pub async fn add_question(store: Store, question: Question) -> Result<impl Reply, Rejection> {
    trace!("adding a new question");
    let Question { title, content, tags, .. } = question;

    trace!("censoring title and content...");
    let (title, content) = tokio::try_join!(store.bad_words_api.censor(title), store.bad_words_api.censor(content))?;

    debug!("censored title: {title}");
    debug!("censored content: {content}");

    match store.add_question(Question { id: None, title, content, tags }).await {
        Ok(question) => {
            info!("created a question with question_id = {:?}", question.id);
            Ok(with_status(json(&question), StatusCode::CREATED))
        }
        Err(e) => Err(ServiceError::DatabaseQueryError(e).into()),
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
pub async fn update_question(store: Store, question_id: QuestionId, question: Question) -> Result<impl Reply, Rejection> {
    let QuestionId(id) = question_id;
    trace!("updating the question with question_id = {id}");
    let Question {
        title, content, tags, ..
    } = question;

    trace!("censoring title and content...");
    let (title, content) = tokio::try_join!(store.bad_words_api.censor(title), store.bad_words_api.censor(content))?;

    debug!("censored title: {title}");
    debug!("censored content: {content}");

    let censored_question = Question {
        id: Some(id.into()),
        title,
        content,
        tags,
    };

    match store.update_question(censored_question, id).await {
        Ok(question) => {
            info!("updated question with question_id = {id}");
            debug!(updated_question = ?question);
            Ok(with_status("Question updated", StatusCode::OK))
        }
        Err(e) => Err(ServiceError::DatabaseQueryError(e).into()),
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
pub async fn delete_question(store: Store, question_id: QuestionId) -> Result<impl Reply, Rejection> {
    let QuestionId(id) = question_id;

    trace!("deleting the question with question_id = {id:?}");


    match store.delete_question(id).await {
        Ok(true) => {
            info!("deleted question with question_id = {id:?}");
            Ok(with_status("Question deleted", StatusCode::OK))
        }
        Ok(false) => Err(ServiceError::QuestionNotFound(question_id.into()).into()),
        Err(e) => Err(ServiceError::DatabaseQueryError(e).into()),
    }
}
