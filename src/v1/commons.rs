use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ChromaAPIError {
    pub message: String,
}

impl ChromaAPIError {
    pub fn error<E: Error>(e: E) -> ChromaAPIError {
        ChromaAPIError {
            message: e.to_string(),
        }
    }
}
impl fmt::Display for ChromaAPIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ChromaAPIError: {}", self.message)
    }
}

impl Error for ChromaAPIError {}

type _Embedding = Vec<f64>;

type _Embeddings = Vec<_Embedding>;

type _Documents = Vec<String>;
