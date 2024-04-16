use super::commons::Result;
use base64::prelude::*;
use minreq::Response;
use serde_json::Value;

/// Which header to send the token if using `ChromaAuthMethod::TokenAuth`.
#[derive(Clone, Debug)]
pub enum ChromaTokenHeader {
    /// Authorization: Bearer
    Authorization,
    /// X-Chroma-Token
    XChromaToken,
}

/// Authentication options, currently only supported in server/client mode.
#[derive(Clone, Debug)]
pub enum ChromaAuthMethod {
    /// No authentication
    None,

    /// Basic authentication: RFC 7617
    BasicAuth { username: String, password: String },

    /// Static token authentication: RFC 6750
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
pub(super) struct APIClientV1 {
    pub(super) api_endpoint: String,
    pub(super) auth_method: ChromaAuthMethod,
}

impl APIClientV1 {
    pub fn new(endpoint: String, auth_method: ChromaAuthMethod) -> Self {
        Self {
            api_endpoint: format!("{}/api/v1", endpoint),
            auth_method,
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
            request
                .with_header("Content-Type", "application/json")
                .with_json(&body)?
        } else {
            request
        };

        let request = match &self.auth_method {
            ChromaAuthMethod::None => request,
            ChromaAuthMethod::BasicAuth { username, password } => {
                let credentials = BASE64_STANDARD.encode(format!("{username}:{password}"));
                request.with_header("Authorization", format!("Basic {credentials}"))
            }
            ChromaAuthMethod::TokenAuth {
                token,
                header: token_header,
            } => match token_header {
                ChromaTokenHeader::Authorization => {
                    request.with_header("Authorization", format!("Bearer {token}"))
                }
                ChromaTokenHeader::XChromaToken => request.with_header("X-Chroma-Token", token),
            },
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
