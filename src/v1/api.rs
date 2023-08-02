use reqwest::Response;

use super::error::ChromaAPIError;

pub struct APIClient {
    pub api_endpoint: String,
}

impl APIClient {
    pub async fn post<T: serde::ser::Serialize>(
        &self,
        path: &str,
        params: &T,
    ) -> Result<Response, ChromaAPIError> {
        let client = reqwest::Client::new();
        let url = format!(
            "{api_endpoint}{path}",
            api_endpoint = self.api_endpoint,
            path = path
        );
        let res = client
            .post(&url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&params)
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
        let res = client
            .get(&url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
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

    pub async fn delete(&self, path: &str) -> Result<Response, ChromaAPIError> {
        let client = reqwest::Client::new();
        let url = format!(
            "{api_endpoint}{path}",
            api_endpoint = self.api_endpoint,
            path = path
        );
        let res = client
            .delete(&url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
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

    fn error(&self, err: reqwest::Error) -> ChromaAPIError {
        ChromaAPIError {
            message: err.to_string(),
        }
    }
}
