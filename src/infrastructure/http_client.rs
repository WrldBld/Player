//! Unified HTTP client for Engine API requests
//!
//! This module provides a platform-agnostic HTTP client that abstracts away
//! the differences between WASM (gloo_net) and desktop (reqwest) environments.
//!
//! # Usage
//!
//! ```ignore
//! use crate::infrastructure::http_client::{HttpClient, ApiError};
//!
//! // GET request
//! let worlds: Vec<WorldData> = HttpClient::get("/api/worlds").await?;
//!
//! // POST request with JSON body
//! let created: CharacterData = HttpClient::post("/api/characters", &new_character).await?;
//!
//! // PUT request
//! HttpClient::put_no_response("/api/characters/123", &update).await?;
//!
//! // DELETE request
//! HttpClient::delete("/api/characters/123").await?;
//! ```

use serde::{de::DeserializeOwned, Serialize};
use std::fmt;

use super::api::get_engine_url;

/// API error type for all HTTP operations
#[derive(Debug, Clone)]
pub enum ApiError {
    /// Network request failed
    RequestFailed(String),
    /// Server returned non-success status
    HttpError(u16, String),
    /// Failed to parse response JSON
    ParseError(String),
    /// Failed to serialize request body
    SerializeError(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::RequestFailed(msg) => write!(f, "Request failed: {}", msg),
            ApiError::HttpError(status, msg) => write!(f, "HTTP {}: {}", status, msg),
            ApiError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ApiError::SerializeError(msg) => write!(f, "Serialize error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

// Allow conversion to String for backward compatibility
impl From<ApiError> for String {
    fn from(err: ApiError) -> String {
        err.to_string()
    }
}

/// Unified HTTP client for Engine API
///
/// All methods take a path (e.g., "/api/worlds") and automatically
/// prepend the Engine base URL from configuration.
pub struct HttpClient;

impl HttpClient {
    /// Build full URL from API path
    fn build_url(path: &str) -> String {
        let base = get_engine_url();
        if path.starts_with('/') {
            format!("{}{}", base, path)
        } else {
            format!("{}/{}", base, path)
        }
    }

    /// GET request that returns deserialized JSON
    pub async fn get<T: DeserializeOwned>(path: &str) -> Result<T, ApiError> {
        let url = Self::build_url(path);

        #[cfg(target_arch = "wasm32")]
        {
            use gloo_net::http::Request;

            let response = Request::get(&url)
                .send()
                .await
                .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

            if response.ok() {
                response
                    .json::<T>()
                    .await
                    .map_err(|e| ApiError::ParseError(e.to_string()))
            } else {
                Err(ApiError::HttpError(
                    response.status(),
                    format!("GET {} failed", path),
                ))
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = reqwest::Client::new();
            let response = client
                .get(&url)
                .send()
                .await
                .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

            let status = response.status().as_u16();
            if response.status().is_success() {
                response
                    .json::<T>()
                    .await
                    .map_err(|e| ApiError::ParseError(e.to_string()))
            } else {
                Err(ApiError::HttpError(status, format!("GET {} failed", path)))
            }
        }
    }

    /// POST request with JSON body, returns deserialized JSON response
    pub async fn post<T: DeserializeOwned, B: Serialize>(path: &str, body: &B) -> Result<T, ApiError> {
        let url = Self::build_url(path);

        #[cfg(target_arch = "wasm32")]
        {
            use gloo_net::http::Request;

            let body_str = serde_json::to_string(body)
                .map_err(|e| ApiError::SerializeError(e.to_string()))?;

            let response = Request::post(&url)
                .header("Content-Type", "application/json")
                .body(body_str)
                .map_err(|e| ApiError::RequestFailed(e.to_string()))?
                .send()
                .await
                .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

            if response.ok() {
                response
                    .json::<T>()
                    .await
                    .map_err(|e| ApiError::ParseError(e.to_string()))
            } else {
                Err(ApiError::HttpError(
                    response.status(),
                    format!("POST {} failed", path),
                ))
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = reqwest::Client::new();
            let response = client
                .post(&url)
                .json(body)
                .send()
                .await
                .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

            let status = response.status().as_u16();
            if response.status().is_success() {
                response
                    .json::<T>()
                    .await
                    .map_err(|e| ApiError::ParseError(e.to_string()))
            } else {
                Err(ApiError::HttpError(status, format!("POST {} failed", path)))
            }
        }
    }

    /// POST request with JSON body, no response body expected
    pub async fn post_no_response<B: Serialize>(path: &str, body: &B) -> Result<(), ApiError> {
        let url = Self::build_url(path);

        #[cfg(target_arch = "wasm32")]
        {
            use gloo_net::http::Request;

            let body_str = serde_json::to_string(body)
                .map_err(|e| ApiError::SerializeError(e.to_string()))?;

            let response = Request::post(&url)
                .header("Content-Type", "application/json")
                .body(body_str)
                .map_err(|e| ApiError::RequestFailed(e.to_string()))?
                .send()
                .await
                .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

            if response.ok() {
                Ok(())
            } else {
                Err(ApiError::HttpError(
                    response.status(),
                    format!("POST {} failed", path),
                ))
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = reqwest::Client::new();
            let response = client
                .post(&url)
                .json(body)
                .send()
                .await
                .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

            let status = response.status().as_u16();
            if response.status().is_success() {
                Ok(())
            } else {
                Err(ApiError::HttpError(status, format!("POST {} failed", path)))
            }
        }
    }

    /// POST request without body, no response expected (for toggle endpoints)
    pub async fn post_empty(path: &str) -> Result<(), ApiError> {
        let url = Self::build_url(path);

        #[cfg(target_arch = "wasm32")]
        {
            use gloo_net::http::Request;

            let response = Request::post(&url)
                .send()
                .await
                .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

            if response.ok() {
                Ok(())
            } else {
                Err(ApiError::HttpError(
                    response.status(),
                    format!("POST {} failed", path),
                ))
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = reqwest::Client::new();
            let response = client
                .post(&url)
                .send()
                .await
                .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

            let status = response.status().as_u16();
            if response.status().is_success() {
                Ok(())
            } else {
                Err(ApiError::HttpError(status, format!("POST {} failed", path)))
            }
        }
    }

    /// PUT request with JSON body, returns deserialized JSON response
    pub async fn put<T: DeserializeOwned, B: Serialize>(path: &str, body: &B) -> Result<T, ApiError> {
        let url = Self::build_url(path);

        #[cfg(target_arch = "wasm32")]
        {
            use gloo_net::http::Request;

            let body_str = serde_json::to_string(body)
                .map_err(|e| ApiError::SerializeError(e.to_string()))?;

            let response = Request::put(&url)
                .header("Content-Type", "application/json")
                .body(body_str)
                .map_err(|e| ApiError::RequestFailed(e.to_string()))?
                .send()
                .await
                .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

            if response.ok() {
                response
                    .json::<T>()
                    .await
                    .map_err(|e| ApiError::ParseError(e.to_string()))
            } else {
                Err(ApiError::HttpError(
                    response.status(),
                    format!("PUT {} failed", path),
                ))
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = reqwest::Client::new();
            let response = client
                .put(&url)
                .json(body)
                .send()
                .await
                .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

            let status = response.status().as_u16();
            if response.status().is_success() {
                response
                    .json::<T>()
                    .await
                    .map_err(|e| ApiError::ParseError(e.to_string()))
            } else {
                Err(ApiError::HttpError(status, format!("PUT {} failed", path)))
            }
        }
    }

    /// PUT request with JSON body, no response body expected
    pub async fn put_no_response<B: Serialize>(path: &str, body: &B) -> Result<(), ApiError> {
        let url = Self::build_url(path);

        #[cfg(target_arch = "wasm32")]
        {
            use gloo_net::http::Request;

            let body_str = serde_json::to_string(body)
                .map_err(|e| ApiError::SerializeError(e.to_string()))?;

            let response = Request::put(&url)
                .header("Content-Type", "application/json")
                .body(body_str)
                .map_err(|e| ApiError::RequestFailed(e.to_string()))?
                .send()
                .await
                .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

            if response.ok() {
                Ok(())
            } else {
                Err(ApiError::HttpError(
                    response.status(),
                    format!("PUT {} failed", path),
                ))
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = reqwest::Client::new();
            let response = client
                .put(&url)
                .json(body)
                .send()
                .await
                .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

            let status = response.status().as_u16();
            if response.status().is_success() {
                Ok(())
            } else {
                Err(ApiError::HttpError(status, format!("PUT {} failed", path)))
            }
        }
    }

    /// PUT request without body (for toggle endpoints), no response expected
    pub async fn put_empty(path: &str) -> Result<(), ApiError> {
        let url = Self::build_url(path);

        #[cfg(target_arch = "wasm32")]
        {
            use gloo_net::http::Request;

            let response = Request::put(&url)
                .send()
                .await
                .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

            if response.ok() {
                Ok(())
            } else {
                Err(ApiError::HttpError(
                    response.status(),
                    format!("PUT {} failed", path),
                ))
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = reqwest::Client::new();
            let response = client
                .put(&url)
                .send()
                .await
                .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

            let status = response.status().as_u16();
            if response.status().is_success() {
                Ok(())
            } else {
                Err(ApiError::HttpError(status, format!("PUT {} failed", path)))
            }
        }
    }

    /// PUT request without body, returns deserialized JSON response (for toggle endpoints that return state)
    pub async fn put_empty_with_response<T: DeserializeOwned>(path: &str) -> Result<T, ApiError> {
        let url = Self::build_url(path);

        #[cfg(target_arch = "wasm32")]
        {
            use gloo_net::http::Request;

            let response = Request::put(&url)
                .send()
                .await
                .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

            if response.ok() {
                response
                    .json::<T>()
                    .await
                    .map_err(|e| ApiError::ParseError(e.to_string()))
            } else {
                Err(ApiError::HttpError(
                    response.status(),
                    format!("PUT {} failed", path),
                ))
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = reqwest::Client::new();
            let response = client
                .put(&url)
                .send()
                .await
                .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

            let status = response.status().as_u16();
            if response.status().is_success() {
                response
                    .json::<T>()
                    .await
                    .map_err(|e| ApiError::ParseError(e.to_string()))
            } else {
                Err(ApiError::HttpError(status, format!("PUT {} failed", path)))
            }
        }
    }

    /// DELETE request
    pub async fn delete(path: &str) -> Result<(), ApiError> {
        let url = Self::build_url(path);

        #[cfg(target_arch = "wasm32")]
        {
            use gloo_net::http::Request;

            let response = Request::delete(&url)
                .send()
                .await
                .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

            if response.ok() {
                Ok(())
            } else {
                Err(ApiError::HttpError(
                    response.status(),
                    format!("DELETE {} failed", path),
                ))
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = reqwest::Client::new();
            let response = client
                .delete(&url)
                .send()
                .await
                .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

            let status = response.status().as_u16();
            if response.status().is_success() {
                Ok(())
            } else {
                Err(ApiError::HttpError(status, format!("DELETE {} failed", path)))
            }
        }
    }

    /// GET request that returns Option<T> - returns None for 404, Some(T) for success
    pub async fn get_optional<T: DeserializeOwned>(path: &str) -> Result<Option<T>, ApiError> {
        let url = Self::build_url(path);

        #[cfg(target_arch = "wasm32")]
        {
            use gloo_net::http::Request;

            let response = Request::get(&url)
                .send()
                .await
                .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

            if response.status() == 404 {
                return Ok(None);
            }

            if response.ok() {
                let data = response
                    .json::<T>()
                    .await
                    .map_err(|e| ApiError::ParseError(e.to_string()))?;
                Ok(Some(data))
            } else {
                Err(ApiError::HttpError(
                    response.status(),
                    format!("GET {} failed", path),
                ))
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = reqwest::Client::new();
            let response = client
                .get(&url)
                .send()
                .await
                .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Ok(None);
            }

            let status = response.status().as_u16();
            if response.status().is_success() {
                let data = response
                    .json::<T>()
                    .await
                    .map_err(|e| ApiError::ParseError(e.to_string()))?;
                Ok(Some(data))
            } else {
                Err(ApiError::HttpError(status, format!("GET {} failed", path)))
            }
        }
    }
}

// ============================================================================
// ApiPort Implementation
// ============================================================================

use crate::application::ports::outbound::{ApiError as PortApiError, ApiPort};

/// Convert infrastructure ApiError to port ApiError
fn to_port_error(err: ApiError) -> PortApiError {
    match err {
        ApiError::RequestFailed(msg) => PortApiError::RequestFailed(msg),
        ApiError::HttpError(status, msg) => PortApiError::HttpError(status, msg),
        ApiError::ParseError(msg) => PortApiError::ParseError(msg),
        ApiError::SerializeError(msg) => PortApiError::SerializeError(msg),
    }
}

/// API adapter that implements the ApiPort trait
///
/// This adapter wraps the static HttpClient methods to provide an instance-based
/// API that can be injected into application services.
#[derive(Clone, Debug, Default)]
pub struct ApiAdapter;

impl ApiAdapter {
    /// Create a new API adapter
    pub fn new() -> Self {
        Self
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl ApiPort for ApiAdapter {
    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, PortApiError> {
        HttpClient::get(path).await.map_err(to_port_error)
    }

    async fn get_optional<T: DeserializeOwned>(&self, path: &str) -> Result<Option<T>, PortApiError> {
        HttpClient::get_optional(path).await.map_err(to_port_error)
    }

    async fn post<T: DeserializeOwned, B: Serialize + Send + Sync>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, PortApiError> {
        HttpClient::post(path, body).await.map_err(to_port_error)
    }

    async fn post_no_response<B: Serialize + Send + Sync>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<(), PortApiError> {
        HttpClient::post_no_response(path, body).await.map_err(to_port_error)
    }

    async fn post_empty(&self, path: &str) -> Result<(), PortApiError> {
        HttpClient::post_empty(path).await.map_err(to_port_error)
    }

    async fn put<T: DeserializeOwned, B: Serialize + Send + Sync>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, PortApiError> {
        HttpClient::put(path, body).await.map_err(to_port_error)
    }

    async fn put_no_response<B: Serialize + Send + Sync>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<(), PortApiError> {
        HttpClient::put_no_response(path, body).await.map_err(to_port_error)
    }

    async fn put_empty(&self, path: &str) -> Result<(), PortApiError> {
        HttpClient::put_empty(path).await.map_err(to_port_error)
    }

    async fn put_empty_with_response<T: DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, PortApiError> {
        HttpClient::put_empty_with_response(path).await.map_err(to_port_error)
    }

    async fn delete(&self, path: &str) -> Result<(), PortApiError> {
        HttpClient::delete(path).await.map_err(to_port_error)
    }
}
