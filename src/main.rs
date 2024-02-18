#![warn(clippy::all)]

use warp::Filter;

mod error;
mod filters;
mod handlers;
mod routes;
mod store;
mod types;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    log4rs::init_file("log4rs.yaml", Default::default()).expect("Failed to initialize log4rs");

    log::error!("This is an error!");
    log::info!("This is info!");
    log::warn!("This is a warning!");

    let log = warp::log::custom(|info| {
        eprintln!(
            "{} {} {} {:?} from {} with {:?}",
            info.method(),
            info.path(),
            info.status(),
            info.elapsed(),
            info.remote_addr()
                .map_or_else(|| "Unknown".to_string(), |addr| addr.to_string()),
            info.request_headers()
        );
    });

    // This is the store that holds the questions and answers.
    let store = store::Store::new();

    // This is the filter that will be used to serve the routes.
    // It is composed of the filters defined in the filters module.
    // It handles the CORS headers and the error handling.
    // It handles resources at the /questions and /answers endpoints.
    // The error handling is done by the return_error function defined in the error module.
    let filter = filters::questions_filter(&store)
        .or(filters::answers_filter(&store))
        .with(filters::cors())
        .with(log)
        .recover(error::return_error);

    // Start the server.
    warp::serve(filter).run(([0, 0, 0, 0], 3030)).await;

    Ok(())
}
