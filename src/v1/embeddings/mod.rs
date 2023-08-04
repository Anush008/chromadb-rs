pub trait EmbeddingFunction {
    fn embed(&self, docs: &Vec<String>) -> Vec<Vec<f64>>;
}

#[derive(Clone)]
pub(super) struct MockEmbeddingProvider;

impl EmbeddingFunction for MockEmbeddingProvider {
    fn embed(&self, docs: &Vec<String>) -> Vec<Vec<f64>> {
        docs.iter().map(|_| vec![0.0_f64; 768]).collect()
    }
}
