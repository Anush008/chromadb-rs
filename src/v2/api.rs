use super::commons::Result;
use base64::prelude::*;
use reqwest::{Client, Method, Response};
use serde_json::Value;

#[derive(Clone, Debug)]
pub enum ChromaTokenHeader {
    Authorization,
    XChromaToken,
}

#[derive(Clone, Debug)]
pub enum ChromaAuthMethod {
    None,
    BasicAuth {
        username: String,
        password: String,
    },
    TokenAuth {
        token: String,
        header: ChromaTokenHeader,
    },
}

impl Default for ChromaAuthMethod {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Default, Debug)]
pub(super) struct APIClientAsync {
    pub(super) api_endpoint: String,
    pub(super) auth_method: ChromaAuthMethod,
    pub(super) database: Option<String>,
    client: Client,
}

impl APIClientAsync {
    pub fn new(endpoint: String, auth_method: ChromaAuthMethod, database: Option<String>) -> Self {
        Self {
            api_endpoint: format!("{}/api/v1", endpoint),
            auth_method,
            database,
            client: Client::new(),
        }
    }

    pub async fn post(&self, path: &str, json_body: Option<Value>) -> Result<Response> {
        self.send_request(Method::POST, path, json_body).await
    }

    pub async fn get(&self, path: &str) -> Result<Response> {
        self.send_request(Method::GET, path, None).await
    }

    pub async fn put(&self, path: &str, json_body: Option<Value>) -> Result<Response> {
        self.send_request(Method::PUT, path, json_body).await
    }

    pub async fn delete(&self, path: &str) -> Result<Response> {
        self.send_request(Method::DELETE, path, None).await
    }

    async fn send_request(
        &self,
        method: Method,
        path: &str,
        json_body: Option<Value>,
    ) -> Result<Response> {
        let url = if let Some(database) = &self.database {
            format!("{}{}?database={}", self.api_endpoint, path, database)
        } else {
            format!("{}{}", self.api_endpoint, path,)
        };

        let mut request = self.client.request(method, &url);

        // Add auth headers if needed
        match &self.auth_method {
            ChromaAuthMethod::None => {}
            ChromaAuthMethod::BasicAuth { username, password } => {
                let credentials = BASE64_STANDARD.encode(format!("{username}:{password}"));
                request = request.header("Authorization", format!("Basic {credentials}"));
            }
            ChromaAuthMethod::TokenAuth { token, header } => match header {
                ChromaTokenHeader::Authorization => {
                    request = request.header("Authorization", format!("Bearer {token}"));
                }
                ChromaTokenHeader::XChromaToken => {
                    request = request.header("X-Chroma-Token", token);
                }
            },
        }

        // Add JSON body if present
        if let Some(body) = json_body {
            request = request
                .header("Content-Type", "application/json")
                .json(&body);
        }

        let response = request.send().await?;
        let status = response.status();

        if status.is_success() {
            Ok(response)
        } else {
            let error_text = response.text().await?;
            anyhow::bail!(
                "{} {}: {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown"),
                error_text
            )
        }
    }
}
