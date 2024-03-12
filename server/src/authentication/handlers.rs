use crate::store::Store;
use crate::types::account::Account;
use warp::{Rejection, Reply};

pub async fn register(store: Store, account: Account) -> Result<impl Reply, Rejection> {
    match store.add_account(account).await {
        Ok(_) => Ok(warp::reply::with_status(
            "Account created",
            warp::http::StatusCode::CREATED,
        )),
        Err(error) => Err(warp::reject::custom(error)),
    }
}
