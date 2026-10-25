//! UI-facing snapshots emitted by a workflow executor, never inferred from tool names.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Waiting,
    Running,
    Completed,
    Failed,
    Interrupted,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Agent,
    Script,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub kind: NodeKind,
    /// Stable executor-assigned avatar identity; absent in legacy snapshots.
    #[serde(default)]
    pub avatar_index: Option<i32>,
    pub status: Status,
    #[serde(default)]
    pub detail: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stage {
    pub id: String,
    pub label: String,
    pub status: Status,
    pub nodes: Vec<Node>,
    /// Executor-observed executions, including failed settlements and repeated runs.
    /// Planned node rows alone cannot supply this counter.
    #[serde(default)]
    pub progress: Option<StageProgress>,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StageProgress {
    pub settled: u64,
    pub observed: u64,
}
impl Stage {
    pub fn visible_progress(&self) -> Option<StageProgress> {
        self.progress
            .filter(|p| p.observed > 0 && p.settled <= p.observed)
    }
}
/// Changes attributed by the executor to this workflow, not a workspace-wide Git scan.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    /// None denotes a binary or otherwise uncounted change.
    pub added: Option<u64>,
    pub removed: Option<u64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowSnapshot {
    pub id: String,
    pub revision: u64,
    pub parent_message_id: String,
    pub title: String,
    pub model: String,
    pub status: Status,
    pub stages: Vec<Stage>,
    #[serde(default)]
    pub delivery: String,
    #[serde(default)]
    pub changes: Vec<FileChange>,
}
impl WorkflowSnapshot {
    pub fn change_totals(&self) -> (u64, u64, usize) {
        self.changes.iter().fold(
            (0u64, 0u64, 0usize),
            |(added, removed, uncounted), file| match (file.added, file.removed) {
                (Some(a), Some(r)) => (
                    added.saturating_add(a),
                    removed.saturating_add(r),
                    uncounted,
                ),
                _ => (added, removed, uncounted + 1),
            },
        )
    }
    pub fn running_agents(&self) -> usize {
        self.stages
            .iter()
            .flat_map(|s| &s.nodes)
            .filter(|n| n.kind == NodeKind::Agent && n.status == Status::Running)
            .count()
    }
    pub fn interrupt(&mut self) {
        if self.status == Status::Running {
            self.status = Status::Interrupted;
        }
        for stage in &mut self.stages {
            if stage.status == Status::Running {
                stage.status = Status::Interrupted;
            }
            for node in &mut stage.nodes {
                if node.status == Status::Running {
                    node.status = Status::Interrupted;
                }
            }
        }
    }
}
