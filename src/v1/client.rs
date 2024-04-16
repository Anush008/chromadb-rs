use std::sync::Arc;

pub use super::api::{ChromaAuthMethod, ChromaTokenHeader};
use super::{
    api::APIClientV1,
    commons::{Metadata, Result},
    ChromaCollection,
};

use serde::Deserialize;
use serde_json::json;

const DEFAULT_ENDPOINT: &str = "http://localhost:8000";

// A client representation for interacting with ChromaDB.
pub struct ChromaClient {
    api: Arc<APIClientV1>,
}

/// The options for instantiating ChromaClient.
#[derive(Debug, Default)]
pub struct ChromaClientOptions {
    pub url: String,
    pub auth: ChromaAuthMethod,
}

impl ChromaClient {
    /// Create a new Chroma client with the given options.
    /// * Defaults to `url`: http://localhost:8000
    pub fn new(ChromaClientOptions { url, auth }: ChromaClientOptions) -> ChromaClient {
        let endpoint = if url.is_empty() {
            std::env::var("CHROMA_URL").unwrap_or(DEFAULT_ENDPOINT.to_string())
        } else {
            url
        };

        ChromaClient {
            api: Arc::new(APIClientV1::new(endpoint, auth)),
        }
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
    pub fn create_collection(
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
        let response = self.api.post("/collections", Some(request_body))?;
        let mut collection = response.json::<ChromaCollection>()?;
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
    pub fn get_or_create_collection(
        &self,
        name: &str,
        metadata: Option<Metadata>,
    ) -> Result<ChromaCollection> {
        self.create_collection(name, metadata, true)
    }

    /// List all collections
    pub fn list_collections(&self) -> Result<Vec<ChromaCollection>> {
        let response = self.api.get("/collections")?;
        let collections = response.json::<Vec<ChromaCollection>>()?;
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
    pub fn get_collection(&self, name: &str) -> Result<ChromaCollection> {
        let response = self.api.get(&format!("/collections/{}", name))?;
        let mut collection = response.json::<ChromaCollection>()?;
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
    pub fn delete_collection(&self, name: &str) -> Result<()> {
        self.api.delete(&format!("/collections/{}", name))?;
        Ok(())
    }

    /// Resets the database. This will delete all collections and entries.
    pub fn reset(&self) -> Result<bool> {
        let respones = self.api.post("/reset", None)?;
        let result = respones.json::<bool>()?;
        Ok(result)
    }

    /// The version of Chroma
    pub fn version(&self) -> Result<String> {
        let response = self.api.get("/version")?;
        let version = response.json::<String>()?;
        Ok(version)
    }

    /// Get the current time in nanoseconds since epoch. Used to check if the server is alive.
    pub fn heartbeat(&self) -> Result<u64> {
        let response = self.api.get("/heartbeat")?;
        let json = response.json::<HeartbeatResponse>()?;
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

    #[test]
    fn test_heartbeat() {
        let client: ChromaClient = ChromaClient::new(Default::default());

        let heartbeat = client.heartbeat().unwrap();
        assert!(heartbeat > 0);
    }

    #[test]
    fn test_version() {
        let client: ChromaClient = ChromaClient::new(Default::default());

        let version = client.version().unwrap();
        assert_eq!(version.split('.').count(), 3);
    }

    #[test]
    fn test_reset() {
        let client: ChromaClient = ChromaClient::new(Default::default());

        let result = client.reset();
        assert!(result.is_err_and(|e| e
            .to_string()
            .contains("Resetting is not allowed by this configuration")));
    }

    #[test]
    fn test_create_collection() {
        let client: ChromaClient = ChromaClient::new(Default::default());

        let result = client
            .create_collection(TEST_COLLECTION, None, true)
            .unwrap();
        assert_eq!(result.name(), TEST_COLLECTION);
    }

    #[test]
    fn test_get_collection() {
        let client: ChromaClient = ChromaClient::new(Default::default());

        let collection = client.get_collection(TEST_COLLECTION).unwrap();
        assert_eq!(collection.name(), TEST_COLLECTION);
    }

    #[test]
    fn test_list_collection() {
        let client: ChromaClient = ChromaClient::new(Default::default());

        let result = client.list_collections().unwrap();
        assert!(result.len() > 0);
    }

    #[test]
    fn test_delete_collection() {
        let client: ChromaClient = ChromaClient::new(Default::default());

        const DELETE_TEST_COLLECTION: &str = "6-recipies-for-octopus";
        client
            .get_or_create_collection(DELETE_TEST_COLLECTION, None)
            .unwrap();

        let collection = client.delete_collection(DELETE_TEST_COLLECTION);
        assert!(collection.is_ok());

        let collection = client.delete_collection(DELETE_TEST_COLLECTION);
        assert!(collection.is_err());
    }
}
