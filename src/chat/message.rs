use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use bon::{builder, Builder};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[serde(rename_all = "lowercase")]
pub struct SystemMessage {
    #[builder(into)]
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl From<String> for SystemMessage {
    fn from(content: String) -> Self {
        SystemMessage::builder().content(content).build()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct UserMessage {
    #[builder(into)]
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl From<String> for UserMessage {
    fn from(content: String) -> Self {
        UserMessage::builder().content(content).build()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct AssistantMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct ToolMessage {
    #[builder(into)]
    pub content: String,
    pub tool_call_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "role")]
#[serde(rename_all = "lowercase")]
pub enum Message {
    System(SystemMessage),
    User(UserMessage),
    Assistant(AssistantMessage),
    Tool(ToolMessage),
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Message::System(SystemMessage::from(content.into()))
    }
    pub fn user(content: impl Into<String>) -> Self {
        Message::User(UserMessage::from(content.into()))
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Message::Assistant(AssistantMessage::builder().content(content.into()).build())
    }
    pub fn content(&self) -> Option<&str> {
        match self {
            Message::System(msg) => Some(&msg.content),
            Message::User(msg) => Some(&msg.content),
            Message::Assistant(msg) => msg.content.as_deref(),
            Message::Tool(msg) => Some(&msg.content),
        }
    }
}

impl From<SystemMessage> for Message {
    fn from(message: SystemMessage) -> Self {
        Message::System(message)
    }
}

impl From<UserMessage> for Message {
    fn from(message: UserMessage) -> Self {
        Message::User(message)
    }
}

impl From<AssistantMessage> for Message {
    fn from(message: AssistantMessage) -> Self {
        Message::Assistant(message)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Messages(pub Vec<Message>);

impl Deref for Messages {
    type Target = Vec<Message>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Messages {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Message> for Messages {
    fn from(value: Message) -> Self {
        Messages(vec![value])
    }
}

impl IntoIterator for Messages {
    type Item = Message;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::chat::message::UserMessage;

    use super::{AssistantMessage, Message, SystemMessage, ToolMessage};

    #[test]
    fn test_assistant_message_deserialization() {
        let json = json!({
            "content": "Hello John! How can I assist you today?",
            "refusal": null,
            "role": "assistant"
        });

        let msg: AssistantMessage = serde_json::from_value(json).unwrap();
        assert_eq!(
            msg.content.unwrap(),
            "Hello John! How can I assist you today?"
        );
        assert!(msg.refusal.is_none());
    }

    #[test]
    fn test_system_message_deserialization() {
        let json = json!({
            "content": "You are a helpful assistant",
            "role": "system"
        });

        let msg: SystemMessage = serde_json::from_value(json).unwrap();
        assert_eq!(msg.content, "You are a helpful assistant");
    }

    #[test]
    fn test_user_message_deserialization() {
        let json = json!({
            "content": "What is the weather?",
            "role": "user"
        });

        let msg: UserMessage = serde_json::from_value(json).unwrap();
        assert_eq!(msg.content, "What is the weather?");
    }

    #[test]
    fn test_tool_message_deserialization() {
        let json = json!({
            "content": "The temperature is 72F",
            "role": "tool",
            "tool_call_id": "weather_123"
        });

        let msg: ToolMessage = serde_json::from_value(json).unwrap();
        assert_eq!(msg.content, "The temperature is 72F");
        assert_eq!(msg.tool_call_id, "weather_123");
    }

    #[test]
    fn test_message_deserialization() {
        let json = json!({
            "content": "Hello John! How can I assist you today?",
            "refusal": null,
            "role": "assistant"
        });

        let msg: Message = serde_json::from_value(json).unwrap();
        match msg {
            Message::Assistant(assistant_msg) => {
                assert_eq!(
                    assistant_msg.content.unwrap(),
                    "Hello John! How can I assist you today?"
                );
                assert!(assistant_msg.refusal.is_none());
            }
            _ => panic!("Expected assistant message"),
        }
    }
}
