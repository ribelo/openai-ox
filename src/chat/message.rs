use std::{
    fmt,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use bon::{builder, Builder};
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Text {
    pub text: String,
}

impl Text {
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl<T: Into<String>> From<T> for Text {
    fn from(text: T) -> Self {
        Text { text: text.into() }
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.text)
    }
}

pub trait ChatMessage: Serialize + DeserializeOwned {
    fn role(&self) -> Role;
    fn content(&self) -> impl IntoIterator<Item = &MultimodalContent>;
    fn content_mut(&mut self) -> impl IntoIterator<Item = &mut MultimodalContent>;
    fn push_content(&mut self, content: impl Into<MultimodalContent>);
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
}

#[derive(Debug, Serialize, Deserialize, derive_more::Display, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MultimodalContent {
    Text(Text),
    // Image(Image),
    // ToolUse(ToolUse),
    // ToolResult(ToolResult),
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[serde(rename_all = "lowercase")]
pub struct SystemMessage {
    role: Role,
    #[builder(into)]
    content: Vec<MultimodalContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

impl ChatMessage for SystemMessage {
    fn role(&self) -> Role {
        self.role
    }

    fn content(&self) -> impl IntoIterator<Item = &MultimodalContent> {
        &self.content
    }

    fn content_mut(&mut self) -> impl IntoIterator<Item = &mut MultimodalContent> {
        &mut self.content
    }

    fn push_content(&mut self, content: impl Into<MultimodalContent>) {
        self.content.push(content.into());
    }

    fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    fn len(&self) -> usize {
        self.content.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct UserMessage {
    role: Role,
    #[builder(into)]
    content: Vec<MultimodalContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

impl ChatMessage for UserMessage {
    fn role(&self) -> Role {
        self.role
    }

    fn content(&self) -> impl IntoIterator<Item = &MultimodalContent> {
        &self.content
    }

    fn content_mut(&mut self) -> impl IntoIterator<Item = &mut MultimodalContent> {
        &mut self.content
    }

    fn push_content(&mut self, content: impl Into<MultimodalContent>) {
        self.content.push(content.into());
    }

    fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    fn len(&self) -> usize {
        self.content.len()
    }
}

impl UserMessage {
    pub fn new<T, U>(content: T) -> Self
    where
        T: IntoIterator<Item = U>,
        U: Into<MultimodalContent>,
    {
        Self {
            role: Role::User,
            content: content.into_iter().map(Into::into).collect(),
            name: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct AssistantMessage {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(into)]
    pub content: Vec<MultimodalContent>,
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
    pub content: Vec<MultimodalContent>,
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
    #[must_use]
    pub fn role(&self) -> Role {
        match self {
            Message::System(_) => Role::System,
            Message::User(_) => Role::User,
            Message::Assistant(_) => Role::Assistant,
            Message::Tool(_) => Role::Tool,
        }
    }

    pub fn content(&self) -> impl IntoIterator<Item = &MultimodalContent> {
        match self {
            Message::System(msg) => &msg.content,
            Message::User(msg) => &msg.content,
            Message::Assistant(msg) => &msg.content,
            Message::Tool(msg) => &msg.content,
        }
    }

    pub fn content_mut(&mut self) -> impl IntoIterator<Item = &mut MultimodalContent> {
        match self {
            Message::System(msg) => &mut msg.content,
            Message::User(msg) => &mut msg.content,
            Message::Assistant(msg) => &mut msg.content,
            Message::Tool(msg) => &mut msg.content,
        }
    }

    pub fn push_content(&mut self, content: impl Into<MultimodalContent>) {
        match self {
            Message::System(_) => {}
            Message::User(msg) => msg.content = content.into(),
            Message::Assistant(msg) => msg.content = Some(content.into()),
            Message::Tool(_) => {}
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            Message::System(_) => false,
            Message::User(_) => false,
            Message::Assistant(msg) => msg.content.is_none(),
            Message::Tool(_) => false,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Message::System(_) => 1,
            Message::User(_) => 1,
            Message::Assistant(msg) => {
                if msg.content.is_some() {
                    1
                } else {
                    0
                }
            }
            Message::Tool(_) => 1,
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
