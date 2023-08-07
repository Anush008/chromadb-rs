use super::commons::Embedding;
use anyhow::Result;

#[cfg(feature = "bert")]
pub mod bert;

#[cfg(feature = "openai")]
pub mod openai;

pub trait EmbeddingFunction {
    fn embed(&self, docs: &[&str]) -> Result<Vec<Embedding>>;
}

#[derive(Clone)]
pub(super) struct MockEmbeddingProvider;

impl EmbeddingFunction for MockEmbeddingProvider {
    fn embed(&self, docs: &[&str]) -> Result<Vec<Embedding>> {
        Ok(docs.iter().map(|_| vec![0.0_f32; 768]).collect())
    }
}

