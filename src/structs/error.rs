use std::fmt;

#[derive(Debug)]
pub enum Error {
    ParserError(String),
    TokenFilterError(String),
    IoError(std::io::Error),
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ParserError(msg) => write!(f, "Parser Error: {}", msg),
            Error::TokenFilterError(msg) => write!(f, "Token Filter Error: {}", msg),
            Error::IoError(err) => write!(f, "IO Error: {}", err),
            Error::Other(msg) => write!(f, "Other Error: {}", msg),
        }
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Error {
        Error::ParserError(msg)
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Error {
        Error::ParserError(msg.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(err: Box<dyn std::error::Error>) -> Error {
        Error::Other(err.to_string())
    }
}
