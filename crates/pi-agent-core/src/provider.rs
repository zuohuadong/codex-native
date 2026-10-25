use anyhow::{Context, Result};
use futures::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use tokio::sync::mpsc;

use crate::types::{AgentEvent, Message, ModelConfig, Role, ToolCall};

#[derive(Clone)]
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
        let base = config
            .api_base
            .as_deref()
            .unwrap_or("https://api.openai.com/v1");
        let trimmed = base.trim_end_matches('/');
        let url = if trimmed.ends_with("/chat/completions") {
            trimmed.to_string()
        } else {
            format!("{}/chat/completions", trimmed)
        };

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if let Some(key) = &config.api_key {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", key.trim())) {
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
            if tools_schema
                .as_array()
                .map(|a| !a.is_empty())
                .unwrap_or(false)
            {
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
        let mut full_reasoning = String::new();
        let mut total_tokens = None;
        let mut buffer = Vec::new();
        let mut finished = false;

        // Accumulate tool calls: index -> (id, name, args_buf)
        let mut pending_tool_calls: std::collections::BTreeMap<usize, (String, String, String)> =
            std::collections::BTreeMap::new();

        'streaming: loop {
            let Some(chunk_res) =
                tokio::time::timeout(std::time::Duration::from_secs(60), stream.next())
                    .await
                    .context("Stream stalled for 60 seconds; output may be incomplete")?
            else {
                break;
            };
            buffer.extend_from_slice(&chunk_res?);
            if buffer.len() > 4 * 1024 * 1024 {
                anyhow::bail!("Provider event exceeded 4 MiB");
            }
            while let Some(pos) = buffer.iter().position(|byte| *byte == b'\n') {
                let line = String::from_utf8(buffer.drain(..=pos).collect())
                    .context("Provider returned invalid UTF-8")?;
                let line = line.trim();

                if line.is_empty() || line.starts_with(':') {
                    continue;
                }

                if let Some(data) = line.strip_prefix("data:") {
                    let data = data.trim();
                    if data == "[DONE]" {
                        finished = true;
                        break 'streaming;
                    }

                    {
                        let val: Value =
                            serde_json::from_str(data).context("Malformed provider event")?;
                        validate_event_schema(&val)?;
                        if val.get("error").is_some() {
                            anyhow::bail!("Provider reported a streaming error");
                        }
                        // Extract usage metrics if returned by the provider
                        if let Some(usage) = val.get("usage") {
                            if let Some(tokens) = usage.get("total_tokens").and_then(|t| t.as_u64())
                            {
                                total_tokens = Some(tokens);
                            }
                        }

                        if let Some(choices) = val.get("choices").and_then(|c| c.as_array()) {
                            if let Some(first) = choices.first() {
                                if let Some(reason) = first
                                    .get("finish_reason")
                                    .filter(|reason| !reason.is_null())
                                {
                                    let Some(reason) = reason.as_str() else {
                                        anyhow::bail!("Invalid finish_reason schema");
                                    };
                                    if !matches!(reason, "stop" | "tool_calls") {
                                        anyhow::bail!(
                                            "Provider did not complete the response: {reason}"
                                        );
                                    }
                                    finished = true;
                                }
                                if let Some(delta) = first.get("delta") {
                                    // Handle reasoning / thought delta (e.g. DeepSeek R1 / SenseNova reasoning_content)
                                    if let Some(reasoning_chunk) = delta
                                        .get("reasoning_content")
                                        .or_else(|| delta.get("reasoning"))
                                        .and_then(|c| c.as_str())
                                    {
                                        if !reasoning_chunk.is_empty() {
                                            full_reasoning.push_str(reasoning_chunk);
                                            let _ = tx
                                                .send(AgentEvent::Thought(
                                                    reasoning_chunk.to_string(),
                                                ))
                                                .await;
                                        }
                                    }

                                    // Handle content delta
                                    if let Some(content_chunk) =
                                        delta.get("content").and_then(|c| c.as_str())
                                    {
                                        if !content_chunk.is_empty() {
                                            full_text.push_str(content_chunk);
                                            let _ = tx
                                                .send(AgentEvent::TextDelta(
                                                    content_chunk.to_string(),
                                                ))
                                                .await;
                                        }
                                    }

                                    // Handle streaming tool calls
                                    if let Some(tc_list) =
                                        delta.get("tool_calls").and_then(|t| t.as_array())
                                    {
                                        for tc in tc_list {
                                            let index = tc
                                                .get("index")
                                                .and_then(|i| i.as_u64())
                                                .unwrap_or(0)
                                                as usize;
                                            let entry = pending_tool_calls
                                                .entry(index)
                                                .or_insert_with(|| {
                                                    (String::new(), String::new(), String::new())
                                                });

                                            if let Some(id) = tc.get("id").and_then(|s| s.as_str())
                                            {
                                                entry.0.push_str(id);
                                            }
                                            if let Some(func) = tc.get("function") {
                                                if let Some(name) =
                                                    func.get("name").and_then(|s| s.as_str())
                                                {
                                                    entry.1.push_str(name);
                                                }
                                                if let Some(args_chunk) =
                                                    func.get("arguments").and_then(|s| s.as_str())
                                                {
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

        if !finished {
            anyhow::bail!("Provider disconnected before completion; partial output retained");
        }
        let mut final_tool_calls = Vec::new();
        for (_, (id, name, args_str)) in pending_tool_calls {
            if id.is_empty() || name.is_empty() {
                anyhow::bail!("Incomplete tool call identity");
            }
            let parsed_args =
                serde_json::from_str::<Value>(&args_str).context("Invalid tool arguments")?;
            let tc = ToolCall {
                id,
                name,
                arguments: parsed_args,
            };
            final_tool_calls.push(tc);
        }

        let mut assistant_msg = Message::assistant(full_text);
        if !full_reasoning.is_empty() {
            assistant_msg.reasoning = Some(full_reasoning);
        }
        if !final_tool_calls.is_empty() {
            assistant_msg.tool_calls = Some(final_tool_calls);
        }

        let _ = tx.send(AgentEvent::Completed { total_tokens }).await;
        Ok(assistant_msg)
    }
}

impl Default for LlmClient {
    fn default() -> Self {
        Self::new()
    }
}

fn validate_event_schema(event: &Value) -> Result<()> {
    if event.get("error").is_some() {
        return Ok(());
    }
    if let Some(choices) = event.get("choices") {
        let choices = choices.as_array().context("Invalid choices schema")?;
        for choice in choices {
            if !choice.is_object() {
                anyhow::bail!("Invalid choice schema");
            }
            if let Some(delta) = choice.get("delta") {
                if !delta.is_object() {
                    anyhow::bail!("Invalid delta schema");
                }
            }
            if choice.get("delta").is_none() && choice.get("finish_reason").is_none() {
                anyhow::bail!("Choice is missing delta and finish_reason");
            }
        }
    } else if !event.get("usage").is_some_and(Value::is_object) {
        anyhow::bail!("Provider event is missing choices and usage");
    }
    Ok(())
}
