use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::ErrorKind;
use std::str::FromStr;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::RwLock;
use warp::{
    body::BodyDeserializeError,
    cors::CorsForbidden,
    http::{Method, StatusCode},
    reject::Reject,
    Filter, Rejection, Reply,
};

trait NextId
where
    Self: FromStr<Err = std::io::Error>,
{
    fn counter() -> &'static AtomicUsize;

    fn next() -> Self {
        let id = Self::counter().fetch_add(1, Ordering::SeqCst);
        Self::from_str(&id.to_string()).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuestionId(pub String);

impl NextId for QuestionId {
    fn counter() -> &'static AtomicUsize {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        &COUNTER
    }
}

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
    id: Option<QuestionId>,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct AnswerId(String);

impl NextId for AnswerId {
    fn counter() -> &'static AtomicUsize {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        &COUNTER
    }
}

impl FromStr for AnswerId {
    type Err = std::io::Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        match id.is_empty() {
            false => Ok(AnswerId(id.to_string())),
            true => Err(Self::Err::new(ErrorKind::InvalidInput, "No id provided")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Answer {
    id: AnswerId,
    content: String,
    question_id: QuestionId,
}
#[derive(Debug)]
struct AnswerContentMissing;
impl std::fmt::Display for AnswerContentMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing answer content")
    }
}
impl Reject for AnswerContentMissing {}

#[derive(Clone)]
struct Store {
    questions: Arc<RwLock<HashMap<QuestionId, Question>>>,
    answers: Arc<RwLock<HashMap<AnswerId, Answer>>>,
}

impl Store {
    fn new() -> Store {
        Store {
            questions: Arc::new(RwLock::new(Self::init())),
            answers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../questions.json");
        serde_json::from_str(file).expect("Failed to parse questions.json")
    }
}

#[derive(Debug)]
enum PaginationError {
    MissingParameters,
    InvalidRange(usize, usize),
    ParseError(std::num::ParseIntError),
}

impl std::fmt::Display for PaginationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PaginationError::MissingParameters => write!(f, "Missing parameters"),
            PaginationError::InvalidRange(start, end) => {
                write!(f, "Invalid range: [{start}, {end}]")
            }
            PaginationError::ParseError(err) => write!(f, "Cannot parse parameter: {}", err),
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
    let questions = store.questions.read().await;
    let res: Vec<_> = match params.is_empty() {
        true => questions.values().cloned().collect(),
        false => {
            let pagination = Pagination::extract(params)?;
            questions
                .values()
                .page_iter_slice(pagination)?
                .cloned()
                .collect()
        }
    };
    Ok(warp::reply::json(&res))
}

async fn get_question(id: QuestionId, store: Store) -> Result<impl Reply, Rejection> {
    match store.questions.read().await.get(&id) {
        Some(q) => Ok(warp::reply::json(q)),
        None => return Err(warp::reject::custom(QuestionNotFound)),
    }
}

async fn add_question(data: Question, store: Store) -> Result<impl Reply, Rejection> {
    let id = QuestionId::next();
    store.questions.write().await.insert(
        id.clone(),
        Question {
            id: Some(id),
            ..data
        },
    );

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
    id: QuestionId,
    question: Question,
    store: Store,
) -> Result<impl Reply, Rejection> {
    match store.questions.write().await.get_mut(&id) {
        Some(q) => {
            *q = Question {
                id: Some(id),
                ..question
            };
            Ok(warp::reply::with_status("Question updated", StatusCode::OK))
        }
        None => Err(warp::reject::custom(QuestionNotFound)),
    }
}

async fn delete_question(id: QuestionId, store: Store) -> Result<impl Reply, Rejection> {
    match store.questions.write().await.remove(&id) {
        Some(_) => Ok(warp::reply::with_status("Question deleted", StatusCode::OK)),
        None => Err(warp::reject::custom(QuestionNotFound)),
    }
}

async fn return_error(rej: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = rej.find::<PaginationError>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::RANGE_NOT_SATISFIABLE,
        ))
    } else if let Some(error) = rej.find::<AnswerContentMissing>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
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

async fn add_answer(
    question_id: QuestionId,
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl Reply, Rejection> {
    if !store.questions.read().await.contains_key(&question_id) {
        return Err(warp::reject::custom(QuestionNotFound));
    }

    let answer = Answer {
        id: AnswerId::next(),
        content: params
            .get("content")
            .ok_or(AnswerContentMissing)?
            .to_string(),
        question_id,
    };

    store
        .answers
        .write()
        .await
        .insert(answer.id.clone(), answer);

    Ok(warp::reply::with_status(
        "Answer added",
        StatusCode::CREATED,
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        .and(warp::path!("questions"))
        .and(warp::query())
        .and(store_filter.clone())
        .and_then(get_questions);

    let get_question = warp::get()
        .and(warp::path!("questions" / QuestionId))
        .and(store_filter.clone())
        .and_then(get_question);

    let add_question = warp::post()
        .and(warp::path!("questions"))
        .and(warp::body::json())
        .and(store_filter.clone())
        .and_then(add_question);

    let update_question = warp::put()
        .and(warp::path!("questions" / QuestionId))
        .and(warp::body::json())
        .and(store_filter.clone())
        .and_then(update_question);

    let delete_question = warp::delete()
        .and(warp::path!("questions" / QuestionId))
        .and(store_filter.clone())
        .and_then(delete_question);

    let add_answer = warp::post()
        .and(warp::path!("questions" / QuestionId / "answers"))
        .and(warp::body::form())
        .and(store_filter.clone())
        .and_then(add_answer);

    let routes = get_questions
        .or(get_question)
        .or(add_question)
        .or(add_answer)
        .or(update_question)
        .or(delete_question)
        .with(cors)
        .recover(return_error);

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;

    Ok(())
}
