use super::api::APIClientV1;
use super::collection::Metadata;
use super::commons::ChromaAPIError;
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
    pub url: String,
}

impl ChromaClient {
    pub fn new(options: ChromaClientOptions) -> ChromaClient {
        let endpoint = if options.url.is_empty() {
            DEFAULT_ENDPOINT.into()
        } else {
            options.url
        };

        ChromaClient {
            api: APIClientV1::new(endpoint.to_string()),
        }
    }

    /// Create a new collection with the given name and metadata.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the collection to create
    /// * `metadata` - Optional metadata to associate with the collection
    /// * `get_or_create` - If true, return the existing collection if it exists
    ///
    /// # Errors
    ///
    /// * `ChromaAPIError` - If the collection already exists and get_or_create is false
    ///                    - If the collection name is invalid
    pub async fn create_collection(
        &self,
        name: &str,
        metadata: Option<Metadata>,
        get_or_create: bool,
    ) -> Result<ChromaCollection, ChromaAPIError> {
        let request_body: Value = json!({
            "name": name,
            "metadata": metadata,
            "get_or_create": get_or_create,
        });
        let response: Response = self.api.post("/collections", Some(request_body)).await?;
        let response: ChromaCollection = response
            .json::<ChromaCollection>()
            .await
            .map_err(ChromaAPIError::error)?;
        let ChromaCollection {
            name, id, metadata, ..
        } = response;
        let collection_metadata: Option<serde_json::Map<String, Value>> =
            metadata.map(|m| m.clone());

        Ok(ChromaCollection::new(
            name,
            id,
            collection_metadata,
            self.api.clone(),
        ))
    }

    /// Get or create a collection with the given name and metadata.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the collection to get or create
    /// * `metadata` - Optional metadata to associate with the collection
    ///
    /// # Errors
    ///
    /// * `ChromaAPIError` - If the collection name is invalid
    pub async fn get_or_create_collection(
        &self,
        name: &str,
        metadata: Option<Metadata>,
    ) -> Result<ChromaCollection, ChromaAPIError> {
        self.create_collection(name, metadata, true).await
    }

    /// List all collections
    pub async fn list_collections(&self) -> Result<Vec<ChromaCollection>, ChromaAPIError> {
        let response = self.api.get("/collections").await?;
        let json = response
            .json::<Vec<ChromaCollection>>()
            .await
            .map_err(ChromaAPIError::error)?;
        Ok(json)
    }

    /// Get a collection with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the collection to get
    ///
    /// # Errors
    ///
    /// * `ChromaAPIError` - If the collection does not exist
    pub async fn get_collection(&self, name: &str) -> Result<ChromaCollection, ChromaAPIError> {
        let response = self.api.get(&format!("/collections/{}", name)).await?;
        let collection = response
            .json::<ChromaCollection>()
            .await
            .map_err(ChromaAPIError::error)?;
        Ok(collection)
    }

    /// Delete a collection with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the collection to delete
    ///
    /// # Errors
    ///
    /// * `ChromaAPIError` - If the collection does not exist
    pub async fn delete_collection(&self, name: &str) -> Result<(), ChromaAPIError> {
        self.api.delete(&format!("/collections/{}", name)).await?;
        Ok(())
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
}

#[derive(Deserialize)]
struct HeartbeatResponse {
    #[serde(rename = "nanosecond heartbeat")]
    pub heartbeat: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_COLLECTION: &str = "8-recipies-for-octopus";

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
    async fn test_create_collection() {
        let client: ChromaClient = ChromaClient::new(Default::default());

        let result = client
            .create_collection(TEST_COLLECTION, None, true)
            .await
            .unwrap();
        assert_eq!(result.name(), TEST_COLLECTION);
    }

    #[tokio::test]
    async fn test_get_collection() {
        let client: ChromaClient = ChromaClient::new(Default::default());

        let collection = client.get_collection(TEST_COLLECTION).await.unwrap();
        assert_eq!(collection.name(), TEST_COLLECTION);
    }

    #[tokio::test]
    async fn test_list_collection() {
        let client: ChromaClient = ChromaClient::new(Default::default());

        let result = client.list_collections().await.unwrap();
        assert!(result.len() > 0);
    }

    #[tokio::test]
    async fn test_delete_collection() {
        let client: ChromaClient = ChromaClient::new(Default::default());

        let collection = client.delete_collection(TEST_COLLECTION).await;
        assert!(collection.is_ok());

        let collection = client.delete_collection(TEST_COLLECTION).await;
        assert!(collection.is_err());
    }
}
