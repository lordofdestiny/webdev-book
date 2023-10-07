use warp::{
    filters::{body::BodyDeserializeError, cors::CorsForbidden},
    http::StatusCode,
    reject::Reject,
    Rejection, Reply,
};

#[derive(Debug)]
pub struct AnswerContentMissing;
impl std::fmt::Display for AnswerContentMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing answer content")
    }
}
impl Reject for AnswerContentMissing {}

#[derive(Debug)]
pub struct PaginationError(pub std::num::ParseIntError);

impl std::fmt::Display for PaginationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Cannot parse parameter: {}", self.0)
    }
}

impl Reject for PaginationError {}
#[derive(Debug)]
pub struct QuestionNotFound;

impl std::fmt::Display for QuestionNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Question not found")
    }
}

impl Reject for QuestionNotFound {}

macro_rules! impl_return_error {
        ($($type:ty : $status_code:expr,)*) => {
            pub async fn return_error(rej: Rejection) -> Result<impl Reply, Rejection> {
                $(if let Some(error) = rej.find::<$type>() {
                    Ok(warp::reply::with_status(
                        error.to_string(),
                        $status_code
                    ))
                } else)* {
                    Ok(warp::reply::with_status(
                        "Route not found".to_string(),
                        StatusCode::NOT_FOUND
                    ))
                }
            }
        };
    }

impl_return_error!(
    CorsForbidden : StatusCode::FORBIDDEN,
    QuestionNotFound : StatusCode::NOT_FOUND,
    PaginationError : StatusCode::RANGE_NOT_SATISFIABLE,
    AnswerContentMissing : StatusCode::UNPROCESSABLE_ENTITY,
    BodyDeserializeError : StatusCode::UNPROCESSABLE_ENTITY,
);
