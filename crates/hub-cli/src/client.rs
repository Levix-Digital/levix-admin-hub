//! The HTTP side: one Hub address, its trust settings and an optional bearer token.

use std::error::Error as _;
use std::path::PathBuf;

use reqwest::{Method, StatusCode};
use serde_json::Value;

use crate::error::{CliError, CliResult};
use crate::tls;

/// Where the Hub is and how its certificate is trusted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub url: String,
    pub fingerprint: Option<String>,
    pub ca_file: Option<PathBuf>,
}

impl Target {
    pub fn new(
        url: &str,
        fingerprint: Option<String>,
        ca_file: Option<PathBuf>,
    ) -> CliResult<Self> {
        let url = url.trim().trim_end_matches('/').to_owned();
        if !(url.starts_with("https://") || url.starts_with("http://")) {
            return Err(CliError::Usage(format!(
                "the Hub address must start with https:// (got {url})"
            )));
        }
        let fingerprint = fingerprint
            .map(|f| tls::normalize_fingerprint(&f))
            .transpose()?;
        Ok(Self {
            url,
            fingerprint,
            ca_file,
        })
    }
}

pub struct Client {
    http: reqwest::Client,
    target: Target,
    token: Option<String>,
}

impl Client {
    pub fn new(target: Target, token: Option<String>) -> CliResult<Self> {
        let config = tls::client_config(target.fingerprint.as_deref(), target.ca_file.as_deref())?;
        let http = reqwest::Client::builder()
            .use_preconfigured_tls(config)
            .user_agent(concat!("levix-hub-cli/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| CliError::Local(format!("building the HTTP client: {e}")))?;
        Ok(Self {
            http,
            target,
            token,
        })
    }

    pub fn target(&self) -> &Target {
        &self.target
    }

    pub fn has_token(&self) -> bool {
        self.token.is_some()
    }

    pub async fn get(&self, path: &str, query: &[(&str, String)]) -> CliResult<Value> {
        self.send(Method::GET, path, query, None).await
    }

    pub async fn post(&self, path: &str, body: Option<&Value>) -> CliResult<Value> {
        self.send(Method::POST, path, &[], body).await
    }

    pub async fn put(&self, path: &str, body: &Value) -> CliResult<Value> {
        self.send(Method::PUT, path, &[], Some(body)).await
    }

    pub async fn patch(&self, path: &str, body: &Value) -> CliResult<Value> {
        self.send(Method::PATCH, path, &[], Some(body)).await
    }

    pub async fn delete(&self, path: &str) -> CliResult<Value> {
        self.send(Method::DELETE, path, &[], None).await
    }

    /// Sends one request; a success returns the JSON body (`null` when there is none), a
    /// failure the Hub's error object.
    pub async fn send(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, String)],
        body: Option<&Value>,
    ) -> CliResult<Value> {
        let mut request = self
            .http
            .request(method, format!("{}{path}", self.target.url));
        if !query.is_empty() {
            request = request.query(query);
        }
        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request.send().await.map_err(|e| self.connection(&e))?;
        let status = response.status();
        let text = response.text().await.map_err(|e| self.connection(&e))?;
        if status.is_success() {
            if text.trim().is_empty() {
                return Ok(Value::Null);
            }
            return serde_json::from_str(&text).map_err(|e| {
                CliError::Local(format!("the Hub answered something that is not JSON: {e}"))
            });
        }
        Err(api_error(status, &text))
    }

    fn connection(&self, error: &reqwest::Error) -> CliError {
        let mut message = format!("cannot reach the Hub at {}", self.target.url);
        let mut source = error.source();
        while let Some(cause) = source {
            message.push_str(": ");
            message.push_str(&cause.to_string());
            source = cause.source();
        }
        CliError::Connection(message)
    }
}

fn api_error(status: StatusCode, text: &str) -> CliError {
    let parsed: Option<Value> = serde_json::from_str(text).ok();
    let detail = parsed.as_ref().and_then(|v| v.get("error"));
    let field = |name: &str| {
        detail
            .and_then(|d| d.get(name))
            .and_then(Value::as_str)
            .map(str::to_owned)
    };
    CliError::Api {
        status: status.as_u16(),
        code: field("code").unwrap_or_else(|| format!("http_{}", status.as_u16())),
        message: field("message").unwrap_or_else(|| {
            status
                .canonical_reason()
                .unwrap_or("request failed")
                .to_owned()
        }),
    }
}
