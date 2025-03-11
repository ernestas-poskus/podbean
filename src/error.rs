//! Error types for the Podbean API client.
//!
//! This module defines the various error types that can occur when
//! interacting with the Podbean API.

use std::error::Error;
use std::fmt;

/// Possible errors that can occur when using the Podbean API client.
#[derive(Debug)]
pub enum PodbeanError {
    /// Error returned by the Podbean API.
    ApiError {
        /// HTTP status code
        code: u16,
        /// Error message
        message: String,
    },

    /// Rate limit exceeded error.
    RateLimitError {
        /// Optional number of seconds to wait before retrying
        retry_after: Option<u64>,
    },

    /// Network error when communicating with the API.
    NetworkError(reqwest::Error),

    /// Error deserializing JSON response.
    SerializationError(serde_json::Error),

    /// Error parsing a URL.
    UrlParseError(url::ParseError),

    /// Authentication-related error.
    AuthError(String),

    /// Any other type of error.
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

impl Error for PodbeanError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PodbeanError::NetworkError(e) => Some(e),
            PodbeanError::SerializationError(e) => Some(e),
            PodbeanError::UrlParseError(e) => Some(e),
            _ => None,
        }
    }
}

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
