use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BadWord {
    /// Value replaced in the original string
    original: String,
    /// Detected word
    word: String,
    /// Number of differences between original and word
    deviations: u32,
    /// Start of the replaced word
    start: usize,
    /// End of the replaced word
    end: usize,
}

const DEFAULT_CENSOR_CHAR: fn() -> char = || '*';

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BadWords {
    content: String,
    censored_content: String,
    bad_words_list: Vec<BadWord>,
    #[serde(default = "DEFAULT_CENSOR_CHAR")]
    pub censor_char: char,
}

impl BadWords {
    #[allow(dead_code)]
    pub fn original(&self) -> &str {
        &self.content
    }

    #[allow(dead_code)]
    pub fn censored(&self) -> &str {
        &self.censored_content
    }

    #[allow(dead_code)]
    pub fn bad_words(&self) -> &Vec<BadWord> {
        &self.bad_words_list
    }

    pub fn censor_original<F>(&self, predicate: F) -> String
    where
        F: FnMut(&&BadWord) -> bool,
    {
        self.bad_words_list.iter().filter(predicate).fold(
            self.content.clone(),
            |mut result, BadWord { start, end, .. }| {
                let replacement = String::from(self.censor_char).repeat(end - start);
                result.replace_range(*start..*end, &replacement);
                result
            },
        )
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum BadWordsAPIResponse {
    BadWordsResponse(BadWords),
    ErrorResponse {
        message: String,
        #[serde(skip)]
        status_code: reqwest::StatusCode,
    },
}

impl BadWordsAPIResponse {
    fn finalize(&mut self, new_status_code: reqwest::StatusCode, censor_char: char) {
        match self {
            Self::BadWordsResponse(ref mut bad_words) => {
                bad_words.censor_char = censor_char;
                bad_words.bad_words_list.sort_by_key(|word| word.start);
            }
            Self::ErrorResponse {
                ref mut status_code, ..
            } => *status_code = new_status_code,
        }
    }
}

struct BadWordsAPI {
    censor_char: char,
    url: String,
    client: reqwest::Client,
}

#[derive(thiserror::Error, Debug)]
enum BadWordsAPIBuildError {
    #[error("invalid header value: {0}")]
    BadAPIKeyValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("failed to build client: {0}")]
    ClientBuildError(#[from] reqwest::Error),
}

#[derive(thiserror::Error, Debug)]
enum BadWordsAPIError {
    #[error("error with executing the request")]
    RequestError(#[from] reqwest::Error),
    #[error("error with deserializing json response")]
    DeserializationError(#[from] serde_json::Error),
    #[error("header field contains invalid unicode")]
    BadHeaderValue(#[from] reqwest::header::ToStrError),
}

impl BadWordsAPI {
    const API_ENDPOINT: &'static str = "https://api.apilayer.com/bad_words";

    pub fn build(api_key: String, censor_char: char) -> Result<Self, BadWordsAPIBuildError> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("apikey", api_key.parse()?);

        Ok(BadWordsAPI {
            censor_char,
            url: format!("{}?censor_character={censor_char}", Self::API_ENDPOINT),
            client: reqwest::Client::builder().default_headers(headers).build()?,
        })
    }

    pub async fn censor(&self, text: &str) -> Result<BadWordsAPIResponse, BadWordsAPIError> {
        let response = match self.client.post(&self.url).body(text.to_owned()).send().await {
            Ok(response) => response,
            Err(e) => {
                eprintln!("censoring request failed: {e}");
                return Err(e.into());
            }
        };

        let status_code = response.status();

        let response_body = match response.text().await {
            Ok(body) => body,
            Err(e) => {
                eprintln!("censoring request failed: {e}");
                return Err(e.into());
            }
        };

        match serde_json::from_str::<BadWordsAPIResponse>(&response_body) {
            Ok(mut bad_words) => {
                bad_words.finalize(status_code, self.censor_char);
                Ok(bad_words)
            }
            Err(e) => {
                eprintln!("parsing failed");
                Err(e.into())
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<std::process::ExitCode, Box<dyn std::error::Error>> {
    let api_key = std::env::var("API_LAYER_KEY").expect("API_LAYER_KEY not found");

    let api = BadWordsAPI::build(api_key, '*')?;
    let result = api.censor("a list of shit words, you son of a bitch").await?;

    match result {
        BadWordsAPIResponse::BadWordsResponse(bad_words) => {
            println!("Censored: {}", bad_words.censored_content);
            println!("Censored: {}", bad_words.censor_original(|_| true));

            Ok(std::process::ExitCode::SUCCESS)
        }
        BadWordsAPIResponse::ErrorResponse { message, status_code } => {
            eprintln!("{message}; status_code = {status_code}");
            return Ok(std::process::ExitCode::from(1));
        }
    }
}
