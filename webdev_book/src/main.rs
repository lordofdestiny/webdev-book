#![warn(clippy::all)]

use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter, Registry};
use warp::Filter;

mod answers;
mod api;
mod authentication;
mod error;
mod filters;
mod questions;
mod store;
mod types;

/// The main function of the application.
///
/// It sets up the logger, the store, the migrations, and the routes.
/// Then it starts the server on port 3030.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up the logger filter
    let log_filter: EnvFilter = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "webdev_book=info,warp=error".to_owned())
        .parse()
        .expect("Cannot parse RUST_LOG");

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
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let store = store::Store::new(&db_url).await;

    sqlx::migrate!()
        .run(&store.connection)
        .await
        .expect("Cannot run migrations");

    /* This is the filter that will be used to serve the routes.
     * It is composed of the filters defined in the filters module.
     * It handles the CORS headers and the error handling.
     * It handles resources at the /questions and /answers endpoints.
     * The error handling is done by the return_error function defined in the error module.
     */
    let filter = authentication::filter(&store)
        .or(questions::filter(&store))
        .or(answers::filter(&store))
        .with(filters::cors())
        .with(warp::trace::request())
        .recover(error::return_error);

    // Start the server.
    warp::serve(filter).run(([0, 0, 0, 0], 3030)).await;

    Ok(())
}
