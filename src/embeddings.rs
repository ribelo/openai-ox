use std::borrow::Cow;

use bon::Builder;
use serde::{Deserialize, Serialize};

use crate::{ApiRequestError, ErrorResponse, OpenAi};

#[derive(Debug, Serialize, Builder)]
pub struct EmbeddingRequest {
    #[builder(into)]
    model: String,
    input: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dimensions: Option<u32>,
    #[serde(skip)]
    openai: OpenAi,
}

#[derive(Debug, Deserialize)]
pub struct EmbeddingResponse {
    pub object: String,
    pub data: Vec<EmbeddingData>,
    pub model: String,
    pub usage: Usage,
}

#[derive(Debug, Deserialize)]
pub struct EmbeddingData {
    pub object: String,
    pub embedding: Vec<f32>,
    pub index: usize,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum EmbeddingRequestBuilderError {
    #[error("Missing required field: model")]
    MissingModel,
    #[error("Missing required field: openai client")]
    MissingClient,
}

impl EmbeddingRequest {
    pub async fn send(&self) -> Result<EmbeddingResponse, ApiRequestError> {
        #[cfg(feature = "leaky-bucket")]
        if let Some(rate_limiter) = self.openai.leaky_bucket.as_ref() {
            rate_limiter.acquire_one().await;
        }

        let url = "https://api.openai.com/v1/embeddings";
        let response = self
            .openai
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .bearer_auth(&self.openai.api_key)
            .json(&self)
            .send()
            .await?;

        if response.status().is_success() {
            let data: EmbeddingResponse = response.json().await?;
            Ok(data)
        } else {
            let error_response: ErrorResponse = response.json().await?;
            Err(ApiRequestError::InvalidRequestError {
                message: error_response.error.message,
                param: error_response.error.param,
                code: error_response.error.code,
            })
        }
    }
}

impl OpenAi {
    pub fn embeddings(&self) -> EmbeddingRequestBuilder<embedding_request_builder::SetOpenai> {
        EmbeddingRequest::builder().openai(self.clone())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedding_request() {
        let openai_api_key = std::env::var("OPENAI_API_KEY").unwrap();
        let openai = OpenAi::builder().api_key(openai_api_key).build();
        let request = openai
            .embeddings()
            .model("text-embedding-3-small")
            .dimensions(256)
            .input(vec!["Hello world".to_string()])
            .build();

        let response = request.send().await.unwrap();
        dbg!(&response.data[0].embedding);
        // dbg!(response);
    }
}
