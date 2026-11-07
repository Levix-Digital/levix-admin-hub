//! Calls to the public API. The session is the `HttpOnly` cookie the browser sends; writes
//! carry the CSRF token the Hub returned at sign-in.

use std::cell::RefCell;

use gloo_net::http::{Method, RequestBuilder};
use serde_json::Value;

const CSRF_KEY: &str = "levix-hub.csrf";

/// An error of the API, or of the network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiError {
    pub status: u16,
    pub code: String,
    pub message: String,
}

impl ApiError {
    fn network(message: String) -> Self {
        Self {
            status: 0,
            code: "network".to_owned(),
            message,
        }
    }
}

thread_local! {
    /// Called when an authenticated call answers 401: the session ended on the Hub.
    static ON_SIGNED_OUT: RefCell<Option<Box<dyn Fn()>>> = const { RefCell::new(None) };
}

pub fn on_signed_out(callback: impl Fn() + 'static) {
    ON_SIGNED_OUT.with(|cell| *cell.borrow_mut() = Some(Box::new(callback)));
}

fn storage() -> Option<web_sys::Storage> {
    web_sys::window().and_then(|w| w.local_storage().ok().flatten())
}

pub fn set_csrf(token: Option<&str>) {
    if let Some(storage) = storage() {
        let _ignored = match token {
            Some(token) => storage.set_item(CSRF_KEY, token),
            None => storage.remove_item(CSRF_KEY),
        };
    }
}

fn csrf() -> Option<String> {
    storage().and_then(|s| s.get_item(CSRF_KEY).ok().flatten())
}

/// A stored preference of this browser (language before sign-in).
pub fn preference(key: &str) -> Option<String> {
    storage().and_then(|s| s.get_item(key).ok().flatten())
}

pub fn set_preference(key: &str, value: &str) {
    if let Some(storage) = storage() {
        let _ignored = storage.set_item(key, value);
    }
}

async fn send(
    method: Method,
    path: &str,
    body: Option<&Value>,
    signed_in: bool,
) -> Result<Value, ApiError> {
    let mut builder = RequestBuilder::new(path)
        .method(method.clone())
        .header("accept", "application/json");
    if method != Method::GET {
        if let Some(token) = csrf() {
            builder = builder.header("x-csrf-token", &token);
        }
    }
    let request = match body {
        Some(body) => builder.json(body),
        None => builder.build(),
    }
    .map_err(|e| ApiError::network(e.to_string()))?;
    let response = request
        .send()
        .await
        .map_err(|e| ApiError::network(e.to_string()))?;
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    if (200..300).contains(&status) {
        if text.trim().is_empty() {
            return Ok(Value::Null);
        }
        return serde_json::from_str(&text).map_err(|e| ApiError::network(e.to_string()));
    }
    let parsed: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    let field = |name: &str| {
        parsed
            .pointer(&format!("/error/{name}"))
            .and_then(Value::as_str)
            .map(str::to_owned)
    };
    let error = ApiError {
        status,
        code: field("code").unwrap_or_else(|| format!("http_{status}")),
        message: field("message").unwrap_or_default(),
    };
    if status == 401 && signed_in {
        ON_SIGNED_OUT.with(|cell| {
            if let Some(callback) = cell.borrow().as_ref() {
                callback();
            }
        });
    }
    Err(error)
}

/// A call that needs the session; a 401 signs the console out.
pub async fn get(path: &str) -> Result<Value, ApiError> {
    send(Method::GET, path, None, true).await
}

pub async fn post(path: &str, body: Option<&Value>) -> Result<Value, ApiError> {
    send(Method::POST, path, body, true).await
}

pub async fn put(path: &str, body: &Value) -> Result<Value, ApiError> {
    send(Method::PUT, path, Some(body), true).await
}

pub async fn patch(path: &str, body: &Value) -> Result<Value, ApiError> {
    send(Method::PATCH, path, Some(body), true).await
}

pub async fn delete(path: &str) -> Result<Value, ApiError> {
    send(Method::DELETE, path, None, true).await
}

/// A call made before sign-in (instance status, sign-in, links).
pub async fn public_get(path: &str) -> Result<Value, ApiError> {
    send(Method::GET, path, None, false).await
}

pub async fn public_post(path: &str, body: &Value) -> Result<Value, ApiError> {
    send(Method::POST, path, Some(body), false).await
}

/// Encodes one path segment (ids are UUIDs; this keeps any other text inert).
pub fn segment(value: &str) -> String {
    String::from(js_sys::encode_uri_component(value))
}
