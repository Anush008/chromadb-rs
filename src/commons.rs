use serde_json::{Map, Value};

pub(super) type Result<T> = anyhow::Result<T>;
pub(super) type ConfigurationJson = Map<String, Value>;
pub(super) type Metadata = Map<String, Value>;
pub(super) type Metadatas = Vec<Metadata>;
pub(super) type Embedding = Vec<f32>;
pub(super) type Embeddings = Vec<Embedding>;
pub(super) type Documents<'a> = Vec<&'a str>;
