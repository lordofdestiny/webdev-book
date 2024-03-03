use std::collections::HashMap;

use tracing::{debug, info, instrument, trace};
use warp::{Rejection, Reply};
use warp::http::StatusCode;
use warp::reply::{json, with_status};

use crate::{
    error::ServiceError,
    store::Store,
    types::{Pagination, Question, QuestionId},
};
use crate::types::NewQuestion;

/// Handler for `GET /questions?start={}&limit={}`
///
/// Query parameters:
/// - offset: usize - default `0`
/// - limit: usize - default `usize::MAX`
///
/// Returns a list no more than `limit` questions, starting from `offset`.
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
            Ok(json(&questions))
        }
        Err(e) => Err(ServiceError::DatabaseQueryError(e).into()),
    }
}

/// Handler for `GET /questions/{id}`
///
/// Returns the question with the given id
#[instrument(target = "webdev_book::questions", skip(store))]
pub async fn get_question(store: Store, id: QuestionId) -> Result<impl Reply, Rejection> {
    trace!("querying question_id = {id}");

    let question = store
        .get_question(id.0)
        .await
        .map_err(ServiceError::DatabaseQueryError)?;
    debug!(question_found = question.is_some());

    match question {
        Some(question) => Ok(json(&question)),
        None => Err(ServiceError::QuestionNotFound(id.into()).into()),
    }
}

/// Handler for `POST /questions`
///
/// Creates a new question
#[instrument(target = "webdev_book::questions", skip(store))]
pub async fn add_question(store: Store, new_question: NewQuestion) -> Result<impl Reply, Rejection> {
    trace!("adding a new question");

    let NewQuestion { title, content, tags } = new_question;

    trace!("censoring title...");
    let title = store.bad_words_api.censor(title).await?;
    debug!("censored title: {title}");

    trace!("censoring content...");
    let content = store.bad_words_api.censor(content).await?;
    debug!("censored content: {content}");

    match store.add_question(NewQuestion { title, content, tags }).await {
        Ok(question) => {
            info!("created a question with question_id = {}", question.id);
            Ok(with_status(json(&question), StatusCode::CREATED))
        }
        Err(e) => Err(ServiceError::DatabaseQueryError(e).into()),
    }
}

/// Handler for `PUT /questions/{id}`
///
/// Updates the question with the given id
#[instrument(target = "webdev_book::questions", skip(store))]
pub async fn update_question(store: Store, id: QuestionId, question: Question) -> Result<impl Reply, Rejection> {
    trace!("updating the question with question_id = {id}");
    let Question {
        title, content, tags, ..
    } = question;

    trace!("censoring title...");
    let title = store.bad_words_api.censor(title).await?;
    debug!("censored title: {title}");

    trace!("censoring content...");
    let content = store.bad_words_api.censor(content).await?;
    debug!("censored content: {content}");

    let censored_question = Question {
        id: id.clone(),
        title,
        content,
        tags,
    };

    match store.update_question(censored_question, id.0).await {
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
#[instrument(target = "webdev_book::questions", skip(store))]
pub async fn delete_question(store: Store, id: QuestionId) -> Result<impl Reply, Rejection> {
    trace!("deleting the question with question_id = {id}");

    match store.delete_question(id.0).await {
        Ok(true) => {
            info!("deleted question with question_id = {id}");
            Ok(with_status("Question deleted", StatusCode::OK))
        }
        Ok(false) => Err(ServiceError::QuestionNotFound(id.into()).into()),
        Err(e) => Err(ServiceError::DatabaseQueryError(e).into()),
    }
}
