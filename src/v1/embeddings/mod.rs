use async_trait::async_trait;

#[async_trait]
pub trait IEmbeddingFunction {
    async fn generate(&self, texts: Vec<&str>) -> Vec<Vec<f64>>;
}
