use argon2::Config;
use paseto::v2::local_paseto;
use rand::random;
use tracing::{debug, info, instrument, trace};
use warp::http::StatusCode;
use warp::reply::{json, with_status};
use warp::{Rejection, Reply};

use crate::error::ServiceError;
use crate::store::Store;
use crate::types::account::{Account, AccountId};

/// Hashes a password using Argon2.
///
/// Hashes a password using Argon2 and returns the hash as a string.
/// Hashing is done using a random salt and the default Argon2 configuration.
///
/// Salt is generated using the `rand` crate. Size of the salt is 32 bytes.
///
/// # Parameters
/// - `password` - The password to hash.
pub fn hash_password(password: &[u8]) -> String {
    let salt = random::<[u8; 32]>();
    let config = Config::default();
    argon2::hash_encoded(password, &salt, &config).unwrap()
}

/// Handler for the `POST /register` route.
///
/// This handler is used to register a new account.
///
/// # Parameters
/// - `store` - The [Store] to use for handling requests.
#[instrument(target = "webdev_books::accounts", skip(store))]
pub async fn register(store: Store, account: Account) -> Result<impl Reply, Rejection> {
    trace!("creating a new account");
    let Account { id, email, password } = account;
    trace!("hashing the password");
    let hashed_password = hash_password(password.as_bytes());

    let account = Account {
        id,
        email,
        password: hashed_password,
    };

    match store.add_account(account).await {
        Ok(_) => {
            info!("account created");
            Ok(with_status("Account created", StatusCode::CREATED))
        }
        Err(error) => Err(warp::reject::custom(error)),
    }
}

/// Verifies a password using Argon2.
///
/// Verifies a password using Argon2 and returns a boolean indicating whether the password is correct.
///
/// # Parameters
/// - `hashed` - The hashed password to verify against.
/// - `password` - The password to verify.
fn verify_password(hashed: &str, password: &str) -> Result<bool, argon2::Error> {
    argon2::verify_encoded(hashed, password.as_bytes())
}

/// Generates a PASETO token for an account.
///
/// Generates a PASETO token for an account using the account's ID.
///
/// # Parameters
/// - `account_id` - The ID of the account to generate a token for.
///
/// # Returns
/// A PASETO token as a string.
///
/// # Panics
/// - If the state cannot be serialized.
/// - If the token cannot be issued.
fn issue_token(account_id: AccountId) -> String {
    let state = serde_json::to_string(&account_id).expect("failed to serialize state");

    local_paseto(&state, None, "RANDOM WORDS WINTER MACINTOSH PC".as_bytes()).expect("failed to issue token")
}

/// Handler for the `POST /login` route.
///
/// This handler is used to log in an account.
///
/// # Parameters
/// - `store` - The [Store] to use for handling requests.
/// - `login` - The login details.
///
/// # Panics
/// - If the account ID is not found.
#[instrument(target = "webdev_books::accounts", skip(store))]
pub async fn login(store: Store, login: Account) -> Result<impl Reply, Rejection> {
    let Account { email, password, .. } = login;
    trace!("querying account with email = {email:?}");
    match store.get_account(&email).await {
        Ok(account) => {
            trace!("account found. verifying password");
            match verify_password(&account.password, &password) {
                Ok(true) => {
                    debug!("password verified. issuing token");
                    info!("account logged in, issuing token...");
                    Ok(json(&issue_token(account.id.expect("Account id not found"))))
                }
                Ok(false) => Err(warp::reject::custom(ServiceError::WrongPassword)),
                Err(error) => Err(warp::reject::custom(ServiceError::ArgonLibraryError(error))),
            }
        }
        Err(error) => Err(warp::reject::custom(error)),
    }
}
