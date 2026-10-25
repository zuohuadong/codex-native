use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub name: String,
    pub output: String,
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub reasoning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_result: Option<ToolResult>,
    pub timestamp: i64,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            id: uuid_simple(),
            role: Role::System,
            content: content.into(),
            reasoning: None,
            tool_calls: None,
            tool_result: None,
            timestamp: now_ms(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            id: uuid_simple(),
            role: Role::User,
            content: content.into(),
            reasoning: None,
            tool_calls: None,
            tool_result: None,
            timestamp: now_ms(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            id: uuid_simple(),
            role: Role::Assistant,
            content: content.into(),
            reasoning: None,
            tool_calls: None,
            tool_result: None,
            timestamp: now_ms(),
        }
    }

    pub fn tool_result(
        tool_call_id: impl Into<String>,
        name: impl Into<String>,
        output: impl Into<String>,
        is_error: bool,
    ) -> Self {
        let output_str = output.into();
        Self {
            id: uuid_simple(),
            role: Role::Tool,
            content: output_str.clone(),
            reasoning: None,
            tool_calls: None,
            tool_result: Some(ToolResult {
                tool_call_id: tool_call_id.into(),
                name: name.into(),
                output: output_str,
                is_error,
            }),
            timestamp: now_ms(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: String,
    pub model: String,
    pub api_base: Option<String>,
    pub api_key: Option<String>,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        let api_key = std::env::var("SENSENOVA_API_KEY")
            .or_else(|_| std::env::var("OPENAI_API_KEY"))
            .ok();
        Self {
            provider: "openai-compatible".into(),
            model: "sensenova-6.8-flash-lite".into(),
            api_base: Some("https://token.sensenova.cn/v1".into()),
            api_key,
            temperature: 0.2,
            max_tokens: Some(8192),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    WorkflowUpdated(crate::workflow::WorkflowSnapshot),
    Thought(String),
    TextDelta(String),
    ToolCallStarted {
        id: String,
        name: String,
        arguments: serde_json::Value,
    },
    ToolCallFinished {
        id: String,
        name: String,
        output: String,
        is_error: bool,
    },
    Completed {
        total_tokens: Option<u64>,
    },
    Error(String),
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("msg-{:x}", now)
}
