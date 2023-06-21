use scraper::error::SelectorErrorKind;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Error: {0}")]
    GenericError(String),

    #[error("JSON error: {source}")]
    JSONError {
        #[from]
        source: serde_json::Error,
    },

    #[error("IO error: {source}")]
    IOError {
        #[from]
        source: std::io::Error,
    },

    #[error("HTTP error: {source}")]
    HTTPError {
        #[from]
        source: reqwest::Error,
    },

    #[error("Scrape error: {source}")]
    ScrapeError {
        #[from]
        source: SelectorErrorKind<'static>,
    },

    #[error("Parse error: {source}")]
    ParseError {
        #[from]
        source: regex::Error,
    },

    #[error("Date error: {source}")]
    DateError {
        #[from]
        source: chrono::ParseError,
    },

    #[error("Spotify client error: {source}")]
    SpotifyClientError {
        #[from]
        source: rspotify::ClientError,
    },

    #[error("Slack Error: {0}")]
    SlackError(String),
}

// slack_rust does not implement the std::error::Error trait, so we have to do this manually
impl From<slack_rust::error::Error> for AppError {
    fn from(err: slack_rust::error::Error) -> Self {
        Self::SlackError(err.to_string())
    }
}

// If we needed our own to_string() (without implementing some trait), this is
// how we would do it:
//
// use std::fmt;
//
// impl AppError {
//     pub fn to_string(&self) -> String {
//         match self {
//             AppError::JSONError { source } => source.to_string(),
//             AppError::IOError { source } => source.to_string(),
//         }
//     }
// }
