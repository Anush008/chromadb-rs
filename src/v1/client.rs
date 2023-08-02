use super::api::APIClientV1;
use super::commons::ChromaAPIError;

use serde::Deserialize;

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
        let endpoint = options.url.unwrap_or(DEFAULT_ENDPOINT.into());

        ChromaClient {
            api: APIClientV1::new(endpoint),
        }
    }

    /// Resets the database. This will delete all collections and entries.
    pub async fn reset(&self) -> Result<bool, ChromaAPIError> {
        let respones = self.api.post("/reset", None).await?;
        let result = respones.json::<bool>().await.map_err(ChromaAPIError::error)?;
        Ok(result)
    }

    /// The version of Chroma
    pub async fn version(&self) -> Result<String, ChromaAPIError> {
        let response = self.api.get("/version").await?;
        let version = response.json::<String>().await.map_err(ChromaAPIError::error)?;
        Ok(version)
    }

    /// Get the current time in nanoseconds since epoch. Used to check if the server is alive.
    pub async fn heartbeat(&self) -> Result<u64, ChromaAPIError> {
        let response = self.api.get("/heartbeat").await?;
        let json = response.json::<HeartbeatResponse>().await.map_err(ChromaAPIError::error)?;
        Ok(json.heartbeat)
    }

    pub fn create_collection(&self) {
        todo!()
    }

    pub fn get_or_create_collection(&self) {
        todo!()
    }

    pub fn list_collections(&self) {
        todo!()
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
        dbg!(&result);
        assert!(result.is_err_and(|e| e.message.contains("Resetting is not allowed by this configuration")));
    }
}
