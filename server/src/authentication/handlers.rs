use crate::store::Store;
use crate::types::account::Account;
use warp::{Rejection, Reply};

/// Handler for the `POST /register` route.
///
/// This handler is used to register a new account.
///
/// # Parameters
/// - `store` - The [Store] to use for handling requests.
pub async fn register(store: Store, account: Account) -> Result<impl Reply, Rejection> {
    match store.add_account(account).await {
        Ok(_) => Ok(warp::reply::with_status(
            "Account created",
            warp::http::StatusCode::CREATED,
        )),
        Err(error) => Err(warp::reject::custom(error)),
    }
}
