use crate::task_state::{FileChangeItem, TaskState, TurnFileChanges};
use pi_agent_core::{Message, Role};
use std::collections::{BTreeMap, HashMap};

pub(super) struct TurnSummary {
    pub turn_id: String,
    pub changes: TurnFileChanges,
}

pub(super) fn disclosure_key(session: &str, turn: &str) -> String {
    format!(
        "file-summary:{}:{session}{}:{turn}",
        session.len(),
        turn.len()
    )
}

/// Summaries belong to the user turn, after its final visible message. A tool
/// request is not a write receipt; successful local tools establish only paths,
/// since their results do not report a before/after diff.
pub(super) fn by_row(
    messages: &[&Message],
    state: Option<&TaskState>,
) -> HashMap<usize, TurnSummary> {
    let mut summaries = HashMap::new();
    let mut start = 0;
    while start < messages.len() {
        if messages[start].role != Role::User {
            start += 1;
            continue;
        }
        let end = messages[start + 1..]
            .iter()
            .position(|message| message.role == Role::User)
            .map_or(messages.len(), |offset| start + 1 + offset);
        let turn = &messages[start..end];
        let results: HashMap<_, _> = turn
            .iter()
            .enumerate()
            .filter_map(|(index, message)| {
                message
                    .tool_result
                    .as_ref()
                    .map(|result| (result.tool_call_id.as_str(), (index, result)))
            })
            .collect();
        let mut files = BTreeMap::new();
        for (index, message) in turn.iter().enumerate() {
            for call in message.tool_calls.iter().flatten() {
                if !matches!(call.name.as_str(), "write_file" | "edit_file") {
                    continue;
                }
                let Some((result_index, result)) = results.get(call.id.as_str()) else {
                    continue;
                };
                if *result_index <= index || result.is_error || result.name != call.name {
                    continue;
                }
                let Some(path) = call.arguments.get("path").and_then(|path| path.as_str()) else {
                    continue;
                };
                if !path.trim().is_empty() {
                    files
                        .entry(path.to_owned())
                        .or_insert_with(|| FileChangeItem {
                            path: path.to_owned(),
                            additions: None,
                            deletions: None,
                        });
                }
            }
        }
        // Enrich only receipt-confirmed paths with persisted observations.
        // Old snapshots cannot turn a failed request into a successful write.
        if let Some(state) = state {
            for message in turn {
                if let Some(changes) = state.get_file_changes(&message.id) {
                    for item in &changes.items {
                        if let Some(file) = files.get_mut(&item.path) {
                            *file = item.clone();
                        }
                    }
                }
            }
        }
        if !files.is_empty() {
            summaries.insert(
                end - 1,
                TurnSummary {
                    turn_id: messages[start].id.clone(),
                    changes: TurnFileChanges {
                        items: files.into_values().collect(),
                    },
                },
            );
        }
        start = end;
    }
    summaries
}

#[cfg(test)]
mod tests {
    use super::*;
    use pi_agent_core::ToolCall;
    use serde_json::json;

    fn call(id: &str, name: &str, path: &str) -> Message {
        let mut message = Message::assistant("");
        message.tool_calls = Some(vec![ToolCall {
            id: id.into(),
            name: name.into(),
            arguments: json!({
                "path": path,
                "content": "Not evidence of additions",
                "target_content": "old",
                "replacement_content": "new",
            }),
        }]);
        message
    }

    fn summarize(messages: &[Message]) -> HashMap<usize, TurnSummary> {
        by_row(&messages.iter().collect::<Vec<_>>(), None)
    }

