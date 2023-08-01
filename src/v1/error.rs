use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ChromaAPIError {
    pub message: String,
}

impl fmt::Display for ChromaAPIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "APIError: {}", self.message)
    }
}

impl Error for ChromaAPIError {}