use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum PodbeanError {
    ApiError { code: u16, message: String },
    RateLimitError { retry_after: Option<u64> },
    NetworkError(reqwest::Error),
    SerializationError(serde_json::Error),
    UrlParseError(url::ParseError),
    AuthError(String),
    OtherError(String),
}

impl fmt::Display for PodbeanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PodbeanError::ApiError { code, message } => {
                write!(f, "API error {}: {}", code, message)
            }
            PodbeanError::RateLimitError { retry_after } => {
                if let Some(seconds) = retry_after {
                    write!(f, "Rate limit exceeded. Retry after {} seconds", seconds)
                } else {
                    write!(f, "Rate limit exceeded")
                }
            }
            PodbeanError::NetworkError(e) => write!(f, "Network error: {}", e),
            PodbeanError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            PodbeanError::UrlParseError(e) => write!(f, "URL parse error: {}", e),
            PodbeanError::AuthError(msg) => write!(f, "Authentication error: {}", msg),
            PodbeanError::OtherError(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl Error for PodbeanError {}

impl From<reqwest::Error> for PodbeanError {
    fn from(err: reqwest::Error) -> Self {
        PodbeanError::NetworkError(err)
    }
}

impl From<serde_json::Error> for PodbeanError {
    fn from(err: serde_json::Error) -> Self {
        PodbeanError::SerializationError(err)
    }
}

impl From<url::ParseError> for PodbeanError {
    fn from(err: url::ParseError) -> Self {
        PodbeanError::UrlParseError(err)
    }
}
