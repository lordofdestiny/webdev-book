use argon2::Config;
use rand::random;
use warp::http::StatusCode;
use warp::reply::with_status;
use warp::{Rejection, Reply};

use crate::store::Store;
use crate::types::account::Account;

/// Handler for the `POST /register` route.
///
/// This handler is used to register a new account.
///
/// # Parameters
/// - `store` - The [Store] to use for handling requests.
pub async fn register(store: Store, account: Account) -> Result<impl Reply, Rejection> {
    let Account { id, email, password } = account;
    let hashed_password = hash_password(password.as_bytes());

    println!("{:?} {:?} {:?}", id, email, hashed_password);

    let account = Account {
        id,
        email,
        password: hashed_password,
    };

    match store.add_account(account).await {
        Ok(_) => Ok(with_status("Account created", StatusCode::CREATED)),
        Err(error) => Err(warp::reject::custom(error)),
    }
}

pub fn hash_password(password: &[u8]) -> String {
    let salt = random::<[u8; 32]>();
    let config = Config::default();
    argon2::hash_encoded(password, &salt, &config).unwrap()
}
