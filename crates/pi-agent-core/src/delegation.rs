use crate::{
    agent::PiAgent,
    types::{AgentEvent, Message, ToolCall},
    workflow::{Node, NodeKind, Stage, StageProgress, Status, WorkflowSnapshot},
};
use anyhow::{bail, Result};
use futures::{stream::FuturesUnordered, StreamExt};
use serde_json::{json, Value};
use tokio::sync::mpsc;

pub const TOOL_NAME: &str = "delegate_agents";

pub fn schema() -> Value {
    json!({
        "type": "function",
        "function": {
            "name": TOOL_NAME,
            "description": "Run up to three independent read-only analysis tasks in parallel. Give each task a self-contained prompt including the relevant context and constraints; workers do not inherit the conversation. Use this when the user's request benefits from separate investigations. The parent agent remains responsible for edits and final decisions.",
            "parameters": {
                "type": "object",
                "properties": {
                    "title": {"type": "string", "description": "Short workflow title"},
                    "tasks": {
                        "type": "array", "minItems": 1, "maxItems": 3,
                        "items": {"type": "object", "properties": {
                            "label": {"type": "string"},
                            "prompt": {"type": "string"}
                        }, "required": ["label", "prompt"]}
                    }
                },
                "required": ["tasks"]
            }
        }
    })
}

