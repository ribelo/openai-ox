use serde::Deserialize;
use thiserror::Error;

pub mod audio;
pub mod chat;
pub mod embeddings;
pub mod models;
pub mod tokenizer;
const BASE_URL: &str = "https://api.openai.com";

cfg_if::cfg_if! {
    if #[cfg(feature = "leaky-bucket")] {
        use derivative::Derivative;
        use std::sync::Arc;
        pub use leaky_bucket::RateLimiter;
    }
}

#[derive(Default)]
pub struct OpenAiBuilder {
    api_key: Option<String>,
    client: Option<reqwest::Client>,
    #[cfg(feature = "leaky-bucket")]
    leaky_bucket: Option<RateLimiter>,
}

cfg_if::cfg_if! {
    if #[cfg(feature = "leaky-bucket")] {
        #[derive(Clone, Derivative)]
        #[derivative(Debug)]
        pub struct OpenAi {
            api_key: String,
            client: reqwest::Client,
            #[derivative(Debug = "ignore")]
            leaky_bucket: Option<Arc<RateLimiter>>,
        }
    } else {
        #[derive(Clone, Debug)]
        pub struct OpenAi {
            api_key: String,
            client: reqwest::Client,
        }
    }
}

#[derive(Debug, Error)]
pub enum OpenAiBuilderError {
    #[error("API key not set")]
    ApiKeyNotSet,
}

impl OpenAi {
    pub fn builder() -> OpenAiBuilder {
        OpenAiBuilder::default()
    }
}

impl OpenAiBuilder {
    pub fn new() -> Self {
        Default::default()
    }
    /// Sets the API key for the OpenAI builder.
    ///
    /// # Parameters
    /// * `api_key`: OpenAi API key as `String`
    ///
    /// # Returns
    /// * An instance of `OpenAiBuilder` with the API key set
    pub fn api_key(mut self, api_key: String) -> OpenAiBuilder {
        self.api_key = Some(api_key);
        self
    }

    /// Sets the client for the OpenAI builder.
    ///
    /// # Parameters
    /// * `client`: A `reqwest::Client` instance
    ///
    /// # Returns
    /// * An instance of `OpenAiBuilder` with the client set
    pub fn client(mut self, client: &reqwest::Client) -> OpenAiBuilder {
        self.client = Some(client.clone());
        self
    }

    #[cfg(feature = "leaky-bucket")]
    /// Sets the RateLimiter for the OpenAI builder. This feature is only available if the "leaky-bucket" feature is enabled.
    ///
    /// # Parameters
    /// * `leaky_bucket`: An `Arc<RateLimiter>` instance
    ///
    /// # Returns
    /// * An instance of `OpenAiBuilder` with the RateLimiter set
    pub fn limiter(mut self, leaky_bucket: RateLimiter) -> OpenAiBuilder {
        self.leaky_bucket = Some(leaky_bucket);
        self
    }

    /// Builds the `OpenAi` instance using the set configuration.
    ///
    /// # Returns
    /// * An `OpenAi` instance
    ///
    /// # Panics
    /// * When neither API key nor client is provided.
    pub fn build(self) -> Result<OpenAi, OpenAiBuilderError> {
        let Some(api_key) = self.api_key else {
            return Err(OpenAiBuilderError::ApiKeyNotSet);
        };
        let client = self.client.unwrap_or_default();

        #[cfg(feature = "leaky-bucket")]
        let leaky_bucket = self.leaky_bucket.map(Arc::new);

        Ok(OpenAi {
            api_key,
            client,
            #[cfg(feature = "leaky-bucket")]
            leaky_bucket,
        })
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
    #[error(transparent)]
    EventSourceError(#[from] reqwest_eventsource::Error),

    #[error("Invalid request error: {message}")]
    InvalidRequestError {
        message: String,
        param: Option<String>,
        code: Option<String>,
    },
    #[error("Unexpected response from API: {response}")]
    UnexpectedResponse { response: String },
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
