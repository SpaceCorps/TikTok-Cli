//! HTTP client to the Apify/TikTok API, and translation from HTTP status to [`ErrorCode`].

use std::time::Duration;

use serde_json::Value;
use ureq::Agent;
use ureq::http::Response;

use crate::error::{Error, ErrorCode, Result};

const DEFAULT_BASE: &str = "https://api.apify.com/v2/";
const MAX_BODY: u64 = 512 * 1024 * 1024;

#[derive(Clone)]
pub struct Client {
    agent: Agent,
    base: String,
    token: String,
}

enum Method {
    Get,
    Post,
}

impl Client {
    pub fn new(api_key: &str) -> Client {
        let agent: Agent = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(300)))
            .timeout_connect(Some(Duration::from_secs(15)))
            .http_status_as_error(false)
            .user_agent(concat!("tiktok-cli/", env!("CARGO_PKG_VERSION")))
            .build()
            .into();

        let mut base = std::env::var("TIKTOK_API_URL")
            .or_else(|_| std::env::var("APIFY_API_URL"))
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_BASE.to_string());
        if !base.ends_with('/') {
            base.push('/');
        }

        Client { agent, base, token: api_key.trim().to_string() }
    }

    pub fn get(&self, path: &str) -> Result<Value> {
        self.send(Method::Get, path, None)
    }

    pub fn post(&self, path: &str, body: &Value) -> Result<Value> {
        self.send(Method::Post, path, Some(body))
    }

    /// Run the Apify TikTok scraper actor synchronously and retrieve dataset items.
    pub fn scrape(&self, body: &Value) -> Result<Value> {
        let endpoint = format!("acts/clockworks~tiktok-scraper/run-sync-get-dataset-items?token={}", self.token);
        self.post(&endpoint, body)
    }

    fn send(&self, method: Method, path: &str, body: Option<&Value>) -> Result<Value> {
        let clean_path = path.trim_start_matches('/');
        let url = format!("{}{}", self.base, clean_path);

        let auth_header = format!("Bearer {}", self.token);

        macro_rules! headers {
            ($req:expr) => {
                $req.header("Authorization", &auth_header).header("Accept", "application/json")
            };
        }

        let result = match (method, body) {
            (Method::Get, _) => headers!(self.agent.get(&url)).call(),
            (Method::Post, Some(b)) => {
                let json = serde_json::to_vec(b).expect("a Value always serializes");
                headers!(self.agent.post(&url)).header("Content-Type", "application/json").send(&json[..])
            }
            (Method::Post, None) => headers!(self.agent.post(&url)).send_empty(),
        };

        let response = result.map_err(transport_error)?;
        read(response)
    }
}

fn read(mut response: Response<ureq::Body>) -> Result<Value> {
    let status = response.status().as_u16();
    let bytes = response.body_mut().with_config().limit(MAX_BODY).read_to_vec().map_err(transport_error)?;

    if !(200..300).contains(&status) {
        let body = String::from_utf8_lossy(&bytes).trim().to_string();
        return Err(status_error(status, &body));
    }

    if bytes.iter().all(u8::is_ascii_whitespace) {
        return Ok(Value::Object(Default::default()));
    }

    serde_json::from_slice(&bytes).or_else(|_| Ok(Value::String(String::from_utf8_lossy(&bytes).into_owned())))
}

fn transport_error(e: ureq::Error) -> Error {
    match e {
        ureq::Error::Timeout(_) => Error::new(ErrorCode::Network, "The request timed out.")
            .fix("Retry once, or check actor progress on Apify."),
        other => Error::new(ErrorCode::Network, "Could not reach the Apify API.")
            .detail(other.to_string())
            .fix("Check your internet connection, APIFY_TOKEN, or TIKTOK_API_URL."),
    }
}

pub fn status_error(status: u16, body: &str) -> Error {
    let mut detail = format!("HTTP {status}");
    if !body.is_empty() {
        detail.push_str(": ");
        detail.push_str(body);
    }

    // Try extracting error message from JSON body (supports Apify {error: {message: ...}} and string error)
    let err_msg = serde_json::from_str::<Value>(body).ok().and_then(|v| {
        if let Some(err_obj) = v.get("error").and_then(Value::as_object)
            && let Some(msg) = err_obj.get("message").and_then(Value::as_str)
        {
            return Some(msg.to_string());
        }
        v.get("error").and_then(Value::as_str).or_else(|| v.get("message").and_then(Value::as_str)).map(str::to_string)
    });

    let e = match status {
        401 => {
            Error::new(ErrorCode::AuthRequired, err_msg.unwrap_or_else(|| "The Apify API token was rejected.".into()))
                .fix("Check your API token: tiktok accounts add <name> --api-key <key> --force or set APIFY_TOKEN")
        }
        403 => Error::new(
            ErrorCode::AuthRequired,
            err_msg.unwrap_or_else(|| "The Apify API token does not have permission for this request.".into()),
        )
        .fix("Check your Apify account permissions and billing at https://console.apify.com/billing"),
        404 => {
            Error::new(ErrorCode::NotFound, err_msg.unwrap_or_else(|| "The requested resource was not found.".into()))
        }
        429 => Error::new(
            ErrorCode::RateLimited,
            err_msg.unwrap_or_else(|| "Rate limit or compute unit limit exceeded on Apify.".into()),
        )
        .fix("Back off before retrying, or check your compute usage at https://console.apify.com/billing"),
        400 | 422 => Error::new(
            ErrorCode::InvalidInput,
            err_msg.unwrap_or_else(|| "The Apify API rejected the request parameters.".into()),
        ),
        s if s >= 500 => Error::new(ErrorCode::Network, "The Apify API returned a server error.")
            .fix("Retry once; if it persists, check https://status.apify.com"),
        _ => Error::new(ErrorCode::Error, err_msg.unwrap_or_else(|| "The TikTok/Apify request failed.".into())),
    };
    e.detail(detail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_maps_to_codes() {
        assert_eq!(status_error(401, "").code, ErrorCode::AuthRequired);
        assert_eq!(status_error(404, "").code, ErrorCode::NotFound);
        assert_eq!(status_error(429, "").code, ErrorCode::RateLimited);
        assert_eq!(status_error(422, "").code, ErrorCode::InvalidInput);
        assert_eq!(status_error(500, "").code, ErrorCode::Network);
    }

    #[test]
    fn parses_apify_error_object() {
        let json = r#"{"error":{"type":"invalid-token","message":"The token is invalid."}}"#;
        let err = status_error(401, json);
        assert_eq!(err.message, "The token is invalid.");
    }
}
