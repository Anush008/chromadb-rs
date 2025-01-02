use std::sync::Arc;

pub use super::api::{ChromaAuthMethod, ChromaTokenHeader};
use super::{
    api::APIClientAsync,
    commons::{Metadata, Result},
    ChromaCollection,
};

use serde::Deserialize;
use serde_json::json;

const DEFAULT_ENDPOINT: &str = "http://localhost:8000";

// A client representation for interacting with ChromaDB.
pub struct ChromaClient {
    api: Arc<APIClientAsync>,
}

/// The options for instantiating ChromaClient.
#[derive(Debug)]
pub struct ChromaClientOptions {
    /// The URL of the Chroma Server.
    pub url: Option<String>,
    /// Authentication to use to connect to the Chroma Server.
    pub auth: ChromaAuthMethod,
    /// Database to use for the client.  Must be a valid database and match the authorization.
    pub database: String,
}

impl Default for ChromaClientOptions {
    fn default() -> Self {
        Self {
            url: None,
            auth: ChromaAuthMethod::None,
            database: "default_database".to_string(),
        }
    }
}

impl ChromaClient {
    /// Create a new Chroma client with the given options.
    /// * Defaults to `url`: http://localhost:8000
    pub async fn new(
        ChromaClientOptions {
            url,
            auth,
            database,
        }: ChromaClientOptions,
    ) -> Result<ChromaClient> {
        let endpoint = if let Some(url) = url {
            url
        } else {
            std::env::var("CHROMA_HOST")
                .unwrap_or(std::env::var("CHROMA_URL").unwrap_or(DEFAULT_ENDPOINT.to_string()))
        };
        let user_identity = APIClientAsync::get_auth(&endpoint, &auth).await?;
        Ok(ChromaClient {
            api: Arc::new(APIClientAsync::new(
                endpoint,
                auth,
                user_identity.tenant,
                database,
            )),
        })
    }

    /// Create a new collection with the given name and metadata.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the collection to create
    /// * `metadata` - Optional metadata to associate with the collection. Must be a JSON object with keys and values that are either numbers, strings or floats.
    /// * `get_or_create` - If true, return the existing collection if it exists
    ///
    /// # Errors
    ///
    /// * If the collection already exists and get_or_create is false
    /// * If the collection name is invalid
    pub async fn create_collection(
        &self,
        name: &str,
        metadata: Option<Metadata>,
        get_or_create: bool,
    ) -> Result<ChromaCollection> {
        let request_body = json!({
            "name": name,
            "metadata": metadata,
            "get_or_create": get_or_create,
        });
        let response = self
            .api
            .post_database("/collections", Some(request_body))
            .await?;
        let mut collection = response.json::<ChromaCollection>().await?;
        collection.api = self.api.clone();
        Ok(collection)
    }

    /// Get or create a collection with the given name and metadata.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the collection to get or create
    /// * `metadata` - Optional metadata to associate with the collection. Must be a JSON object with keys and values that are either numbers, strings or floats.
    ///
    /// # Errors
    ///
    /// * If the collection name is invalid
    pub async fn get_or_create_collection(
        &self,
        name: &str,
        metadata: Option<Metadata>,
    ) -> Result<ChromaCollection> {
        self.create_collection(name, metadata, true).await
    }

    /// List all collections
    pub async fn list_collections(&self) -> Result<Vec<ChromaCollection>> {
        let response = self.api.get_database("/collections").await?;
        let collections = response.json::<Vec<ChromaCollection>>().await?;
        let collections = collections
            .into_iter()
            .map(|mut collection| {
                collection.api = self.api.clone();
                collection
            })
            .collect();
        Ok(collections)
    }

    /// Get a collection with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the collection to get
    ///
    /// # Errors
    ///
    /// * If the collection name is invalid
    /// * If the collection does not exist
    pub async fn get_collection(&self, name: &str) -> Result<ChromaCollection> {
        let response = self
            .api
            .get_database(&format!("/collections/{}", name))
            .await?;
        let mut collection = response.json::<ChromaCollection>().await?;
        collection.api = self.api.clone();
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
    /// * If the collection name is invalid
    /// * If the collection does not exist
    pub async fn delete_collection(&self, name: &str) -> Result<()> {
        self.api
            .delete_database(&format!("/collections/{}", name))
            .await?;
        Ok(())
    }

    /// The version of Chroma
    pub async fn version(&self) -> Result<String> {
        let response = self.api.get_v1("/version").await?;
        let version = response.json::<String>().await?;
        Ok(version)
    }

    /// Get the current time in nanoseconds since epoch. Used to check if the server is alive.
    pub async fn heartbeat(&self) -> Result<u64> {
        let response = self.api.get_v1("/heartbeat").await?;
        let json = response.json::<HeartbeatResponse>().await?;
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
    use tokio;

    const TEST_COLLECTION: &str = "8-recipies-for-octopus";

    #[tokio::test]
    async fn test_heartbeat() {
        let client: ChromaClient = ChromaClient::new(Default::default()).await.unwrap();

        let heartbeat = client.heartbeat().await.unwrap();
        assert!(heartbeat > 0);
    }

    #[tokio::test]
    async fn test_version() {
        let client: ChromaClient = ChromaClient::new(Default::default()).await.unwrap();

        let version = client.version().await.unwrap();
        assert_eq!(version.split('.').count(), 3);
    }

    #[tokio::test]
    async fn test_create_collection() {
        let client: ChromaClient = ChromaClient::new(Default::default()).await.unwrap();

        let result = client
            .create_collection(TEST_COLLECTION, None, true)
            .await
            .unwrap();
        assert_eq!(result.name(), TEST_COLLECTION);
    }

    #[tokio::test]
    async fn test_get_collection() {
        let client: ChromaClient = ChromaClient::new(Default::default()).await.unwrap();

        const GET_TEST_COLLECTION: &str = "100-recipes-for-octopus";

        client
            .create_collection(GET_TEST_COLLECTION, None, true)
            .await
            .unwrap();

        let collection = client.get_collection(GET_TEST_COLLECTION).await.unwrap();
        assert_eq!(collection.name(), GET_TEST_COLLECTION);
        assert!(collection.configuration_json.is_some());
    }

    #[tokio::test]
    async fn test_list_collection() {
        let client: ChromaClient = ChromaClient::new(Default::default()).await.unwrap();

        let result = client.list_collections().await.unwrap();
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_delete_collection() {
        let client: ChromaClient = ChromaClient::new(Default::default()).await.unwrap();

        const DELETE_TEST_COLLECTION: &str = "6-recipies-for-octopus";
        client
            .get_or_create_collection(DELETE_TEST_COLLECTION, None)
            .await
            .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let collection = client.delete_collection(DELETE_TEST_COLLECTION).await;
        assert!(collection.is_ok());

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let collection = client.delete_collection(DELETE_TEST_COLLECTION).await;
        assert!(collection.is_err());
    }
}
