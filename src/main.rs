#![warn(clippy::all)]

use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter, Registry};
use warp::Filter;

mod error;
mod filters;
mod handlers;
mod routes;
mod store;
mod types;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up the logger filter
    let log_filter: EnvFilter = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "webdev_book=info,warp=error".to_owned())
        .parse()?;

    // Set up rolling file
    let file_appender = tracing_appender::rolling::hourly("logs", "webdev-book.log");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    // Set up the logger for the application.
    // Log to the console and to the file.
    Registry::default()
        .with(fmt::Layer::default().with_ansi(false).with_writer(file_writer))
        .with(fmt::Layer::default().with_writer(std::io::stdout))
        .with(log_filter)
        .init();

    // This is the store that holds the questions and answers.
    let store = store::Store::new("postgres://postgres:admin@localhost:5432/webdev_book").await;

    // This is the filter that will be used to serve the routes.
    // It is composed of the filters defined in the filters module.
    // It handles the CORS headers and the error handling.
    // It handles resources at the /questions and /answers endpoints.
    // The error handling is done by the return_error function defined in the error module.
    let filter = filters::questions_filter(&store)
        .or(filters::answers_filter(&store))
        .with(filters::cors())
        .with(warp::trace::request())
        .recover(error::return_error);

    // Start the server.
    warp::serve(filter).run(([0, 0, 0, 0], 3030)).await;

    Ok(())
}
