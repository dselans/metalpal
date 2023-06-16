use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Error: {0}")]
    GenericError(String),
    // #[error("This is a generic error")]
    // GenericError,
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
    // #[error("Didn't get a file name")]
    // MissingFilename,
    // #[error("Could not load config")]
    // ConfigLoad {
    //     #[from]
    //     source: io::Error,
    // },
}

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
