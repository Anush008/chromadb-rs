use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::EmbeddingFunction;
use crate::commons::Embedding;

const OPENAI_EMBEDDINGS_ENDPOINT: &str = "https://api.openai.com/v1/embeddings";
const OPENAI_EMBEDDINGS_MODEL: &str = "text-embedding-3-small";

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    pub embedding: Vec<f32>,
}

#[derive(Debug, Serialize)]
struct EmbeddingRequest<'a> {
    pub model: &'a str,
    pub input: &'a str,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    pub data: Vec<EmbeddingData>,
}

/// Represents the OpenAI Embeddings provider
pub struct OpenAIEmbeddings {
    config: OpenAIConfig,
}

/// Defaults to the "text-embedding-3-small" model
/// The API key can be set in the OPENAI_API_KEY environment variable
pub struct OpenAIConfig {
    pub api_endpoint: String,
    pub api_key: String,
    pub model: String,
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            api_endpoint: OPENAI_EMBEDDINGS_ENDPOINT.to_string(),
            api_key: std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY env is not set"),
            model: OPENAI_EMBEDDINGS_MODEL.to_string(),
        }
    }
}

impl OpenAIEmbeddings {
    pub fn new(config: OpenAIConfig) -> Self {
        Self { config }
    }

    async fn post<T: Serialize>(&self, json_body: T) -> anyhow::Result<Value> {
        let client = reqwest::Client::new();
        let res = client
            .post(&self.config.api_endpoint)
            .body("the exact body that is sent")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&json_body)
            .send()
            .await?;

        match res.error_for_status() {
            Ok(res) => Ok(res.json().await?),
            Err(e) => Err(e.into()),
        }
    }
}

#[async_trait]
impl EmbeddingFunction for OpenAIEmbeddings {
    async fn embed(&self, docs: &[&str]) -> anyhow::Result<Vec<Embedding>> {
        let mut embeddings = Vec::new();
        for doc in docs {
            let req = EmbeddingRequest {
                model: &self.config.model,
                input: doc,
            };
            let res = self.post(req).await?;
            let body = serde_json::from_value::<EmbeddingResponse>(res)?;
            embeddings.push(body.data[0].embedding.clone());
        }

        Ok(embeddings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::CollectionEntries;
    use crate::ChromaClient;

    #[tokio::test]
    async fn test_openai_embeddings() {
        let client = ChromaClient::new(Default::default());
        let collection = client
            .await
            .unwrap()
            .get_or_create_collection("open-ai-test-collection", None)
            .await
            .unwrap();
        let openai_embeddings = OpenAIEmbeddings::new(Default::default());

        let docs = vec![
            "Once upon a time there was a frog",
            "Once upon a time there was a cow",
            "Once upon a time there was a wolverine",
        ];

        let collection_entries = CollectionEntries {
            ids: vec!["test1", "test2", "test3"],
            metadatas: None,
            documents: Some(docs),
            embeddings: None,
        };

        collection
            .upsert(collection_entries, Some(Box::new(openai_embeddings)))
            .await
            .unwrap();
    }
}
