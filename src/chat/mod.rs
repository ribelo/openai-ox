pub mod message;

use std::{borrow::Cow, ops::Deref, sync::Arc};

#[cfg(feature = "tools")]
use ai_tools_ox::tools::{self, ToTool, Tool, Tools};
use derivative::Derivative;
use reqwest_eventsource::{self, Event, RequestBuilderExt};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use thiserror::Error;
use tokio_stream::{wrappers::LinesStream, Stream, StreamExt};

use crate::{
    audio::transcription::TranscribeRequestBuilder, tokenizer::TokenCount, ApiRequest, ApiRequestError, ApiRequestWithClient, ErrorResponse, OpenAi, BASE_URL
};

use self::message::{Message, Messages};

const API_URL: &str = "v1/chat/completions";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "text")]
pub struct TextType;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "json_object")]
pub struct JsonType;

#[derive(Debug, Serialize, Deserialize)]
pub enum ResponseFormat {
    Text {
        #[serde(rename = "type")]
        format_type: TextType,
    },
    Json {
        #[serde(rename = "type")]
        format_type: JsonType,
    },
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    pub messages: Messages,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(feature = "tools")]
    pub tools: Option<Tools>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip)]
    pub openai: OpenAi,
}

#[derive(Debug, Default)]
pub struct ChatCompletionRequestBuilder {
    messages: Option<Messages>,
    model: Option<String>,
    frequency_penalty: Option<f32>,
    logit_bias: Option<serde_json::Value>,
    logprobs: Option<bool>,
    top_logprobs: Option<u32>,
    max_tokens: Option<u32>,
    n: Option<u32>,
    presence_penalty: Option<f32>,
    response_format: Option<ResponseFormat>,
    seed: Option<u32>,
    stop: Option<Vec<String>>,
    stream: Option<bool>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    #[cfg(feature = "tools")]
    tools: Option<Tools>,
    user: Option<String>,
    openai: Option<OpenAi>,
}

#[derive(Debug, Error)]
pub enum ChatCompletionRequestBuilderError {
    #[error("Messages not set")]
    MessagesNotSet,
    #[error("Model not set")]
    ModelNotSet,
    #[error("Client not set")]
    ClientNotSet,
}

