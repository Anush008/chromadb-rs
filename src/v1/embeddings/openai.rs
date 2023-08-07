use serde::{Deserialize, Serialize};

use super::EmbeddingFunction;
use crate::v1::commons::Embedding;

const OPENAI_EMBEDDINGS_ENDPOINT: &str = "https://api.openai.com/v1/embeddings";
const OPENAI_EMBEDDINGS_MODEL: &str = "text-embedding-ada-002";

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

pub struct OpenAIEmbeddings {
    config: OpenAIConfig,
}

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

    fn post<T: Serialize>(&self, json_body: T) -> anyhow::Result<minreq::Response> {
        let res = minreq::post(&self.config.api_endpoint)
            .with_header("Content-Type", "application/json")
            .with_header("Authorization", format!("Bearer {}", self.config.api_key))
            .with_json(&json_body)?
            .send()?;

        match res.status_code {
            200..=299 => Ok(res),
            _ => anyhow::bail!(
                "{} {}: {}",
                res.status_code,
                res.reason_phrase,
                res.as_str().unwrap()
            ),
        }
    }
}

impl EmbeddingFunction for OpenAIEmbeddings {
    fn embed(&self, docs: &[&str]) -> anyhow::Result<Vec<Embedding>> {
        let mut embeddings = Vec::new();
        docs.iter().for_each(|doc| {
            let req = EmbeddingRequest {
                model: &self.config.model,
                input: &doc,
            };
            let res = self.post(req).unwrap();
            let body = res.json::<EmbeddingResponse>().unwrap();
            embeddings.push(body.data[0].embedding.clone());
        });
        Ok(embeddings)
    }
}

#[cfg(test)]
mod tests {
    use crate::v1::collection::CollectionEntries;
    use super::*;
    use crate::v1::ChromaClient;

    #[test]
    fn test_openai_embeddings() {

        let client = ChromaClient::new(Default::default());
        let collection = client
            .get_or_create_collection("open-ai-test-collection", None)
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
            .unwrap();
    }
}
