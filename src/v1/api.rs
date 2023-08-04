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
            api_endpoint: endpoint + "/api/v1",
        }
    }

    pub fn post(&self, path: &str, json_body: Option<Value>) -> Result<Response> {
        let url = format!(
            "{api_endpoint}{path}",
            api_endpoint = self.api_endpoint,
            path = path
        );

        let res = minreq::post(&url)
            .with_header("Content-Type", "application/json")
            .with_json(&json_body)?
            .send()?;

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

    pub fn get(&self, path: &str) -> Result<Response> {
        let url = format!(
            "{api_endpoint}{path}",
            api_endpoint = self.api_endpoint,
            path = path
        );
        let res = minreq::get(&url).send()?;
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

    pub fn put(&self, path: &str, json_body: Option<Value>) -> Result<Response> {
        let url = format!(
            "{api_endpoint}{path}",
            api_endpoint = self.api_endpoint,
            path = path
        );

        let json_body = match json_body {
            Some(json_body) => serde_json::to_value(json_body).unwrap(),
            None => Value::Null,
        };

        let res = minreq::put(&url)
            .with_header("Content-Type", "application/json")
            .with_json(&json_body)?
            .send()?;
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

    pub fn delete(&self, path: &str) -> Result<Response> {
        let url = format!(
            "{api_endpoint}{path}",
            api_endpoint = self.api_endpoint,
            path = path
        );
        let res = minreq::delete(&url).send()?;
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
