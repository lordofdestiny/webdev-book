mod error;
mod filters;
mod handlers;
mod routes;
mod store;
mod types;

use warp::Filter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This is the store that holds the questions and answers.
    let store = store::Store::new();

    // This is the filter that will be used to serve the routes.
    // It is composed of the filters defined in the filters module.
    // It handles the CORS headers and the error handling.
    // It handles resources at the /questions and /answers endpoints.
    // The error handling is done by the return_error function defined in the error module.
    let filter = filters::questions_filter(store.clone())
        .or(filters::answers_filter(store.clone()))
        .with(filters::cors())
        .recover(error::return_error);

    // Start the server.
    warp::serve(filter).run(([0, 0, 0, 0], 3030)).await;

    Ok(())
}
