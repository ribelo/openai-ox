use serde::{Deserialize, Serialize};

use crate::{ApiRequest, ApiRequestError, ApiRequestWithClient, ErrorResponse, OpenAi};

// #[allow(dead_code)]
// #[derive(Debug, Serialize)]
// pub struct EmbeddingRequest<'a> {
//     model: &'a str,
//     input: Vec<&'a str>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     user: Option<String>,
//     #[serde(skip)]
//     client: Option<OpenAi>,
// }
//
// /// # Struct `EmbeddingResponse`
// ///
// /// The response received for an embedding request.
// ///
// /// # Fields
// ///
// /// - `object`: `String` - Object type returned by the request.
// ///
// /// - `data`: `Vec<EmbeddingData>` - A list of embedding data for each input text.
// ///
// /// - `model`: `String` - The model used to generate the embeddings.
// ///
// /// - `usage`: `Usage` - Information about the usage of tokens in the request.
// #[derive(Debug, Deserialize)]
// pub struct EmbeddingResponse {
//     pub object: String,
//     pub data: Vec<EmbeddingData>,
//     pub model: String,
//     pub usage: Usage,
// }
//
// /// # Struct `EmbeddingData`
// ///
// /// Represents the embedding data for an input text.
// ///
// /// # Fields
// ///
// /// - `object`: `String` - The type of the object returned.
// ///
// /// - `embedding`: `Vec<f32>` - The generated embedding for the input text as a list of f32 values.
// ///
// /// - `index`: `usize` - The index of the input text in the input list for which this embedding was generated.
// #[derive(Debug, Deserialize)]
// pub struct EmbeddingData {
//     pub object: String,
//     pub embedding: Vec<f32>,
//     pub index: usize,
// }
//
// /// # Struct `Usage`
// ///
// /// Tracks the usage of tokens in the request.
// ///
// /// # Fields
// ///
// /// - `prompt_tokens`: `usize` - The number of tokens used in the input text.
// ///
// /// - `total_tokens`: `usize` - The total number of tokens used in the request (includes both input and output tokens).
// #[derive(Debug, Deserialize)]
// pub struct Usage {
//     pub prompt_tokens: usize,
//     pub total_tokens: usize,
// }
//
// pub trait EmbeddingsExt {
//     fn to_embeddings_request(self) -> EmbeddingRequest<NoModel, NoClient>;
// }
//
// impl Default for EmbeddingRequest<NoModel, NoClient> {
//     fn default() -> Self {
//         EmbeddingRequest {
//             model_status: NoModel,
//             client_status: NoClient,
//             model: None,
//             input: Vec::new(),
//             user: None,
//             client: None,
//         }
//     }
// }
//
// impl EmbeddingRequest<NoModel, NoClient> {
//     pub fn new(input: &[String]) -> EmbeddingRequest<NoModel, NoClient> {
//         EmbeddingRequest {
//             model: None,
//             input: input.to_vec(),
//             user: None,
//             model_status: NoModel,
//             client_status: NoClient,
//             client: None,
//         }
//     }
// }
//
// impl<M, C> EmbeddingRequest<M, C>
// where
//     M: ModelStatus,
//     C: ClientStatus,
// {
//     /// Updates the `model` attribute and returns the modified `EmbeddingRequest`.
//     ///
//     /// - `model: String` - The new model identifier.
//     ///
//     /// Examples:
//     ///
//     /// ```
//     /// let request = request.with_model("new_model_name".to_string());
//     /// ```
//     pub fn model(self, model: impl ToString) -> EmbeddingRequest<HasModel, C> {
//         EmbeddingRequest {
//             model: Some(model.to_string()),
//             input: self.input,
//             user: self.user,
//             model_status: HasModel,
//             client_status: self.client_status,
//             client: self.client,
//         }
//     }
//
//     /// Updates the `input` attribute and returns the modified `EmbeddingRequest`.
//     ///
//     /// - `input: Vec<String>` - The new text inputs.
//     ///
//     /// Examples:
//     ///
//     /// ```
//     /// let request = request.with_input(vec!["new_input1".to_string(), "new_input2".to_string()]);
//     /// ```
//     pub fn with_input(mut self, input: Vec<String>) -> Self {
//         self.input = input;
//         self
//     }
//
//     /// Updates the `user` attribute and returns the modified `EmbeddingRequest`.
//     ///
//     /// - `user: Option<String>` - The new user value.
//     ///
//     /// Examples:
//     ///
//     /// ```
//     /// let request = request.with_user(Some("new_user".to_string()));
//     /// ```
//     pub fn with_user(mut self, user: Option<String>) -> Self {
//         self.user = user;
//         self
//     }
//
//     /// Updates the `client` attribute and sets the `client_status` to `WithClient`.
//     /// Returns a new `EmbeddingRequest` with the respective type adaptation.
//     /// This method consumes `self`.
//     ///
//     /// - `client: Option<OpenAi>` - The new client value.
//     pub fn client(self, client: &OpenAi) -> EmbeddingRequest<M, HasClient> {
//         EmbeddingRequest {
//             model: self.model,
//             input: self.input,
//             user: self.user,
//             model_status: self.model_status,
//             client_status: HasClient,
//             client: Some(client.clone()),
//         }
//     }
// }
//
// impl OpenAi {
//     /// Retrieves semantic embeddings for provided text using OpenAI's API.
//     ///
//     /// This method posts a request to OpenAI's endpoint, https://api.openai.com/v1/embeddings, to extract
//     /// semantic embeddings, a representation of the provided text in a semantic vector.
//     ///
//     /// # Type Parameters
//     ///
//     /// - `S`: An entity that implements the `ClientStatus` trait. This trait allows
//     /// the system to track the status of an HTTP client while processing a request.
//     ///
//     /// # Parameters
//     ///
//     /// - `request`: &EmbeddingRequest<S> - Reference to a `EmbeddingRequest` object
//     /// defined by the user which needs to specify text sequences for embedding computation.
//     ///
//     /// # Returns
//     ///
//     /// - Returns a `Future` that resolves to `Ok(EmbeddingResponse)` if the request
//     /// was successful. The `EmbeddingResponse` struct contains the computed embeddings.
//     ///
//     /// - If the request fails it returns `ApiRequestError` describing the failure.
//     ///
//     /// # Examples
//     ///
//     /// ```rust
//     /// let client = OpenAi::new("my-api-key");
//     /// let request = EmbeddingRequest::new(vec!["Please generate embedding for this text."]);
//     /// let response = client.embeddings(&request).await;
//     ///
//     /// match response {
//     ///     Ok(embedding) => println!("Embedding: {:?}", embedding),
//     ///     Err(e) => eprintln!("API request failed: {:?}", e),
//     /// }
//     /// ```
//     pub async fn embeddings<HasModel, HasClient>(
//         &self,
//         request: &EmbeddingRequest<HasModel, HasClient>,
//     ) -> Result<EmbeddingResponse, ApiRequestError>
//     where
//         HasModel: ModelStatus,
//         HasClient: ClientStatus,
//     {
//         cfg_if::cfg_if! {
//             if #[cfg(feature = "leaky-bucket")] {
//                 if let Some(rate_limiter) = self.leaky_bucket.as_ref() {
//                     rate_limiter.acquire_one().await;
//                 }
//             }
//         }
//
//         let url = "https://api.openai.com/v1/embeddings";
//         let response = self
//             .client
//             .post(url)
//             .header("Content-Type", "application/json")
//             .bearer_auth(&self.api_key)
//             .json(&request)
//             .send()
//             .await?;
//
//         if response.status().is_success() {
//             let data: EmbeddingResponse = response.json().await?;
//             Ok(data)
//         } else {
//             let error_response: ErrorResponse = response.json().await?;
//             Err(ApiRequestError::InvalidRequestError {
//                 message: error_response.error.message,
//                 param: error_response.error.param,
//                 code: error_response.error.code,
//             })
//         }
//     }
// }
//
// #[async_trait::async_trait]
// impl<C: ClientStatus + std::marker::Sync> ApiRequest for EmbeddingRequest<HasModel, C> {
//     type Response = EmbeddingResponse;
//     async fn send_with(&self, open_ai: &OpenAi) -> Result<Self::Response, ApiRequestError> {
//         open_ai.embeddings(self).await
//     }
// }
//
// #[async_trait::async_trait]
// impl ApiRequestWithClient for EmbeddingRequest<HasModel, HasClient> {
//     async fn send(&self) -> Result<Self::Response, ApiRequestError> {
//         match &self.client {
//             Some(client) => self.send_with(client).await,
//             None => Err(ApiRequestError::NoClient),
//         }
//     }
// }
