use anyhow::{Context, Result};
use futures::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use tokio::sync::mpsc;

use crate::types::{AgentEvent, Message, ModelConfig, Role, ToolCall};

pub struct LlmClient {
    client: reqwest::Client,
}

impl LlmClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .connect_timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    pub async fn stream_chat(
        &self,
        config: &ModelConfig,
        messages: &[Message],
        tools: Option<Value>,
        tx: mpsc::Sender<AgentEvent>,
    ) -> Result<Message> {
        let base = config.api_base.as_deref().unwrap_or("https://api.openai.com/v1");
        let url = format!("{}/chat/completions", base.trim_end_matches('/'));

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if let Some(key) = &config.api_key {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", key)) {
                headers.insert(AUTHORIZATION, val);
            }
        }

        let api_messages: Vec<Value> = messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    Role::System => "system",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                    Role::Tool => "tool",
                };
                let mut obj = json!({
                    "role": role,
                    "content": m.content,
                });
                if let Some(tool_calls) = &m.tool_calls {
                    let tc_val: Vec<Value> = tool_calls
                        .iter()
                        .map(|tc| {
                            json!({
                                "id": tc.id,
                                "type": "function",
                                "function": {
                                    "name": tc.name,
                                    "arguments": tc.arguments.to_string(),
                                }
                            })
                        })
                        .collect();
                    obj["tool_calls"] = Value::Array(tc_val);
                }
                if let Some(tr) = &m.tool_result {
                    obj["tool_call_id"] = json!(tr.tool_call_id);
                }
                obj
            })
            .collect();

        let mut body = json!({
            "model": config.model,
            "messages": api_messages,
            "stream": true,
            "temperature": config.temperature,
        });

        if let Some(max_tokens) = config.max_tokens {
            body["max_tokens"] = json!(max_tokens);
        }

        if let Some(tools_schema) = tools {
            if tools_schema.as_array().map(|a| !a.is_empty()).unwrap_or(false) {
                body["tools"] = tools_schema;
            }
        }

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .with_context(|| format!("Failed to send request to {}", url))?;

        if !response.status().is_success() {
            let status = response.status();
            let err_text = response.text().await.unwrap_or_default();
            let err_msg = format!("LLM API returned error {}: {}", status, err_text);
            let _ = tx.send(AgentEvent::Error(err_msg.clone())).await;
            anyhow::bail!(err_msg);
        }

        let mut stream = response.bytes_stream();
        let mut full_text = String::new();
        let mut buffer = String::new();

        // Accumulate tool calls: index -> (id, name, args_buf)
        let mut pending_tool_calls: std::collections::BTreeMap<usize, (String, String, String)> =
            std::collections::BTreeMap::new();

        while let Some(chunk_res) = stream.next().await {
            let chunk = chunk_res?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].trim().to_string();
                buffer.drain(..=pos);

                if line.is_empty() || line.starts_with(':') {
                    continue;
                }

                if let Some(data) = line.strip_prefix("data: ") {
                    let data = data.trim();
                    if data == "[DONE]" {
                        break;
                    }

                    if let Ok(val) = serde_json::from_str::<Value>(data) {
                        if let Some(choices) = val.get("choices").and_then(|c| c.as_array()) {
                            if let Some(first) = choices.first() {
                                if let Some(delta) = first.get("delta") {
                                    // Handle content delta
                                    if let Some(content_chunk) = delta.get("content").and_then(|c| c.as_str()) {
                                        if !content_chunk.is_empty() {
                                            full_text.push_str(content_chunk);
                                            let _ = tx.send(AgentEvent::TextDelta(content_chunk.to_string())).await;
                                        }
                                    }

                                    // Handle streaming tool calls
                                    if let Some(tc_list) = delta.get("tool_calls").and_then(|t| t.as_array()) {
                                        for tc in tc_list {
                                            let index = tc.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;
                                            let entry = pending_tool_calls.entry(index).or_insert_with(|| (String::new(), String::new(), String::new()));

                                            if let Some(id) = tc.get("id").and_then(|s| s.as_str()) {
                                                entry.0.push_str(id);
                                            }
                                            if let Some(func) = tc.get("function") {
                                                if let Some(name) = func.get("name").and_then(|s| s.as_str()) {
                                                    entry.1.push_str(name);
                                                }
                                                if let Some(args_chunk) = func.get("arguments").and_then(|s| s.as_str()) {
                                                    entry.2.push_str(args_chunk);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut final_tool_calls = Vec::new();
        for (_, (id, name, args_str)) in pending_tool_calls {
            let parsed_args = serde_json::from_str::<Value>(&args_str).unwrap_or_else(|_| json!({ "raw": args_str }));
            let tc = ToolCall {
                id,
                name,
                arguments: parsed_args,
            };
            final_tool_calls.push(tc);
        }

        let mut assistant_msg = Message::assistant(full_text);
        if !final_tool_calls.is_empty() {
            assistant_msg.tool_calls = Some(final_tool_calls);
        }

        let _ = tx.send(AgentEvent::Completed { total_tokens: None }).await;
        Ok(assistant_msg)
    }
}

impl Default for LlmClient {
    fn default() -> Self {
        Self::new()
    }
}
