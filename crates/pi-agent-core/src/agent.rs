use anyhow::Result;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::provider::LlmClient;
use crate::skills::SkillRegistry;
use crate::tools::ToolRegistry;
use crate::types::{AgentEvent, Message, ModelConfig};

#[derive(Clone)]
pub struct PiAgent {
    pub workspace_dir: PathBuf,
    pub model_config: ModelConfig,
    pub tools: ToolRegistry,
    pub skills: SkillRegistry,
    pub client: LlmClient,
    pub max_iterations: usize,
    /// Upper bound for one delegated specialist, including provider and tools.
    pub delegation_timeout: Duration,
    pub workflow_controls: crate::workflow_control::WorkflowControls,
}

impl PiAgent {
    pub fn new(workspace_dir: impl Into<PathBuf>, model_config: ModelConfig) -> Self {
        let ws = workspace_dir.into();
        let tools = ToolRegistry::with_standard_tools(ws.clone());
        let skills = SkillRegistry::with_builtin_skills();
        Self {
            workspace_dir: ws,
            model_config,
            tools,
            skills,
            client: LlmClient::new(),
            max_iterations: 15,
            delegation_timeout: Duration::from_secs(90),
            workflow_controls: Default::default(),
        }
    }

    pub fn default_system_prompt(&self) -> String {
        let skills_appendix = self.skills.build_system_prompt_appendix();
        format!(
            "You are Pi (Volt), a high-performance, autonomous native AI coding and office assistant.\n\
            Current workspace directory: {}\n\n\
            You have access to native tools for inspecting files, editing code, searching patterns, executing shell commands, high-performance office table/document processing (`office`), and scientific computing (`calc`).\n\
            Rules:\n\
            1. Analyze the context before editing. Use read_file to inspect target files.\n\
            2. When modifying files, prefer edit_file for surgical edits, or write_file for new files.\n\
            3. Run shell commands using bash when necessary to build, test, or verify your changes.\n\
            4. For table queries, CSV/TSV processing, Markdown tables, docx extraction, slide generation, and reports, use the native `office` tool.\n\
            5. For mathematical expressions, statistical analysis, linear regression, matrix operations, loan PMT, CAGR, unit conversions, and dates, use the native `calc` tool.\n\
            6. Keep responses concise, accurate, and focused on the solution.\n\
            7. Proceed step-by-step until the task is completely finished.\n\
            {}",
            self.workspace_dir.display(),
            skills_appendix
        )
    }

    pub async fn run_turn(
        &self,
        messages: &mut Vec<Message>,
        event_tx: mpsc::Sender<AgentEvent>,
    ) -> Result<()> {
        self.run_turn_inner(messages, event_tx, true).await
    }

    // Boxing gives the parent/child call graph a finite future type. Children
    // receive a restricted registry and cannot recursively delegate.
    pub(crate) fn run_turn_inner<'a>(
        &'a self,
        messages: &'a mut Vec<Message>,
        event_tx: mpsc::Sender<AgentEvent>,
        allow_delegation: bool,
    ) -> futures::future::BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            if messages.is_empty() || messages[0].role != crate::types::Role::System {
                messages.insert(0, Message::system(self.default_system_prompt()));
            }

            let mut tools_schema = self.tools.to_schema_json();
            if allow_delegation {
                tools_schema
                    .as_array_mut()
                    .expect("tool schema array")
                    .push(crate::delegation::schema());
            }

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

                    let output_res = if allow_delegation && tc.name == crate::delegation::TOOL_NAME
                    {
                        crate::delegation::execute(self, &tc, messages, &event_tx).await
                    } else if let Some(tool) = self.tools.get(&tc.name) {
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
        })
    }
}