pub async fn execute(
    parent: &PiAgent,
    call: &ToolCall,
    messages: &[Message],
    event_tx: &mpsc::Sender<AgentEvent>,
) -> Result<String> {
    let tasks = call
        .arguments
        .get("tasks")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("delegate_agents requires a tasks array"))?;
    if tasks.is_empty() || tasks.len() > 3 {
        bail!("delegate_agents accepts between 1 and 3 tasks");
    }

    let task_data: Vec<(String, String)> = tasks
        .iter()
        .enumerate()
        .map(|(index, task)| {
            let label = task
                .get("label")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| if index == 0 { "Analysis" } else { "Worker" })
                .trim()
                .chars()
                .take(80)
                .collect::<String>();
            let prompt = task
                .get("prompt")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("delegate_agents task {index} is missing prompt"))?
                .trim()
                .to_string();
            if prompt.is_empty() {
                bail!("delegate_agents task {index} has an empty prompt");
            }
            Ok((label, prompt))
        })
        .collect::<Result<_>>()?;

    let parent_message_id = messages
        .iter()
        .rev()
        .find(|message| message.role == crate::types::Role::User)
        .map(|message| message.id.clone())
        .ok_or_else(|| anyhow::anyhow!("delegate_agents requires a user message"))?;
    let workflow_id = format!("workflow-{}", unique_suffix());
    let title = call
        .arguments
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("Parallel analysis")
        .trim()
        .chars()
        .take(120)
        .collect::<String>();
    let mut snapshot = WorkflowSnapshot {
        id: workflow_id,
        revision: 1,
        parent_message_id,
        title,
        model: parent.model_config.model.clone(),
        status: Status::Running,
        stages: vec![Stage {
            id: "analysis".into(),
            label: "Parallel analysis".into(),
            status: Status::Running,
            progress: Some(StageProgress {
                settled: 0,
                observed: task_data.len() as u64,
            }),
            nodes: task_data
                .iter()
                .enumerate()
                .map(|(index, (label, _))| Node {
                    id: format!("agent-{index}"),
                    label: label.clone(),
                    kind: NodeKind::Agent,
                    avatar_index: Some(index as i32),
                    status: Status::Running,
                    detail: String::new(),
                })
                .collect(),
        }],
        delivery: String::new(),
        changes: Vec::new(),
    };
    let (_registration, mut cancellation) = parent.workflow_controls.register(snapshot.id.clone());
    emit(event_tx, &snapshot).await;

    // Futures are owned by this invocation. Dropping the parent's run drops
    // every in-flight request; no detached worker or channel-drain survives it.
    let mut children = FuturesUnordered::new();
    let (updates_tx, mut updates_rx) = mpsc::channel(64);
    for (index, (_, prompt)) in task_data.iter().enumerate() {
        let mut child = parent.clone();
        child.tools = read_only_tools(parent);
        child.max_iterations = parent.max_iterations.min(6);
        let prompt = prompt.clone();
        let updates_tx = updates_tx.clone();
        children.push(async move {
            let mut child_messages = vec![Message::system(
                "You are a read-only specialist working for a parent coding agent. Inspect and reason about the workspace, but never edit files, run destructive commands, or delegate further. Return concise evidence and recommendations to the parent.",
            ), Message::user(prompt)];
            let (child_tx, mut child_rx) = mpsc::channel(32);
            let result = {
                let mut run = child.run_turn_inner(&mut child_messages, child_tx, false);
                let deadline = tokio::time::sleep(parent.delegation_timeout);
                tokio::pin!(deadline);
                loop {
                    tokio::select! {
                        result = &mut run => break result,
                        _ = &mut deadline => break Err(anyhow::anyhow!("specialist exceeded {:?}", parent.delegation_timeout)),
                        Some(event) = child_rx.recv() => {
                            if matches!(&event, AgentEvent::TextDelta(_) | AgentEvent::ToolCallStarted { .. }) {
                                let _ = updates_tx.send((index, event)).await;
                            }
                        }
                    }
                }
            }.map(|_| {
                    child_messages
                        .iter()
                        .rev()
                        .find(|message| message.role == crate::types::Role::Assistant)
                        .map(|message| message.content.clone())
                        .unwrap_or_else(|| "No conclusion returned.".into())
                });
            (index, result)
        });
    }
    drop(updates_tx);

    let mut reports = vec![String::new(); task_data.len()];
    let mut failed = false;
    let mut interrupted = false;
    while !children.is_empty() {
        let (index, result) = tokio::select! {
            biased;
            _ = cancellation.changed() => {
                // Drop in-flight HTTP/tool futures before publishing interruption.
                children.clear();
                snapshot.interrupt();
                for (index, node) in snapshot.stages[0].nodes.iter().enumerate() {
                    if node.status == Status::Interrupted {
                        reports[index] = "Stopped by user; no final result was produced.".into();
                    }
                }
                interrupted = true;
                break;
            }
            Some(update) = children.next() => update,
            Some((index, event)) = updates_rx.recv() => {
                let node = &mut snapshot.stages[0].nodes[index];
                if node.status == Status::Running {
                    match event {
                        AgentEvent::TextDelta(text) => {
                            node.detail.push_str(&text);
                            node.detail = preview(&node.detail);
                        }
                        AgentEvent::ToolCallStarted { name, .. } => {
                            node.detail.push_str(&format!("\nUsing {name}\n"));
                            node.detail = preview(&node.detail);
                        }
                        _ => {}
                    }
                    snapshot.revision += 1;
                    emit(event_tx, &snapshot).await;
                }
                continue;
            }
        };
        match result {
            Ok(report) => {
                reports[index] = report.clone();
                snapshot.stages[0].nodes[index].status = Status::Completed;
                snapshot.stages[0].nodes[index].detail = preview(&report);
            }
            Err(error) => {
                failed = true;
                reports[index] = format!("Worker failed: {error:#}");
                snapshot.stages[0].nodes[index].status = Status::Failed;
                snapshot.stages[0].nodes[index].detail = preview(&reports[index]);
            }
        }
        snapshot.stages[0].progress.as_mut().unwrap().settled += 1;
        snapshot.revision += 1;
        emit(event_tx, &snapshot).await;
    }

    snapshot.status = if interrupted {
        Status::Interrupted
    } else if failed {
        Status::Failed
    } else {
        Status::Completed
    };
    snapshot.stages[0].status = snapshot.status;
    snapshot.revision += 1;
    let report = task_data
        .iter()
        .enumerate()
        .map(|(index, (label, _))| format!("## {label}\n{}", reports[index]))
        .collect::<Vec<_>>()
        .join("\n\n");
    snapshot.delivery = preview(&report);
    emit(event_tx, &snapshot).await;
    Ok(report)
}

fn read_only_tools(parent: &PiAgent) -> crate::tools::ToolRegistry {
    let mut tools = crate::tools::ToolRegistry::new();
    // Preserve parent-owned wrappers and restrictions, including omitted tools.
    for name in ["read_file", "grep", "find"] {
        if let Some(tool) = parent.tools.get(name) {
            tools.register(tool);
        }
    }
    tools
}

async fn emit(tx: &mpsc::Sender<AgentEvent>, snapshot: &WorkflowSnapshot) {
    let _ = tx.send(AgentEvent::WorkflowUpdated(snapshot.clone())).await;
}

fn preview(value: &str) -> String {
    let mut chars = value.chars();
    let mut text: String = chars.by_ref().take(16_384).collect();
    if chars.next().is_some() {
        text.push_str("\n… [preview truncated]");
    }
    text
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}
