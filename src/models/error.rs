// TODO: Wire in

use std::fmt;

#[derive(Debug)]
pub enum MyError {
    CoverageAnalysisError(String),
    MissingDataError(String),
    IoError(std::io::Error),
    Other(String),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyError::CoverageAnalysisError(msg) => write!(f, "Coverage Analysis Error: {}", msg),
            MyError::MissingDataError(msg) => write!(f, "Missing Data Error: {}", msg),
            MyError::IoError(err) => write!(f, "IO Error: {}", err),
            MyError::Other(msg) => write!(f, "Other Error: {}", msg),
        }
    }
}

impl From<std::io::Error> for MyError {
    fn from(err: std::io::Error) -> MyError {
        MyError::IoError(err)
    }
}
