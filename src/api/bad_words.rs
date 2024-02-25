use serde::{Deserialize, Serialize};
use tracing::error;

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
    client: reqwest::Client,
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
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("apikey", api_key.parse()?);

        Ok(BadWordsAPI {
            url: Self::url(censor_char),
            client: reqwest::Client::builder().default_headers(headers).build()?,
        })
    }

    pub async fn censor(&self, text: String) -> Result<BadWordsResponse, ServiceError> {
        let response = match self.client.post(&self.url).body(text).send().await {
            Ok(response) => response,
            Err(e) => {
                return Err(ServiceError::ExternalAPIError(e));
            }
        };

        if !response.status().is_success() {
            let client_error = response.status().is_client_error();
            let error = APILayerError::transform_error(response).await;
            return Err(if client_error {
                ServiceError::ClientError(error)
            } else {
                ServiceError::ServerError(error)
            });
        }

        response.json().await.map_err(ServiceError::ExternalAPIError)
    }
}
