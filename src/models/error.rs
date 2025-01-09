// TODO: Wire in

use std::fmt;

#[derive(Debug)]
pub enum Error {
    ParserError(String),
    TokenFilterError(String),
    // CoverageAnalysisError(String),
    // ConfidenceAnalysisError(String),
    // MissingDataError(String),
    // IoError(std::io::Error),
    // Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ParserError(msg) => write!(f, "Parser Error: {}", msg),
            Error::TokenFilterError(msg) => write!(f, "Token Filter Error: {}", msg),
            // Error::CoverageAnalysisError(msg) => write!(f, "Coverage Analysis Error: {}", msg),
            // Error::ConfidenceAnalysisError(msg) => {
            //     write!(f, "Confidence Analysis Error: {}", msg)
            // }
            // Error::MissingDataError(msg) => write!(f, "Missing Data Error: {}", msg),
            // Error::IoError(err) => write!(f, "IO Error: {}", err),
            // Error::Other(msg) => write!(f, "Other Error: {}", msg),
        }
    }
}

// impl From<std::io::Error> for Error {
//     fn from(err: std::io::Error) -> Error {
//         Error::IoError(err)
//     }
// }
