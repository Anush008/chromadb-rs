use async_trait::async_trait;

#[async_trait]
pub trait EmbeddingFunction {
    async fn embed(&self, docs: &Vec<String>) -> Vec<Vec<f64>>;
}

#[derive(Clone)]
pub struct MockEmbeddingProvider;

#[async_trait]
impl EmbeddingFunction for MockEmbeddingProvider {
    async fn embed(&self, docs: &Vec<String>) -> Vec<Vec<f64>> {
        docs.iter().map(|_| vec![0.0; 768]).collect()
    }
}
