//! The web console, built by `trunk` into `crates/hub-console/dist` and embedded in the binary.
//! `index.html` is served with a strict Content Security Policy that allows only the console's
//! own files, WebAssembly, and the hashes of the loader scripts trunk inlines.

use std::sync::OnceLock;

use axum::body::Body;
use axum::http::{header, HeaderValue, Method, Uri};
use axum::response::{IntoResponse, Response};
use rust_embed::Embed;
use sha2::{Digest, Sha256};

#[derive(Embed)]
#[folder = "../hub-console/dist"]
#[allow_missing = true]
#[exclude = ".*"]
#[exclude = ".stage/*"]
struct Assets;

const NOT_BUILT: &str = "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
<title>Levix Admin Hub</title></head><body><main><h1>The console is not part of this build</h1>\
<p>Build it with <code>trunk build --release</code> in <code>crates/hub-console</code>, then \
rebuild the server. The API is available under <code>/api/v1</code>.</p></main></body></html>";

/// The policy of every console response.
pub fn content_security_policy(index: &str) -> String {
    let hashes: String = inline_scripts(index)
        .map(|script| {
            let digest = Sha256::digest(script.as_bytes());
            format!(" 'sha256-{}'", data_encoding::BASE64.encode(&digest))
        })
        .collect();
    format!(
        "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'{hashes}; style-src 'self'; \
         img-src 'self' data:; connect-src 'self'; font-src 'self'; object-src 'none'; \
         base-uri 'none'; form-action 'self'; frame-ancestors 'none'"
    )
}

/// The bodies of `<script>` elements without a `src` attribute.
fn inline_scripts(html: &str) -> impl Iterator<Item = &str> {
    html.split("<script").skip(1).filter_map(|rest| {
        let (attributes, body) = rest.split_once('>')?;
        if attributes.contains("src=") {
            return None;
        }
        body.split_once("</script>").map(|(script, _)| script)
    })
}

struct Index {
    html: Vec<u8>,
    policy: HeaderValue,
}

fn index() -> &'static Index {
    static INDEX: OnceLock<Index> = OnceLock::new();
    INDEX.get_or_init(|| {
        let html = Assets::get("index.html")
            .map(|file| file.data.into_owned())
            .unwrap_or_else(|| NOT_BUILT.as_bytes().to_vec());
        let policy = content_security_policy(&String::from_utf8_lossy(&html));
        Index {
            policy: HeaderValue::from_str(&policy)
                .unwrap_or_else(|_| HeaderValue::from_static("default-src 'none'")),
            html,
        }
    })
}

fn with_security_headers(mut response: Response) -> Response {
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_SECURITY_POLICY, index().policy.clone());
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    response
}

/// Serves the console's start page and files; anything else is the API's 404.
pub async fn serve(method: Method, uri: Uri) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return crate::error::not_found().await.into_response();
    }
    let path = uri.path().trim_start_matches('/');
    if path.is_empty() || path == "index.html" {
        let response = (
            [
                (
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("text/html; charset=utf-8"),
                ),
                (header::CACHE_CONTROL, HeaderValue::from_static("no-cache")),
            ],
            Body::from(index().html.clone()),
        )
            .into_response();
        return with_security_headers(response);
    }
    match Assets::get(path) {
        Some(file) => {
            let mime = file.metadata.mimetype();
            let response = (
                [
                    (
                        header::CONTENT_TYPE,
                        HeaderValue::from_str(mime).unwrap_or_else(|_| {
                            HeaderValue::from_static("application/octet-stream")
                        }),
                    ),
                    // File names carry their content hash.
                    (
                        header::CACHE_CONTROL,
                        HeaderValue::from_static("public, max-age=31536000, immutable"),
                    ),
                ],
                Body::from(file.data.into_owned()),
            )
                .into_response();
            with_security_headers(response)
        }
        None => crate::error::not_found().await.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_loader_scripts_are_hashed() {
        let html = "<script type=\"module\">boot()</script><script src=\"/x.js\"></script>";
        let policy = content_security_policy(html);
        let digest = Sha256::digest(b"boot()");
        let expected = format!("'sha256-{}'", data_encoding::BASE64.encode(&digest));
        assert!(policy.contains(&expected), "{policy}");
        assert_eq!(policy.matches("'sha256-").count(), 1);
        assert!(policy.contains("frame-ancestors 'none'"));
        assert!(!policy.contains("unsafe-inline"));
    }
}
