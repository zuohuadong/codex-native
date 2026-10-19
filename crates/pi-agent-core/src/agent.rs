use std::path::PathBuf;
use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::provider::LlmClient;
use crate::tools::ToolRegistry;
use crate::types::{AgentEvent, Message, ModelConfig};

pub struct PiAgent {
    pub workspace_dir: PathBuf,
    pub model_config: ModelConfig,
    pub tools: ToolRegistry,
    pub client: LlmClient,
    pub max_iterations: usize,
}

impl PiAgent {
    pub fn new(workspace_dir: impl Into<PathBuf>, model_config: ModelConfig) -> Self {
        let ws = workspace_dir.into();
        let tools = ToolRegistry::with_standard_tools(ws.clone());
        Self {
            workspace_dir: ws,
            model_config,
            tools,
            client: LlmClient::new(),
            max_iterations: 15,
        }
    }

    pub fn default_system_prompt(&self) -> String {
        format!(
            "You are Pi, a high-performance, autonomous native AI coding agent.\n\
            Current workspace directory: {}\n\n\
            You have access to tools for inspecting files, editing code, searching patterns, and executing shell commands.\n\
            Rules:\n\
            1. Analyze the context before editing. Use read_file to inspect target files.\n\
            2. When modifying files, prefer edit_file for surgical edits, or write_file for new files.\n\
            3. Run shell commands using bash when necessary to build, test, or verify your changes.\n\
            4. Keep responses concise and focused on the solution.\n\
            5. Proceed step-by-step until the task is completely finished.",
            self.workspace_dir.display()
        )
    }

    pub async fn run_turn(
        &self,
        messages: &mut Vec<Message>,
        event_tx: mpsc::Sender<AgentEvent>,
    ) -> Result<()> {
        if messages.is_empty() || messages[0].role != crate::types::Role::System {
            messages.insert(0, Message::system(self.default_system_prompt()));
        }

        let tools_schema = self.tools.to_schema_json();

        for iter in 0..self.max_iterations {
            info!("Pi agent iteration {}/{}", iter + 1, self.max_iterations);

            let assistant_msg = self
                .client
                .stream_chat(
                    &self.model_config,
                    messages,
                    Some(tools_schema.clone()),
                    event_tx.clone(),
                )
                .await?;

            messages.push(assistant_msg.clone());

            let tool_calls = match &assistant_msg.tool_calls {
                Some(calls) if !calls.is_empty() => calls.clone(),
                _ => {
                    // No more tool calls; agent has completed its turn
                    info!("Agent completed turn without pending tool calls");
                    return Ok(());
                }
            };

            for tc in tool_calls {
                let _ = event_tx
                    .send(AgentEvent::ToolCallStarted {
                        id: tc.id.clone(),
                        name: tc.name.clone(),
                        arguments: tc.arguments.clone(),
                    })
                    .await;

                let output_res = if let Some(tool) = self.tools.get(&tc.name) {
                    tool.execute(tc.arguments.clone()).await
                } else {
                    Err(anyhow::anyhow!("Unknown tool: {}", tc.name))
                };

                let (output, is_error) = match output_res {
                    Ok(out) => (out, false),
                    Err(err) => {
                        error!("Tool {} execution failed: {:?}", tc.name, err);
                        (format!("Error: {:#}", err), true)
                    }
                };

                let _ = event_tx
                    .send(AgentEvent::ToolCallFinished {
                        id: tc.id.clone(),
                        name: tc.name.clone(),
                        output: output.clone(),
                        is_error,
                    })
                    .await;

                let tool_msg = Message::tool_result(tc.id, tc.name, output, is_error);
                messages.push(tool_msg);
            }
        }

        anyhow::bail!("Reached maximum iteration limit of {}", self.max_iterations);
    }
}
