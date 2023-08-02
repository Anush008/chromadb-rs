use super::api::APIClient;

pub struct ChromaClient {
    api: APIClient,
}

#[derive(Default)]
pub struct ChromaClientOptions {
    pub url: Option<String>,
}

const DEFAULT_ENDPOINT: &str = "http://localhost:8000";

impl ChromaClient {
    pub fn new(options: ChromaClientOptions) -> ChromaClient {
        let endpoint = options.url.unwrap_or(DEFAULT_ENDPOINT.into());

        let api = APIClient {
            api_endpoint: endpoint,
        };

        ChromaClient { api }
    }

    pub fn reset(&self) {
        todo!()
    }

    pub fn version(&self) {
        todo!()
    }

    pub fn heartbeat(&self) {
        todo!()
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