impl ChatCompletionRequestBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn messages(mut self, messages: impl Into<Messages>) -> Self {
        self.messages = Some(messages.into());
        self
    }
    pub fn add_message(mut self, message: impl Into<Message>) -> Self {
        if let Some(ref mut messages) = self.messages {
            messages.push_message(message);
        } else {
            self.messages = Some(Messages::from(message.into()));
        }
        self
    }
    pub fn model(mut self, model: &str) -> Self {
        self.model = Some(model.to_string());
        self
    }
    pub fn frequency_penalty(mut self, frequency_penalty: f32) -> Self {
        self.frequency_penalty = Some(frequency_penalty);
        self
    }
    pub fn logit_bias(mut self, logit_bias: serde_json::Value) -> Self {
        self.logit_bias = Some(logit_bias);
        self
    }
    pub fn logprobs(mut self, logprobs: bool) -> Self {
        self.logprobs = Some(logprobs);
        self
    }
    pub fn top_logprobs(mut self, top_logprobs: u32) -> Self {
        self.top_logprobs = Some(top_logprobs);
        self
    }
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
    pub fn n(mut self, n: u32) -> Self {
        self.n = Some(n);
        self
    }
    pub fn presence_penalty(mut self, presence_penalty: f32) -> Self {
        self.presence_penalty = Some(presence_penalty);
        self
    }
    pub fn response_format(mut self, response_format: ResponseFormat) -> Self {
        self.response_format = Some(response_format);
        self
    }
    pub fn seed(mut self, seed: u32) -> Self {
        self.seed = Some(seed);
        self
    }
    pub fn stop(mut self, stop: Vec<String>) -> Self {
        self.stop = Some(stop);
        self
    }
    pub fn stream(mut self) -> Self {
        self.stream = Some(true);
        self
    }
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }
    #[cfg(feature = "tools")]
    pub fn tools(mut self, tools: Tools) -> Self {
        self.tools = Some(tools);
        self
    }
    pub fn user(mut self, user: String) -> Self {
        self.user = Some(user);
        self
    }
    pub fn client(mut self, client: OpenAi) -> Self {
        self.openai = Some(client);
        self
    }
    pub fn build(self) -> Result<ChatCompletionRequest, ChatCompletionRequestBuilderError> {
        let Some(messages) = self.messages else {
            return Err(ChatCompletionRequestBuilderError::MessagesNotSet);
        };
        let Some(model) = self.model else {
            return Err(ChatCompletionRequestBuilderError::ModelNotSet);
        };
        let Some(openai) = self.openai else {
            return Err(ChatCompletionRequestBuilderError::ClientNotSet);
        };

        Ok(ChatCompletionRequest {
            messages,
            model,
            frequency_penalty: self.frequency_penalty,
            logit_bias: self.logit_bias,
            logprobs: self.logprobs,
            top_logprobs: self.top_logprobs,
            max_tokens: self.max_tokens,
            n: self.n,
            presence_penalty: self.presence_penalty,
            response_format: self.response_format,
            seed: self.seed,
            stop: self.stop,
            stream: self.stream,
            temperature: self.temperature,
            top_p: self.top_p,
            #[cfg(feature = "tools")]
            tools: self.tools,
            user: self.user,
            openai,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Limit,
    ContentFilter,
    ToolCalls,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: FinishReason,
    pub logprobs: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Delta {
    pub content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChoiceStreamed {
    pub index: u32,
    pub delta: Delta,
    pub finish_reason: Option<FinishReason>,
    pub logprobs: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub created: u64,
    pub model: String,
    pub system_fingerprint: String,
    pub object: String,
    pub usage: Usage,
}

// impl From<ChatCompletionResponse> for String {
//     fn from(response: ChatCompletionResponse) -> Self {
//         response
//             .choices
//             .into_iter()
//             .map(|c| match c.message {
//                 Message::System(msg) => msg.content.clone(),
//                 Message::User(msg) => msg.content.clone(),
//                 Message::Assistant(msg) => msg.content.clone(),
//                 Message::Tool(_) => String::new(),
//             })
//             .collect()
//     }
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionChunkResponse {
    pub id: String,
    pub choices: Vec<ChoiceStreamed>,
    pub created: u64,
    pub model: String,
    pub system_fingerprint: String,
    pub object: String,
}

impl From<ChatCompletionChunkResponse> for String {
    fn from(response: ChatCompletionChunkResponse) -> Self {
        response
            .choices
            .into_iter()
            .map(|c| c.delta.content.unwrap_or_default())
            .collect()
    }
}

impl ChatCompletionRequest {
    pub async fn send(&self) -> Result<ChatCompletionResponse, ApiRequestError> {
        let url = format!("{}/{}", BASE_URL, API_URL);
        let req = self
            .openai
            .client
            .post(&url)
            .bearer_auth(&self.openai.api_key)
            .json(self);
        let res = req.send().await?;
        if res.status().is_success() {
            let data: ChatCompletionResponse = res.json().await?;
            Ok(data)
        } else {
            let error_response: ErrorResponse = res.json().await?;
            Err(ApiRequestError::InvalidRequestError {
                message: error_response.error.message,
                param: error_response.error.param,
                code: error_response.error.code,
            })
        }
    }
    pub async fn stream(&self) -> impl Stream<Item = ChatCompletionChunkResponse> {
        let url = format!("{}/{}", BASE_URL, API_URL);
        let mut body = serde_json::to_value(self).unwrap();
        body["stream"] = serde_json::Value::Bool(true);
        let mut es = self
            .openai
            .client
            .post(url)
            .bearer_auth(&self.openai.api_key)
            .json(&body)
            .eventsource()
            .unwrap();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(async move {
            while let Some(event) = es.next().await {
                match event {
                    Ok(Event::Message(msg)) => match msg.data.as_str() {
                        "[DONE]" => {
                            es.close();
                        }
                        "" => {}
                        data => {
                            if let Ok(json) =
                                serde_json::from_str::<ChatCompletionChunkResponse>(data)
                            {
                                tx.send(json).unwrap();
                            } else {
                                println!("err: {:#?}", msg);
                            }
                        }
                    },
                    Err(err) => {
                        println!("err: {:#?}", err);
                    }
                    _ => {}
                }
            }
        });
        tokio_stream::wrappers::UnboundedReceiverStream::new(rx)
    }
}

impl OpenAi {
    pub fn chat_completion(&self) -> ChatCompletionRequestBuilder {
        ChatCompletionRequestBuilder {
            openai: Some(self.clone()),
            ..Default::default()
        }
    }
    pub fn transcribe(&self) -> TranscribeRequest {
        TranscribeRequestBuilder {
            openai: Some(self.clone()),
            ....Default::default()
        }
    }
}

// impl TokenCount for Message {
//     fn token_count(&self) -> usize {
//         match self {
//             Message::System(message) => message.content.token_count(),
//             Message::User(message) => message.content.token_count(),
//             Message::Assistant(message) => message.content.token_count(),
//             Message::Tool(message) => message.content.token_count(),
//         }
//     }
// }

// impl TokenCount for Messages {
//     fn token_count(&self) -> usize {
//         self.0.iter().map(|m| m.token_count()).sum()
//     }
// }

#[cfg(test)]
mod test {
    #[cfg(feature = "tools")]
    use ai_tools_ox::tools::{ToTool, Tool, ToolBuilder, ToolBuilderError, ToolCallResult, Tools};
    use tokio_stream::StreamExt;

    use crate::{
        chat::{message::Messages, Message},
        OpenAiBuilder,
    };

    #[tokio::test]
    async fn test_chat_request_builder() {
        let api_key = std::env::var("OPENAI_API_KEY").unwrap();
        let client = reqwest::Client::new();
        let openai = OpenAiBuilder::default()
            .api_key(api_key)
            .client(&client)
            .build()
            .unwrap();
        let mut res = openai
            .chat_completion()
            .model("gpt-4-1106-preview")
            .stream()
            .messages(Message::user("Hi, I'm John."))
            .build()
            .unwrap()
            .stream()
            .await;
        while let Some(res) = res.next().await {
            println!("{}", String::from(res));
        }
    }

    #[cfg(feature = "tools")]
    #[tokio::test]
    async fn test_wikipedia_tool() {
        #[derive(Debug)]
        pub struct Wikipedia;

        #[async_trait::async_trait]
        impl ToTool for Wikipedia {
            fn to_tool(&self) -> Tool {
                ToolBuilder::default()
                    .name("wikipedia")
                    .description("Search in wikipedia")
                    .add_parameter::<String>("query", "Query")
                    .build()
                    .unwrap()
            }
            async fn call_tool(
                &self,
                tool_call_id: &str,
                input: serde_json::Value,
            ) -> ToolCallResult {
                dbg!(&input);
                let query = input["query"].as_str().unwrap();
                let url = format!("https://en.wikipedia.org/w/api.php?action=query&format=json&list=search&srsearch={}", query);
                let res = reqwest::get(&url)
                    .await
                    .unwrap()
                    .json::<serde_json::Value>()
                    .await
                    .unwrap();

                ToolCallResult {
                    tool_call_id: tool_call_id.to_string(),
                    content: res.to_string(),
                }
            }
        }

        let api_key = std::env::var("OPENAI_API_KEY").unwrap();
        let client = reqwest::Client::new();
        let openai = OpenAiBuilder::default()
            .api_key(api_key)
            .client(&client)
            .build()
            .unwrap();
        let tools = Tools::default().add_tool(Wikipedia);
        let mut messages = Messages::from(Message::user("Search Apollo project on Wikipedia."));
        let res = openai
            .chat_completion()
            .model("gpt-4-1106-preview")
            .tools(tools.clone())
            .messages(messages.clone())
            .build()
            .unwrap()
            .send()
            .await
            .unwrap();

        match &res.choices[0].message {
            Message::Assistant(msg) => {
                if let Some(tool_calls) = &msg.tool_calls {
                    let results = tools.call_tools(tool_calls).await;
                    let tool_msgs = Messages::from(results.clone());
                    messages.push_message(msg.clone());
                    messages.extend(tool_msgs.into_iter());
                    dbg!(&messages);
                    let res = openai
                        .chat_completion()
                        .model("gpt-4-1106-preview")
                        .tools(tools.clone())
                        .messages(messages.clone())
                        .build()
                        .unwrap()
                        .send()
                        .await
                        .unwrap();

                    dbg!(&res);
                }
            }
            _ => panic!("Not a tool call"),
        }
    }
}
