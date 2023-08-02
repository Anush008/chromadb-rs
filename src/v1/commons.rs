pub type Result<T> = anyhow::Result<T>;

type Embedding = Vec<f32>;

type Embeddings = Vec<Embedding>;

type Documents = Vec<String>;