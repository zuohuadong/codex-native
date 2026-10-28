use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalState {
    pub objective: String,
    pub status: String,
    pub iteration: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileChangeItem {
    pub path: String,
    #[serde(default)]
    pub additions: Option<usize>,
    #[serde(default)]
    pub deletions: Option<usize>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnFileChanges {
    pub items: Vec<FileChangeItem>,
}

impl TurnFileChanges {
    pub fn files_count(&self) -> usize {
        self.items.len()
    }

    pub fn total_additions(&self) -> Option<usize> {
        self.items.iter().map(|item| item.additions).sum()
    }

    pub fn total_deletions(&self) -> Option<usize> {
        self.items.iter().map(|item| item.deletions).sum()
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TaskState {
    pub queue: VecDeque<String>,
    pub queue_paused: bool,
    pub error: Option<String>,
    pub tokens: usize,
    pub workflows: Vec<pi_agent_core::workflow::WorkflowSnapshot>,
    pub goal: Option<GoalState>,
    pub pending_recovery: Option<String>,
    pub file_changes: HashMap<String, TurnFileChanges>,
}
impl TaskState {
    pub fn update_workflow(&mut self, snapshot: pi_agent_core::workflow::WorkflowSnapshot) {
        if let Some(current) = self.workflows.iter_mut().find(|w| w.id == snapshot.id) {
            if current.parent_message_id == snapshot.parent_message_id
                && snapshot.revision > current.revision
            {
                *current = snapshot;
            }
        } else {
            self.workflows.push(snapshot);
        }
    }
    pub fn begin(&mut self) {
        self.error = None;
        if self.queue.is_empty() {
            self.queue_paused = false;
        }
    }
    pub fn enqueue(&mut self, prompt: String) {
        if !prompt.trim().is_empty() {
            self.queue.push_back(prompt);
        }
    }
    pub fn next_prompt(&mut self) -> Option<String> {
        if self.queue_paused || self.error.is_some() {
            None
        } else {
            self.queue.pop_front()
        }
    }
    pub fn interrupted(&mut self, reason: String) {
        self.error = Some(reason);
        self.queue_paused = true;
    }
    pub fn set_goal(&mut self, objective: String) {
        self.goal = Some(GoalState {
            objective,
            status: "running".into(),
            iteration: 1,
        });
    }
    pub fn resume_goal(&mut self) -> bool {
        if let Some(goal) = &mut self.goal {
            goal.status = "running".into();
            true
        } else {
            false
        }
    }
    pub fn clear_goal(&mut self) {
        self.goal = None;
    }
    pub fn get_file_changes(&self, message_id: &str) -> Option<&TurnFileChanges> {
        self.file_changes.get(message_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_new_run_clears_only_an_empty_queues_stale_pause() {
        let mut state = TaskState::default();
        state.interrupted("stopped".into());
        state.begin();
        state.enqueue("next".into());
        assert_eq!(state.next_prompt().as_deref(), Some("next"));
        state.enqueue("keep paused".into());
        state.interrupted("failed".into());
        state.begin();
        assert!(state.next_prompt().is_none());
    }
    #[test]
    fn failure_and_stop_pause_without_consuming_pending_work() {
        let mut state = TaskState::default();
        state.enqueue("first".into());
        state.enqueue("second".into());
        state.interrupted("stopped".into());
        assert!(state.next_prompt().is_none());
        state.error = None;
        state.queue_paused = false;
        assert_eq!(state.next_prompt().as_deref(), Some("first"));
        assert_eq!(state.next_prompt().as_deref(), Some("second"));
        assert!(state.next_prompt().is_none());
    }
    #[test]
    fn goal_state_round_trip_and_transitions() {
        let mut state = TaskState::default();
        assert!(state.goal.is_none());
        state.set_goal("Ship feature".into());
        assert_eq!(
            state.goal,
            Some(GoalState {
                objective: "Ship feature".into(),
                status: "running".into(),
                iteration: 1,
            })
        );
        state.goal.as_mut().unwrap().status = "paused".into();
        assert!(state.resume_goal());
        assert_eq!(state.goal.as_ref().unwrap().status, "running");
        state.clear_goal();
        assert!(state.goal.is_none());
        assert!(!state.resume_goal());
    }
    #[test]
    fn deserialization_defaults_new_fields_for_older_saved_states() {
        let json =
            r#"{"queue":["item"],"queue_paused":false,"error":null,"tokens":42,"workflows":[]}"#;
        let state: TaskState = serde_json::from_str(json).expect("deserialize legacy task state");
        assert_eq!(state.tokens, 42);
        assert!(state.goal.is_none());
        assert!(state.pending_recovery.is_none());
        assert!(state.file_changes.is_empty());
    }
    #[test]
    fn file_changes_metrics_and_recording() {
        let mut state = TaskState::default();
        let changes = TurnFileChanges {
            items: vec![
                FileChangeItem {
                    path: "crates/pi-desktop/src/main.rs".into(),
                    additions: Some(12),
                    deletions: Some(4),
                },
                FileChangeItem {
                    path: "Cargo.toml".into(),
                    additions: Some(2),
                    deletions: Some(0),
                },
            ],
        };
        assert_eq!(changes.files_count(), 2);
        assert_eq!(changes.total_additions(), Some(14));
        assert_eq!(changes.total_deletions(), Some(4));
        state.file_changes.insert("msg-1".into(), changes);
        assert!(state.get_file_changes("msg-1").is_some());
        assert_eq!(state.get_file_changes("msg-1").unwrap().files_count(), 2);
    }
}

#[cfg(test)]
mod workflow_tests {
    use super::*;
    use pi_agent_core::workflow::*;
    fn snapshot(revision: u64) -> WorkflowSnapshot {
        WorkflowSnapshot {
            id: "w1".into(),
            revision,
            parent_message_id: "user-1".into(),
            title: "workflow".into(),
            model: "test".into(),
            status: Status::Running,
            delivery: String::new(),
            changes: Vec::new(),
            stages: vec![Stage {
                progress: None,
                id: "s1".into(),
                label: "stage".into(),
                status: Status::Running,
                nodes: vec![
                    Node {
                        id: "agent-1".into(),
                        label: "worker".into(),
                        kind: NodeKind::Agent,
                        avatar_index: None,
                        status: Status::Running,
                        detail: String::new(),
                    },
                    Node {
                        id: "script-1".into(),
                        label: "script".into(),
                        kind: NodeKind::Script,
                        avatar_index: None,
                        status: Status::Running,
                        detail: String::new(),
                    },
                ],
            }],
        }
    }
    #[test]
    fn updates_are_monotonic_and_cannot_move_to_another_turn() {
        let mut state = TaskState::default();
        state.update_workflow(snapshot(2));
        state.update_workflow(snapshot(1));
        assert_eq!(state.workflows[0].revision, 2);
        let mut moved = snapshot(3);
        moved.parent_message_id = "other-turn".into();
        state.update_workflow(moved);
        assert_eq!(state.workflows[0].parent_message_id, "user-1");
        state.update_workflow(snapshot(4));
        assert_eq!(state.workflows.len(), 1);
        assert_eq!(state.workflows[0].revision, 4);
        let restored: TaskState =
            serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
        assert_eq!(restored.workflows[0].revision, 4);
    }
    #[test]
    fn interruption_preserves_completed_nodes_and_excludes_scripts_from_count() {
        let mut workflow = snapshot(1);
        assert_eq!(workflow.running_agents(), 1);
        workflow.stages[0].nodes[1].status = Status::Completed;
        workflow.interrupt();
        assert_eq!(workflow.running_agents(), 0);
        assert_eq!(workflow.stages[0].nodes[0].status, Status::Interrupted);
        assert_eq!(workflow.stages[0].nodes[1].status, Status::Completed);
    }
    #[test]
    fn old_saved_task_has_no_fabricated_workflow() {
        let state: TaskState = serde_json::from_str("{}").unwrap();
        assert!(state.workflows.is_empty());
    }

    #[test]
    fn change_totals_ignore_unscoped_binary_counts() {
        let mut workflow = snapshot(1);
        workflow.changes = vec![
            FileChange {
                path: "a.rs".into(),
                added: Some(9),
                removed: Some(8),
            },
            FileChange {
                path: "image.bin".into(),
                added: None,
                removed: None,
            },
        ];
        assert_eq!(workflow.change_totals(), (9, 8, 1));
    }

    #[test]
    fn stage_counts_preserve_observations_instead_of_counting_planned_rows() {
        let mut stage = snapshot(1).stages.remove(0);
        assert_eq!(stage.visible_progress(), None);
        // Repeated executions can outnumber the two planned rows.
        stage.progress = Some(StageProgress {
            settled: 7,
            observed: 9,
        });
        let restored: Stage =
            serde_json::from_str(&serde_json::to_string(&stage).unwrap()).unwrap();
        assert_eq!(restored.visible_progress(), stage.progress);
        for progress in [
            StageProgress {
                settled: 0,
                observed: 0,
            },
            StageProgress {
                settled: 3,
                observed: 2,
            },
        ] {
            stage.progress = Some(progress);
            assert_eq!(stage.visible_progress(), None);
        }
    }
}

#[cfg(test)]
mod workflow_visual_fixture_tests {
    use pi_agent_core::workflow::{Status, WorkflowSnapshot};

    #[test]
    fn reference_running_fixture_has_two_agents_and_recovers_truthfully() {
        let mut workflow: WorkflowSnapshot = serde_json::from_str(include_str!(
            "../../../scripts/visual_parity/fixtures/multi-agent-running.json"
        ))
        .unwrap();
        assert_eq!(workflow.running_agents(), 2);
        assert_eq!(workflow.status, Status::Running);
        assert_eq!(workflow.stages[0].nodes[2].status, Status::Completed);
        assert_eq!(workflow.stages[0].visible_progress().unwrap().settled, 1);
        assert_eq!(workflow.stages[0].visible_progress().unwrap().observed, 3);
        assert!(workflow.stages[1..]
            .iter()
            .all(|stage| stage.visible_progress().is_none()));
        workflow.interrupt();
        assert_eq!(workflow.status, Status::Interrupted);
        assert_eq!(workflow.running_agents(), 0);
        assert_eq!(workflow.stages[0].nodes[2].status, Status::Completed);
        assert_eq!(workflow.stages[1].status, Status::Waiting);
    }

    #[test]
    fn terminal_visual_fixtures_have_no_running_descendants() {
        for (json, expected) in [
            (
                include_str!("../../../scripts/visual_parity/fixtures/multi-agent-failed.json"),
                Status::Failed,
            ),
            (
                include_str!(
                    "../../../scripts/visual_parity/fixtures/multi-agent-interrupted.json"
                ),
                Status::Interrupted,
            ),
        ] {
            let workflow: WorkflowSnapshot = serde_json::from_str(json).unwrap();
            assert_eq!(workflow.status, expected);
            for stage in &workflow.stages {
                assert_ne!(stage.status, Status::Running);
                for node in &stage.nodes {
                    assert_ne!(node.status, Status::Running);
                    if stage.status == Status::Completed {
                        assert_eq!(node.status, Status::Completed);
                    }
                }
            }
            assert!(workflow.title.contains("离线视觉样例"));
        }
    }
}
