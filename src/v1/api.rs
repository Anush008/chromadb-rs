use super::commons::Result;
use minreq::Response;
use serde_json::Value;

#[derive(Clone, Default, Debug)]
pub(super) struct APIClientV1 {
    pub(super) api_endpoint: String,
}

impl APIClientV1 {
    pub fn new(endpoint: String) -> Self {
        Self {
            api_endpoint: format!("{}/api/v1", endpoint),
        }
    }

    pub fn post(&self, path: &str, json_body: Option<Value>) -> Result<Response> {
        self.send_request("POST", path, json_body)
    }

    pub fn get(&self, path: &str) -> Result<Response> {
        self.send_request("GET", path, None)
    }

    pub fn put(&self, path: &str, json_body: Option<Value>) -> Result<Response> {
        self.send_request("PUT", path, json_body)
    }

    pub fn delete(&self, path: &str) -> Result<Response> {
        self.send_request("DELETE", path, None)
    }

    fn send_request(&self, method: &str, path: &str, json_body: Option<Value>) -> Result<Response> {
        let url = format!(
            "{api_endpoint}{path}",
            api_endpoint = self.api_endpoint,
            path = path
        );

        let request = match method {
            "POST" => minreq::post(url),
            "PUT" => minreq::put(url),
            "DELETE" => minreq::delete(url),
            _ => minreq::get(url),
        };

        let request = if let Some(body) = json_body {
            request.with_header("Content-Type", "application/json").with_json(&body)?
        } else {
            request
        };

        let res = request.send()?;

        match res.status_code {
            200..=299 => Ok(res),
            _ => anyhow::bail!(
                "{} {}: {}",
                res.status_code,
                res.reason_phrase,
                res.as_str().unwrap()
            ),
        }
    }
}
