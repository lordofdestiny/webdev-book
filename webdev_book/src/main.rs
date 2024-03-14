#![warn(clippy::all)]

use dotenv;
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

use config::Config;

/// The configuration of the application.
///
/// Values are read from the `setup.toml` file.
#[derive(Debug, serde::Deserialize, PartialEq)]
pub struct Args {
    /// The log level for the application.
    log_level: String,
    /// The host of the database.
    database_host: String,
    /// The port of the database at host.
    database_port: u16,
    /// The name of the database.
    database_name: String,
    /// The user to connect to the database.
    database_user: String,
    /// The password to connect to the database.
    database_password: String,
}

impl Args {
    /// Returns the database URL.
    pub fn database_url(&self) -> String {
        let Self {
            database_host: host,
            database_port: port,
            database_name: name,
            database_user: user,
            database_password: password,
            ..
        } = self;

        format!("postgres://{user}:{password}@{host}:{port}/{name}")
    }
}

/// The main function of the application.
///
/// It sets up the logger, the store, the migrations, and the routes.
/// Then it starts the server on port 3030.
#[tokio::main]
async fn main() -> Result<(), error::ServiceError> {
    // Load the environment variables from the .env file.
    dotenv::dotenv().ok();

    // Check if the environment variables are set.
    if let Err(_) = std::env::var("API_LAYER_KEY") {
        panic!("API_LAYER_KEY is not set");
    }

    if let Err(_) = std::env::var("PASETO_KEY") {
        panic!("PASETO_KEY is not set");
    }

    let port = std::env::var("PORT")
        .ok()
        .map(|val| val.parse::<u16>())
        .unwrap_or(Ok(8080))?;

    // Load the configuration from the setup file.
    let config: Args = Config::builder()
        .add_source(config::File::with_name("setup"))
        .build()?
        .try_deserialize()?;

    // Set up the logger filter
    let Args { ref log_level, .. } = config;
    let log_filter: EnvFilter = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| format!("webdev_book={log_level},warp={log_level}"))
        .parse()
        .unwrap();

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
    let db_url = config.database_url();
    let store = store::Store::build(&db_url).await?;

    sqlx::migrate!().run(&store.connection).await?;

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
    warp::serve(filter).run(([0, 0, 0, 0], port)).await;

    Ok(())
}
