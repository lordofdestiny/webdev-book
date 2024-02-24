use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BadWord {
    original: String,
    deviations: u32,
    start: usize,
    end: usize,
}

fn default_censor_char() -> char {
    '*'
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BadWords {
    pub content: String,
    pub censored_content: String,
    pub bad_words_list: Vec<BadWord>,
    #[serde(default = "default_censor_char")]
    pub censor_char: char,
}

impl BadWords {
    pub fn restore_word<F>(&self, index: usize, predicate: F) -> Option<String>
    where
        F: FnOnce(&BadWord) -> bool,
    {
        let bad_word = self.bad_words_list.get(index)?;

        if !predicate(bad_word) {
            return None;
        }

        let BadWord {
            start, end, original, ..
        } = bad_word;
        let mut new_str = self.censored_content.clone();
        new_str.replace_range(*start..*end, original);
        Some(new_str)
    }

    pub fn restore<F>(&mut self, predicate: F)
    where
        F: FnOnce(&BadWord) -> bool + Clone,
    {
        for i in 0..self.bad_words_list.len() {
            if let Some(value) = self.restore_word(i, predicate.clone()) {
                self.censored_content = value;
            }
        }
    }

    pub fn apply_word<F>(&self, index: usize, predicate: F) -> Option<String>
    where
        F: FnOnce(&BadWord) -> bool,
    {
        let bad_word = self.bad_words_list.get(index)?;

        if !predicate(bad_word) {
            return None;
        }

        let BadWord { start, end, .. } = bad_word;
        let mut new_str = self.censored_content.clone();
        let replacement = String::from(self.censor_char).repeat(end - start);
        if *end == self.content.len() {
            new_str.replace_range(*start.., &replacement);
        } else {
            new_str.replace_range(*start..*end, &replacement);
        };
        Some(new_str)
    }

    pub fn apply<F>(&mut self, predicate: F)
    where
        F: FnOnce(&BadWord) -> bool + Clone,
    {
        for i in 0..self.bad_words_list.len() {
            if let Some(value) = self.apply_word(i, predicate.clone()) {
                self.censored_content = value;
            }
        }
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
    api_key: String,
    url: String,
    client: reqwest::Client,
}

impl BadWordsAPI {
    const API_ENDPOINT: &'static str = "https://api.apilayer.com/bad_words";

    pub fn new(api_key: String, censor_char: char) -> Self {
        BadWordsAPI {
            censor_char,
            api_key,
            url: format!("{}?censor_character={censor_char}", Self::API_ENDPOINT),
            client: reqwest::Client::new(),
        }
    }

    pub async fn censor(&self, text: &str) -> Result<BadWordsAPIResponse, BadWordsAPIError> {
        let response = match self
            .client
            .post(&self.url)
            .header("apikey", &self.api_key)
            .body(text.to_owned())
            .send()
            .await
        {
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

        let result: Result<BadWordsAPIResponse, _> = serde_json::from_str(&response_body);

        match result {
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

#[derive(thiserror::Error, Debug)]
enum BadWordsAPIError {
    #[error("error with executing the request")]
    RequestError(#[from] reqwest::Error),
    #[error("error with deserializing json response")]
    DeserializationError(#[from] serde_json::Error),
    #[error("header field contains invalid unicode")]
    BadHeaderValue(#[from] reqwest::header::ToStrError),
}

#[tokio::main]
async fn main() -> Result<std::process::ExitCode, BadWordsAPIError> {
    let api_key = std::env::var("API_LAYER_KEY").unwrap_or("KHFM68sNbwzuPqELmjnevUv2k17EdiBE".to_owned());
    // .expect("API_LAYER_KEY not found");

    let api = BadWordsAPI::new(api_key, '*');
    let result = api.censor("a list of shit words, you son of a bitch").await?;

    match result {
        BadWordsAPIResponse::BadWordsResponse(mut bad_words) => {
            println!("Censored: {}", bad_words.censored_content);
            bad_words.restore(|word| word.original.contains("son of a"));
            bad_words.restore(|word| word.original.contains("bitch"));
            println!("Restored: {}", bad_words.censored_content);
            bad_words.apply(|_| true);
            println!("Censored: {}", bad_words.censored_content);
            Ok(std::process::ExitCode::SUCCESS)
        }
        BadWordsAPIResponse::ErrorResponse { message, status_code } => {
            eprintln!("{message}; status_code = {status_code}");
            return Ok(std::process::ExitCode::from(1));
        }
    }
}
