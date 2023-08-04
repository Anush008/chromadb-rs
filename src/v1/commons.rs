use std::fmt::{self, Debug};
use std::{error::Error, fmt::Display};

use serde_json::{Map, Value};

#[derive(Debug)]
pub struct ChromaAPIError {
    pub message: String,
}

impl ChromaAPIError {
    pub fn error<E: Display>(e: E) -> ChromaAPIError {
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

pub type Metadata = Map<String, Value>;

pub type Metadatas = Vec<Metadata>;

pub type Embedding = Vec<f64>;

pub type Embeddings = Vec<Embedding>;

pub type Documents = Vec<String>;