    #[test]
    fn calls_without_successful_matching_results_have_no_summary() {
        for messages in [
            vec![Message::user("edit"), call("a", "edit_file", "a.rs")],
            vec![
                Message::user("edit"),
                call("a", "edit_file", "a.rs"),
                Message::tool_result("a", "edit_file", "failed", true),
            ],
            vec![
                Message::user("edit"),
                call("a", "edit_file", "a.rs"),
                Message::tool_result("other", "edit_file", "ok", false),
            ],
            vec![
                Message::user("edit"),
                call("a", "edit_file", "a.rs"),
                Message::tool_result("a", "read_file", "ok", false),
            ],
            vec![
                Message::user("edit"),
                Message::tool_result("a", "edit_file", "ok", false),
                call("a", "edit_file", "a.rs"),
            ],
            vec![
                Message::user("patch"),
                call("a", "apply_patch", "a.rs"),
                Message::tool_result("a", "apply_patch", "ok", false),
            ],
        ] {
            assert!(summarize(&messages).is_empty());
        }
    }

    #[test]
    fn successful_writes_are_deduplicated_at_the_turn_tail_without_guessed_counts() {
        let user = Message::user("update");
        let id = user.id.clone();
        let messages = vec![
            user,
            call("a", "write_file", "a.rs"),
            Message::tool_result("a", "write_file", "ok", false),
            call("b", "edit_file", "a.rs"),
            Message::tool_result("b", "edit_file", "ok", false),
            call("c", "edit_file", "b.rs"),
            Message::tool_result("c", "edit_file", "ok", false),
            Message::assistant("Done"),
        ];
        let summaries = summarize(&messages);
        assert_eq!(summaries.len(), 1);
        let summary = &summaries[&7];
        assert_eq!(summary.turn_id, id);
        assert_eq!(summary.changes.files_count(), 2);
        assert_eq!(summary.changes.items[0].path, "a.rs");
        assert_eq!(summary.changes.total_additions(), None);
        assert_eq!(summary.changes.total_deletions(), None);
    }

    #[test]
    fn results_cannot_confirm_calls_from_another_turn() {
        let messages = vec![
            Message::user("first"),
            call("a", "write_file", "a.rs"),
            Message::user("second"),
            Message::tool_result("a", "write_file", "ok", false),
        ];
        assert!(summarize(&messages).is_empty());
    }

    #[test]
    fn explicit_observations_override_unknown_counts_and_preserve_legacy_numbers() {
        let messages = vec![
            Message::user("update"),
            call("a", "edit_file", "a.rs"),
            Message::tool_result("a", "edit_file", "ok", false),
            Message::assistant("Done"),
        ];
        let mut state = TaskState::default();
        state.file_changes.insert(
            messages[0].id.clone(),
            serde_json::from_value(json!({
                "items": [{ "path": "a.rs", "additions": 2, "deletions": 1 }]
            }))
            .unwrap(),
        );
        let summaries = by_row(&messages.iter().collect::<Vec<_>>(), Some(&state));
        assert_eq!(summaries[&3].changes.total_additions(), Some(2));
        assert_eq!(summaries[&3].changes.total_deletions(), Some(1));
    }

    #[test]
    fn observations_cannot_create_a_summary_without_a_tool_receipt() {
        let messages = vec![
            Message::user("failed update"),
            call("a", "write_file", "unrelated.rs"),
            Message::tool_result("a", "write_file", "failed", true),
            Message::assistant("failed"),
        ];
        let mut state = TaskState::default();
        for message in &messages {
            state.file_changes.insert(
                message.id.clone(),
                serde_json::from_value(json!({
                    "items": [{ "path": "unrelated.rs", "additions": 9, "deletions": 8 }]
                }))
                .unwrap(),
            );
        }
        assert!(by_row(&messages.iter().collect::<Vec<_>>(), Some(&state)).is_empty());
    }

    #[test]
    fn disclosure_is_scoped_to_session_and_unambiguous_turn_identity() {
        assert_ne!(
            disclosure_key("chat-a", "same"),
            disclosure_key("chat-b", "same")
        );
        assert_ne!(disclosure_key("a:1", "b"), disclosure_key("a", "1:b"));
    }
}
