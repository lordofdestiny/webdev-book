use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::{Deserialize, Serialize};
use tracing::{error, instrument, trace};

use crate::error::{APILayerError, ServiceError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadWord {
    pub original: String,
    pub word: String,
    pub deviations: i64,
    pub info: i64,
    #[serde(rename = "replacedLen")]
    pub replaced_len: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadWordsResponse {
    pub content: String,
    pub bad_words_total: i64,
    pub bad_words_list: Vec<BadWord>,
    pub censored_content: String,
}

#[derive(Debug)]
pub struct BadWordsAPI {
    url: String,
    client: ClientWithMiddleware,
}

//noinspection DuplicatedCode
#[derive(thiserror::Error, Debug)]
pub enum BadWordsAPIBuildError {
    #[error("invalid header value: {0}")]
    BadAPIKeyValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("failed to build client object: {0}")]
    ClientBuildError(#[from] reqwest::Error),
}

impl BadWordsAPI {
    const API_ENDPOINT: &'static str = "https://api.apilayer.com/bad_words";
    fn url(censor_char: char) -> String {
        format!("{}?censor_character={censor_char}", Self::API_ENDPOINT)
    }

    //noinspection DuplicatedCode
    pub fn build(api_key: &str, censor_char: char) -> Result<Self, BadWordsAPIBuildError> {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("apikey", api_key.parse()?);

        let client = reqwest::Client::builder().default_headers(headers).build()?;

        let client = ClientBuilder::new(client)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        Ok(BadWordsAPI {
            url: Self::url(censor_char),
            client,
        })
    }

    #[instrument(level = "debug", skip(self))]
    pub async fn check_profanity(&self, text: String) -> Result<BadWordsResponse, ServiceError> {
        trace!("checking profanity in text: {}", text);
        let response = match self.client.post(&self.url).body(text).send().await {
            Ok(response) => response,
            Err(e) => {
                return Err(e.into());
            }
        };
        trace!(test_censored = response.status().is_success());

        if !response.status().is_success() {
            let client_error = response.status().is_client_error();
            let error = APILayerError::transform_error(response).await;
            return Err(if client_error {
                trace!("client_error: {}", error.message);
                ServiceError::ClientError(error)
            } else {
                trace!("server_error: {}", error.message);
                ServiceError::ServerError(error)
            });
        }

        response.json().await.map_err(|e| e.into())
    }

    #[instrument(level = "debug", skip(self))]
    pub async fn censor(&self, text: String) -> Result<String, ServiceError> {
        trace!("censoring text: {}", text);
        self.check_profanity(text).await.map(|res| res.censored_content)
    }
}
