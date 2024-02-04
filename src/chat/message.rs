#[cfg(feature = "tools")]
use ai_tools_ox::tools::{ToolCall, ToolType, ToolsResults};
use derivative::Derivative;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    #[cfg(feature = "tools")]
    Tool,
}

#[derive(Deserialize)]
struct HelperMessage {
    content: Option<String>,
    name: Option<String>,
    #[cfg(feature = "tools")]
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Serialize, Derivative, Clone)]
#[derivative(Default)]
#[serde(rename_all = "lowercase")]
pub struct SystemMessage {
    #[derivative(Default(value = "Role::System"))]
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl SystemMessage {
    pub fn new(content: impl ToString) -> Self {
        Self {
            role: Role::System,
            content: content.to_string(),
            name: None,
        }
    }
}

impl<'de> Deserialize<'de> for SystemMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let helper = HelperMessage::deserialize(deserializer)?;
        Ok(SystemMessage {
            role: Role::System,
            content: helper.content.unwrap(),
            name: helper.name,
        })
    }
}

impl From<SystemMessage> for String {
    fn from(message: SystemMessage) -> Self {
        message.content
    }
}

#[derive(Debug, Serialize, Derivative, Clone)]
#[derivative(Default)]
pub struct UserMessage {
    #[derivative(Default(value = "Role::User"))]
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl UserMessage {
    pub fn new(content: impl ToString) -> Self {
        Self {
            role: Role::User,
            content: content.to_string(),
            name: None,
        }
    }
}

impl<'de> Deserialize<'de> for UserMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let helper = HelperMessage::deserialize(deserializer)?;
        Ok(UserMessage {
            role: Role::User,
            content: helper.content.unwrap(),
            name: helper.name,
        })
    }
}

impl From<UserMessage> for String {
    fn from(message: UserMessage) -> Self {
        message.content
    }
}

#[derive(Debug, Serialize, Derivative, Clone)]
pub struct AssistantMessage {
    #[derivative(Default(value = "Role::Assistant"))]
    pub role: Role,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(feature = "tools")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl<'de> Deserialize<'de> for AssistantMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let helper = HelperMessage::deserialize(deserializer)?;
        Ok(AssistantMessage {
            role: Role::Assistant,
            content: helper.content,
            name: helper.name,
            #[cfg(feature = "tools")]
            tool_calls: helper.tool_calls,
        })
    }
}

impl AssistantMessage {
    pub fn new(content: impl ToString) -> Self {
        Self {
            role: Role::Assistant,
            content: Some(content.to_string()),
            name: None,
            #[cfg(feature = "tools")]
            tool_calls: None,
        }
    }
}

#[cfg(feature = "tools")]
#[derive(Debug, Serialize, Deserialize, Clone, Derivative)]
pub struct ToolMessage {
    #[derivative(Default(value = "Role::Tool"))]
    pub role: Role,
    pub content: String,
    pub tool_call_id: String,
}

#[cfg(feature = "tools")]
impl From<ToolsResults> for Messages {
    fn from(results: ToolsResults) -> Self {
        let mut messages = Messages::default();
        for result in results.0 {
            let msg = Message::Tool(ToolMessage {
                role: Role::Tool,
                content: result.content,
                tool_call_id: result.tool_call_id,
            });
            messages.push_message(msg);
        }
        messages
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "role")]
#[serde(rename_all = "lowercase")]
pub enum Message {
    System(SystemMessage),
    User(UserMessage),
    Assistant(AssistantMessage),
    #[cfg(feature = "tools")]
    Tool(ToolMessage),
}

impl Message {
    pub fn system(content: impl ToString) -> Self {
        Message::System(SystemMessage::new(content))
    }
    pub fn user(content: impl ToString) -> Self {
        Message::User(UserMessage::new(content))
    }
    pub fn assistant(content: impl ToString) -> Self {
        Message::Assistant(AssistantMessage::new(content))
    }
    pub fn content(&self) -> Option<String> {
        match self {
            Message::System(msg) => Some(msg.content.clone()),
            Message::User(msg) => Some(msg.content.clone()),
            Message::Assistant(msg) => msg.content.as_ref().cloned(),
            #[cfg(feature = "tools")]
            Message::Tool(msg) => Some(msg.content.clone()),
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

impl Messages {
    pub fn push_message(&mut self, message: impl Into<Message>) {
        self.0.push(message.into());
    }
}

impl From<Message> for Messages {
    fn from(value: Message) -> Self {
        Messages(vec![value])
    }
}

impl<T> Extend<T> for Messages
where
    T: Into<Message>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        self.0.extend(iter.into_iter().map(|item| item.into()));
    }
}

impl IntoIterator for Messages {
    type Item = Message;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
