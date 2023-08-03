use super::commons::ChromaAPIError;
use reqwest::Response;
use serde_json::Value;

#[derive(Clone, Default, Debug)]
pub(super) struct APIClientV1 {
    pub(super) api_endpoint: String,
}

impl APIClientV1 {
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
        let url = format!(
            "{api_endpoint}{path}",
            api_endpoint = self.api_endpoint,
            path = path
        );

        let res = reqwest::Client::new()
            .post(&url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&json_body)
            .send()
            .await;
        match res {
            Ok(res) => match res.status().is_success() {
                true => Ok(res),
                false => {
                    let message = format!("{}: {}", res.status(), res.text().await.unwrap());
                    Err(ChromaAPIError { message })
                }
            },
            Err(e) => Err(ChromaAPIError::error(e)),
        }
    }

    pub async fn get(&self, path: &str) -> Result<Response, ChromaAPIError> {
        let url = format!(
            "{api_endpoint}{path}",
            api_endpoint = self.api_endpoint,
            path = path
        );
        let res = reqwest::get(&url).await;
        match res {
            Ok(res) => match res.status().is_success() {
                true => Ok(res),
                false => Err(ChromaAPIError {
                    message: format!("{}: {}", res.status(), res.text().await.unwrap()),
                }),
            },
            Err(e) => Err(ChromaAPIError::error(e)),
        }
    }

    pub async fn put(
        &self,
        path: &str,
        json_body: Option<Value>,
    ) -> Result<Response, ChromaAPIError> {
        let url = format!(
            "{api_endpoint}{path}",
            api_endpoint = self.api_endpoint,
            path = path
        );

        let json_body = match json_body {
            Some(json_body) => serde_json::to_value(json_body).unwrap(),
            None => Value::Null,
        };

        let res = reqwest::Client::new()
            .put(&url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&json_body)
            .send()
            .await;
        match res {
            Ok(res) => match res.status().is_success() {
                true => Ok(res),
                false => {
                    let message = format!("{}: {}", res.status(), res.text().await.unwrap());
                    Err(ChromaAPIError { message })
                }
            },
            Err(e) => Err(ChromaAPIError::error(e)),
        }
    }

    pub async fn delete(&self, path: &str) -> Result<Response, ChromaAPIError> {
        let url = format!(
            "{api_endpoint}{path}",
            api_endpoint = self.api_endpoint,
            path = path
        );
        let res = reqwest::Client::new().delete(&url).send().await;
        match res {
            Ok(res) => match res.status().is_success() {
                true => Ok(res),
                false => Err(ChromaAPIError {
                    message: format!("{}: {}", res.status(), res.text().await.unwrap()),
                }),
            },
            Err(e) => Err(ChromaAPIError::error(e)),
        }
    }
}
