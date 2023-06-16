use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    // #[error("This is a generic error")]
    // GenericError,
    #[error("This is a JSON error")]
    JSONError {
        #[from]
        source: serde_json::Error,
    },
    #[error("This is an IO error: {source}")]
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
