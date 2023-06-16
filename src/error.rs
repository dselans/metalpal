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
}

// Keeping this around as a reminder for how to do this manually
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
