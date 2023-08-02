pub type EmbeddingFunction = dyn Fn(Vec<&str>) -> Vec<Vec<f64>>;
