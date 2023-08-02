use super::api::APIClientV1;
use super::collection::Metadata;
use super::commons::ChromaAPIError;
use super::embeddings::EmbeddingFunction;
use super::ChromaCollection;

use reqwest::Response;
use serde::Deserialize;
use serde_json::{json, Value};

const DEFAULT_ENDPOINT: &str = "http://localhost:8000";
pub struct ChromaClient {
    api: APIClientV1,
}

#[derive(Default)]
pub struct ChromaClientOptions {
    pub url: Option<String>,
}

impl ChromaClient {
    pub fn new(options: ChromaClientOptions) -> ChromaClient {
        let endpoint: String = options.url.unwrap_or(DEFAULT_ENDPOINT.into());

        ChromaClient {
            api: APIClientV1::new(endpoint),
        }
    }

    /// Resets the database. This will delete all collections and entries.
    pub async fn reset(&self) -> Result<bool, ChromaAPIError> {
        let respones: Response = self.api.post("/reset", None).await?;
        let result: bool = respones
            .json::<bool>()
            .await
            .map_err(ChromaAPIError::error)?;
        Ok(result)
    }

    /// The version of Chroma
    pub async fn version(&self) -> Result<String, ChromaAPIError> {
        let response: Response = self.api.get("/version").await?;
        let version: String = response
            .json::<String>()
            .await
            .map_err(ChromaAPIError::error)?;
        Ok(version)
    }

    /// Get the current time in nanoseconds since epoch. Used to check if the server is alive.
    pub async fn heartbeat(&self) -> Result<u64, ChromaAPIError> {
        let response: Response = self.api.get("/heartbeat").await?;
        let json: HeartbeatResponse = response
            .json::<HeartbeatResponse>()
            .await
            .map_err(ChromaAPIError::error)?;
        Ok(json.heartbeat)
    }

    /// Create a new collection with the given name and metadata.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the collection to create
    /// * `metadata` - Optional metadata to associate with the collection
    /// * `embedding_function` - Optional function to use to embed documents. Uses this as the default embedding function for the collection
    /// * `get_or_create` - If true, return the existing collection if it exists
    ///
    /// # Errors
    ///
    /// * `ChromaAPIError` - If the collection already exists and get_or_create is False
    ///                    - If the collection name is invalid
    pub async fn create_collection(
        &self,
        name: &str,
        metadata: Option<Metadata>,
        embedding_function: Option<Box<EmbeddingFunction>>,
        get_or_create: bool,
    ) -> Result<ChromaCollection, ChromaAPIError> {
        let request_body: Value = json!({
            "name": name,
            "metadata": metadata,
            "get_or_create": get_or_create,
        });
        let response: Response = self
            .api
            .post("/collections", Some(request_body))
            .await?;
        let response: CollectionResponse = response
            .json::<CollectionResponse>()
            .await
            .map_err(ChromaAPIError::error)?;
        let CollectionResponse { name, id, metadata } = response;
        let collection_metadata: Option<serde_json::Map<String, Value>> =
            metadata.map(|m| m.clone());

        Ok(ChromaCollection::new(
            name,
            id,
            collection_metadata,
            self.api.clone(),
            embedding_function,
        ))
    }

    pub async fn get_or_create_collection(
        &self,
        name: &str,
        metadata: Option<Metadata>,
        embedding_function: Option<Box<EmbeddingFunction>>,
    ) -> Result<ChromaCollection, ChromaAPIError> {
        self.create_collection(name, metadata, embedding_function, true)
            .await
    }

    /// List all collections
    pub async fn list_collections(&self) -> Result<Vec<CollectionResponse>, ChromaAPIError> {
        let response = self.api.get("/collections").await?;
        let json = response
            .json::<Vec<CollectionResponse>>()
            .await
            .map_err(ChromaAPIError::error)?;
        Ok(json)
    }

    pub fn get_collection(&self) {
        todo!()
    }

    pub fn delete_collection(&self) {
        todo!()
    }
}

#[derive(Deserialize)]
struct HeartbeatResponse {
    #[serde(rename = "nanosecond heartbeat")]
    pub heartbeat: u64,
}

#[derive(Deserialize)]
pub struct CollectionResponse {
    pub name: String,
    pub id: String,
    pub metadata: Option<serde_json::Map<String, Value>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_heartbeat() {
        let client: ChromaClient = ChromaClient::new(Default::default());

        let heartbeat = client.heartbeat().await.unwrap();
        assert!(heartbeat > 0);
    }

    #[tokio::test]
    async fn test_version() {
        let client: ChromaClient = ChromaClient::new(Default::default());

        let version = client.version().await.unwrap();
        assert_eq!(version.split('.').count(), 3);
    }

    #[tokio::test]
    async fn test_reset() {
        let client: ChromaClient = ChromaClient::new(Default::default());

        let result = client.reset().await;
        assert!(result.is_err_and(|e| e
            .message
            .contains("Resetting is not allowed by this configuration")));
    }

    #[tokio::test]
    async fn test_list_collection_empty() {
        let client: ChromaClient = ChromaClient::new(Default::default());

        let result = client.list_collections().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_collection() {
        let client: ChromaClient = ChromaClient::new(Default::default());

        let result = client
            .create_collection("9-recipies-for-Octopus", None, None, true)
            .await
            .unwrap();
        assert_eq!(result.name(), "9-recipies-for-Octopus");
    }

    #[tokio::test]
    async fn test_list_collection() {
        let client: ChromaClient = ChromaClient::new(Default::default());

        let result = client.list_collections().await.unwrap();
        assert!(result.len() > 0);
    }
}
