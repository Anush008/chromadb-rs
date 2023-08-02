use reqwest::Response;
use serde_json::Value;

use super::error::ChromaAPIError;

pub struct APIClientV1 {
    pub api_endpoint: String,
}

impl<'a> APIClientV1 {
    pub fn new(endpoint: String) -> Self {
        Self {
            api_endpoint: endpoint + "/api/v1",
        }
    }
    pub async fn post(
        &self,
        path: &str,
        json_body: Option<Value>,
    ) -> Result<Response, ChromaAPIError> {
        let client = reqwest::Client::new();
        let url = format!(
            "{api_endpoint}{path}",
            api_endpoint = self.api_endpoint,
            path = path
        );

        let json_body = match json_body {
            Some(json_body) => serde_json::to_value(json_body).unwrap(),
            None => Value::Null,
        };

        let res = client
            .post(&url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&json_body)
            .send()
            .await;
        match res {
            Ok(res) => match res.status().is_success() {
                true => Ok(res),
                false => Err(ChromaAPIError {
                    message: format!("{}: {}", res.status(), res.text().await.unwrap()),
                }),
            },
            Err(e) => Err(self.error(e)),
        }
    }

    pub async fn get(&self, path: &str) -> Result<Response, ChromaAPIError> {
        let client = reqwest::Client::new();
        let url = format!(
            "{api_endpoint}{path}",
            api_endpoint = self.api_endpoint,
            path = path
        );
        let res = client.get(&url).send().await;
        match res {
            Ok(res) => match res.status().is_success() {
                true => Ok(res),
                false => Err(ChromaAPIError {
                    message: format!("{}: {}", res.status(), res.text().await.unwrap()),
                }),
            },
            Err(e) => Err(self.error(e)),
        }
    }

    pub async fn delete(&self, path: &str) -> Result<Response, ChromaAPIError> {
        let client = reqwest::Client::new();
        let url = format!(
            "{api_endpoint}{path}",
            api_endpoint = self.api_endpoint,
            path = path
        );
        let res = client.delete(&url).send().await;
        match res {
            Ok(res) => match res.status().is_success() {
                true => Ok(res),
                false => Err(ChromaAPIError {
                    message: format!("{}: {}", res.status(), res.text().await.unwrap()),
                }),
            },
            Err(e) => Err(self.error(e)),
        }
    }

    fn error(&self, err: reqwest::Error) -> ChromaAPIError {
        ChromaAPIError {
            message: err.to_string(),
        }
    }
}
