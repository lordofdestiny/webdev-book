use warp::filters::BoxedFilter;
use warp::{http::Method, Filter, Reply};

mod error;
mod handlers;
mod routes;
mod store;
mod types;

use types::{Answer, AnswerId, NextId, Question, QuestionId};

pub fn questions_filter() -> BoxedFilter<(impl Reply,)> {
    use crate::handlers::questions as handlers;
    use crate::routes::questions as routes;

    routes::get_questions()
        .and_then(handlers::get_questions)
        .or(routes::get_question().and_then(handlers::get_question))
        .or(routes::add_question().and_then(handlers::add_question))
        .or(routes::update_question().and_then(handlers::update_question))
        .or(routes::delete_question().and_then(handlers::delete_question))
        .boxed()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let add_answer = routes::answers::add_answer().and_then(handlers::answers::add_answer);

    let routes = questions_filter()
        .or(add_answer)
        .with(cors)
        .recover(error::return_error);

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;

    Ok(())
}
