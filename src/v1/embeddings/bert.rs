use super::EmbeddingFunction;
pub use rust_bert::pipelines::sentence_embeddings::*;

impl EmbeddingFunction for SentenceEmbeddingsModel {
    fn embed(&self, docs: &[String]) -> Vec<Vec<f32>> {
        self.encode(docs).unwrap()
    }
}
