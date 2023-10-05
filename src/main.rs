use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::ErrorKind;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{
    body::BodyDeserializeError,
    cors::CorsForbidden,
    http::{Method, StatusCode},
    reject::Reject,
    Filter, Rejection, Reply,
};

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuestionId(pub String);

impl FromStr for QuestionId {
    type Err = std::io::Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        match id.is_empty() {
            false => Ok(QuestionId(id.to_string())),
            true => Err(Self::Err::new(ErrorKind::InvalidInput, "No id provided")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Clone)]
struct Store {
    questions: Arc<RwLock<HashMap<QuestionId, Question>>>,
}

impl Store {
    fn new() -> Store {
        Store {
            questions: Arc::new(RwLock::new(Self::init())),
        }
    }

    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../questions.json");
        serde_json::from_str(file).expect("Failed to parse questions.json")
    }
}

#[derive(Debug)]
enum PaginationError {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    InvalidRange(usize, usize),
}

impl std::fmt::Display for PaginationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PaginationError::ParseError(err) => write!(f, "Cannot parse parameter: {}", err),
            PaginationError::MissingParameters => write!(f, "Missing parameters"),
            PaginationError::InvalidRange(start, end) => {
                write!(f, "Invalid range: [{start}, {end}]",)
            }
        }
    }
}

impl Reject for PaginationError {}

#[derive(Debug)]
struct Pagination {
    start: usize,
    end: usize,
}

impl Pagination {
    fn extract(params: HashMap<String, String>) -> Result<Self, PaginationError> {
        match (params.get("start"), params.get("end")) {
            (Some(start), Some(end)) => {
                let start = start
                    .parse::<usize>()
                    .map_err(PaginationError::ParseError)?;
                let end = end.parse::<usize>().map_err(PaginationError::ParseError)?;
                Ok(Pagination { start, end })
            }
            _ => Err(PaginationError::MissingParameters),
        }
    }
}

trait PageSlice {
    type Item;

    fn page_slice(&self, pagination: Pagination) -> Result<&[Self::Item], PaginationError>;
}

impl<T> PageSlice for [T] {
    type Item = T;

    fn page_slice(
        &self,
        Pagination { start, end }: Pagination,
    ) -> Result<&[Self::Item], PaginationError> {
        match start <= end && end < self.len() {
            true => Ok(&self[start..=end]),
            false => Err(PaginationError::InvalidRange(start, end)),
        }
    }
}

trait PageIterSlice
where
    Self: Iterator,
{
    fn page_iter_slice<'a>(
        &'a mut self,
        pagination: Pagination,
    ) -> Result<Box<dyn Iterator<Item = Self::Item> + 'a>, PaginationError>;
}

impl<T> PageIterSlice for T
where
    Self: Iterator + ExactSizeIterator,
{
    fn page_iter_slice<'a>(
        &'a mut self,
        Pagination { start, end }: Pagination,
    ) -> Result<Box<dyn Iterator<Item = Self::Item> + 'a>, PaginationError> {
        match start <= end && end < self.len() {
            true => Ok(Box::new(self.skip(start).take(end - start + 1))),
            false => Err(PaginationError::InvalidRange(start, end)),
        }
    }
}

async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl Reply, Rejection> {
    if !params.is_empty() {
        let pagination = Pagination::extract(params)?;

        let res: Vec<_> = store
            .questions
            .read()
            .await
            .values()
            .page_iter_slice(pagination)?
            .cloned()
            .collect();

        Ok(warp::reply::json(&res))
    } else {
        let res: Vec<_> = store.questions.read().await.values().cloned().collect();
        Ok(warp::reply::json(&res))
    }
}

async fn add_question(store: Store, question: Question) -> Result<impl Reply, Rejection> {
    store
        .questions
        .write()
        .await
        .insert(question.id.clone(), question);

    Ok(warp::reply::with_status(
        "Question added",
        StatusCode::CREATED,
    ))
}

#[derive(Debug)]
struct QuestionNotFound;

impl std::fmt::Display for QuestionNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Question not found")
    }
}

impl Reject for QuestionNotFound {}

async fn update_question(
    id: String,
    store: Store,
    question: Question,
) -> Result<impl Reply, Rejection> {
    match store.questions.write().await.get_mut(&QuestionId(id)) {
        Some(q) => {
            *q = question;
            Ok(warp::reply::with_status("Question updated", StatusCode::OK))
        }
        None => return Err(warp::reject::custom(QuestionNotFound)),
    }
}

async fn delete_question(id: String, store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    match store.questions.write().await.remove(&QuestionId(id)) {
        Some(_) => Ok(warp::reply::with_status("Question deleted", StatusCode::OK)),
        None => return Err(warp::reject::custom(QuestionNotFound)),
    }
}

async fn return_error(rej: Rejection) -> Result<impl Reply, Rejection> {
    println!("{:?}", rej);
    if let Some(error) = rej.find::<PaginationError>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::RANGE_NOT_SATISFIABLE,
        ))
    } else if let Some(error) = rej.find::<QuestionNotFound>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::NOT_FOUND,
        ))
    } else if let Some(error) = rej.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else if let Some(error) = rej.find::<BodyDeserializeError>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{:?}", QuestionId::from_str("1").unwrap());

    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ]);

    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter.clone())
        .and_then(get_questions);

    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(add_question);

    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(update_question);

    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(delete_question);

    let routes = get_questions
        .or(add_question)
        .or(update_question)
        .or(delete_question)
        .with(cors)
        .recover(return_error);

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;

    Ok(())
}
