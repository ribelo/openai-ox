use bon::Builder;
use serde::Deserialize;
use thiserror::Error;

pub mod audio;
pub mod chat;
pub mod embeddings;
pub mod models;
const BASE_URL: &str = "https://api.openai.com";

#[cfg(feature = "leaky-bucket")]
pub use leaky_bucket::RateLimiter;
use std::fmt;
#[cfg(feature = "leaky-bucket")]
use std::sync::Arc;

#[derive(Clone, Builder)]
pub struct OpenAi {
    api_key: String,
    #[builder(default)]
    client: reqwest::Client,
    #[cfg(feature = "leaky-bucket")]
    leaky_bucket: Option<Arc<RateLimiter>>,
}

impl fmt::Debug for OpenAi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenAi")
            .field("api_key", &"[REDACTED]")
            .field("client", &self.client)
            .finish()
    }
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: ApiErrorDetail,
}

#[derive(Debug, Deserialize)]
pub struct ApiErrorDetail {
    message: String,
    #[serde(default)]
    param: Option<String>,
    #[serde(default)]
    code: Option<String>,
}

#[derive(Debug, Error)]
pub enum ApiRequestError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),

    #[error("Invalid request error: {message}")]
    InvalidRequestError {
        message: String,
        param: Option<String>,
        code: Option<String>,
    },
    #[error("Unexpected response from API: {response}")]
    UnexpectedResponse { response: String },
    #[error("Stream error: {0}")]
    Stream(String),
}

/// `ApiRequest` trait allows sending any prepared request by explicitly providing OpenAI client.
///
/// This trait is useful to abstract the details about API request, response, and error handling.
///
/// # Associated Types
///
/// - `Response`: A type that implements the `DeserializeOwned` trait from Serde. This type
/// represents the deserialized response that you expect back from the API call.
#[async_trait::async_trait]
pub trait ApiRequest {
    type Response: serde::de::DeserializeOwned;
    /// An async function that takes in an `OpenAi` object reference and returns a `Result` with
    /// deserialized `Response` type or an `ApiRequestError`. This function sends off the API
    /// request with given OpenAi client.
    async fn send_with(&self, open_ai: &OpenAi) -> Result<Self::Response, ApiRequestError>;
}

/// `ApiRequestWithClient` trait allows sending any prepared request which internally uses OpenAI
/// client.
///
/// This trait is useful when the client does not want to externally manage or provide the `OpenAi`
/// client for making requests.
#[async_trait::async_trait]
pub trait ApiRequestWithClient: ApiRequest {
    /// An async function that takes no parameters. It internally uses the API client and so
    /// returns a `Result` with deserialized `Response` type or an `ApiRequestError`. This function
    /// sends off the API request.
    async fn send(&self) -> Result<Self::Response, ApiRequestError>;
}
