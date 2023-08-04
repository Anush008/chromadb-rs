use serde_json::{Map, Value};

pub(super) type Result<T> = anyhow::Result<T>;
pub(super) type Metadata = Map<String, Value>;
pub(super) type Metadatas = Vec<Metadata>;
pub(super) type Embedding = Vec<f64>;
pub(super) type Embeddings = Vec<Embedding>;
pub(super) type Documents = Vec<String>;
