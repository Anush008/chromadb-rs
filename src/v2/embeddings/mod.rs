use super::commons::Embedding;
use anyhow::Result;
use async_trait::async_trait;

#[cfg(feature = "openai")]
pub mod openai;

#[async_trait]
pub trait EmbeddingFunction {
    async fn embed(&self, docs: &[&str]) -> Result<Vec<Embedding>>;
}

#[derive(Clone)]
pub struct MockEmbeddingProvider;

#[async_trait]
impl EmbeddingFunction for MockEmbeddingProvider {
    async fn embed(&self, docs: &[&str]) -> Result<Vec<Embedding>> {
        Ok(docs.iter().map(|_| vec![0.0_f32; 768]).collect())
    }
}

