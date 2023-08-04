use async_trait::async_trait;

#[async_trait]
pub trait EmbeddingFunction {
    async fn embed(&self, docs: &Vec<String>) -> Vec<Vec<f64>>;
}

#[derive(Clone)]
pub(super) struct MockEmbeddingProvider;

#[async_trait]
impl EmbeddingFunction for MockEmbeddingProvider {
    async fn embed(&self, docs: &Vec<String>) -> Vec<Vec<f64>> {
        docs.iter().map(|_| vec![0.0_f64; 768]).collect()
    }
}
