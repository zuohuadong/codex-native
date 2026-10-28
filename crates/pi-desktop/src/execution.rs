use super::*;
fn local_command(prompt: &str) -> Option<String> {
    let trimmed = prompt.trim();
    if let Some(command) = trimmed.strip_prefix('$') {
        return Some(command.trim().to_string());
    }
    if ["cargo ", "git ", "cat ", "ls "]
        .iter()
        .any(|prefix| trimmed.starts_with(prefix))
    {
        return Some(trimmed.to_string());
    }
    match trimmed.to_lowercase().as_str() {
        "test" | "测试" => Some("cargo test -p pi-agent-core".into()),
        "lint" | "clippy" => Some("cargo clippy -p pi-agent-core".into()),
        "build" => Some("cargo check -p pi-desktop".into()),
        _ => None,
    }
}

pub fn reconcile_interrupted_tools(messages: &mut Vec<Message>) {
    let completed: std::collections::HashSet<String> = messages
        .iter()
        .filter_map(|message| {
            message
                .tool_result
                .as_ref()
                .map(|result| result.tool_call_id.clone())
        })
        .collect();
    let mut reconciled = Vec::with_capacity(messages.len());
    for message in messages.drain(..) {
        let missing: Vec<_> = message
            .tool_calls
            .iter()
            .flatten()
            .filter(|call| !completed.contains(&call.id))
            .cloned()
            .collect();
        reconciled.push(message);
        for call in missing {
            reconciled.push(Message::tool_result(
                call.id,
                call.name,
                "Execution interrupted; outcome unknown. Check workspace before retrying.",
                true,
            ));
        }
    }
    *messages = reconciled;
}
impl PiDesktopApp {
    pub fn handle_generative_ui_action(
        &mut self,
        action: &crate::generative_ui::UiAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let prompt = format!("已执行操作【{}】(ID: {})", action.label, action.id);
        self.input_state.update(cx, |input, cx| {
            input.set_value(prompt, window, cx);
        });
        self.trigger_send(window, cx);
    }

    pub fn trigger_send(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let prompt = self.input_state.read(cx).value().to_string();
        if prompt.trim().is_empty() {
            return;
        }
        self.input_state
            .update(cx, |input, cx| input.clean(window, cx));
        self.drafts.remove(&self.session.id);

        if let Some(cmd) = crate::workbench::parse_slash_command(&prompt) {
            match cmd {
                crate::workbench::SlashCommand::Compact => {
                    self.session.messages.push(Message::user("/compact"));
                    self.session.messages.push(Message::assistant(
                        self.tr("会话历史已压缩。", "Conversation history compacted."),
                    ));
                    self.persist_active_session();
                    cx.notify();
                    return;
                }
                crate::workbench::SlashCommand::SendGoal { objective } => {
                    self.task_states
                        .entry(self.session.id.clone())
                        .or_default()
                        .set_goal(objective.clone());
                    self.submit_prompt(objective, cx);
                    return;
                }
                crate::workbench::SlashCommand::ResumeGoal => {
                    let objective = self.task_states.get_mut(&self.session.id).and_then(|s| {
                        if s.resume_goal() {
                            s.goal.as_ref().map(|g| g.objective.clone())
                        } else {
                            None
                        }
                    });
                    if let Some(obj) = objective {
                        self.submit_prompt(obj, cx);
                    } else {
                        self.run_error = Some(
                            self.tr(
                                "当前没有可恢复的目标，请先使用 /goal 设定目标。",
                                "No goal to resume. Set a goal first using /goal.",
                            )
                            .into(),
                        );
                        cx.notify();
                    }
                    return;
                }
                crate::workbench::SlashCommand::ClearGoal => {
                    self.task_states
                        .entry(self.session.id.clone())
                        .or_default()
                        .clear_goal();
                    self.persist_active_session();
                    cx.notify();
                    return;
                }
                crate::workbench::SlashCommand::EmptyGoal => {
                    self.run_error = Some(
                        self.tr(
                            "请输入目标内容，例如: /goal 实现用户登录",
                            "Please provide a goal objective, e.g.: /goal implement user login",
                        )
                        .into(),
                    );
                    cx.notify();
                    return;
                }
            }
        }

        if self.is_executing {
            self.task_states
                .entry(self.session.id.clone())
                .or_default()
                .enqueue(prompt);
            self.persist_active_session();
            cx.notify();
            return;
        }
        self.submit_prompt(prompt, cx);
    }
    pub fn submit_prompt(&mut self, prompt: String, cx: &mut Context<Self>) {
        if self.is_executing || prompt.trim().is_empty() {
            return;
        }
        self.run_error = None;
        self.run_stopped = false;
        self.task_states
            .entry(self.session.id.clone())
            .or_default()
            .begin();
        if let Some(goal) = self
            .task_states
            .get_mut(&self.session.id)
            .and_then(|s| s.goal.as_mut())
        {
            goal.status = "running".into();
        }
        self.run_generation += 1;
        if !self
            .session
            .messages
            .iter()
            .any(|m| m.role == pi_agent_core::Role::User)
        {
            self.session.title = prompt.chars().take(48).collect();
            if let Some(entry) = self
                .conversations
                .iter_mut()
                .find(|entry| entry.id == self.session.id)
            {
                entry.title = self.session.title.clone();
            }
        }
        self.session.messages.push(Message::user(prompt.trim()));
        self.session.updated_at = desktop_state::now() as i64 * 1000;
        self.is_executing = true;
        self.status_message = self.tr("正在执行", "Working").into();
        self.show_settings = false;
        self.selected_nav = "chat";
        self.persist_active_session();
        self.scroll_pinned = true;
        self.scroll.scroll_to_bottom();
        cx.notify();
        if let Some(command) = local_command(&prompt) {
            self.start_local_command(command, cx);
        } else if !self.api_key.trim().is_empty() {
            self.start_agent_turn(cx);
        } else {
            self.is_executing = false;
            self.run_error = Some(
                self.tr(
                    "请在设置中配置 API Key 后重试。",
                    "Configure an API key in Settings, then retry.",
                )
                .into(),
            );
            self.task_states
                .entry(self.session.id.clone())
                .or_default()
                .interrupted(self.run_error.clone().unwrap_or_default());
            self.persist_active_session();
            cx.notify();
        }
    }
    fn start_local_command(&mut self, command: String, cx: &mut Context<Self>) {
        let agent = self.agent.clone();
        let generation = self.run_generation;
        let id = desktop_state::new_id();
        let mut call = Message::assistant("");
        call.tool_calls = Some(vec![pi_agent_core::ToolCall {
            id: id.clone(),
            name: "bash".into(),
            arguments: serde_json::json!({"command":command}),
        }]);
        self.session.messages.push(call);
        let worker = self.runtime.spawn(async move {
            let start = std::time::Instant::now();
            let result = match agent.tools.get("bash") {
                Some(tool) => tool.execute(serde_json::json!({"command":command})).await,
                None => Err(anyhow::anyhow!("bash is unavailable")),
            };
            (result, start.elapsed())
        });
        self.running = Some(worker.abort_handle());
        cx.spawn(async move |this, cx| {
            let outcome = worker.await;
            let _ = this.update(cx, |this, cx| {
                if this.run_generation != generation {
                    return;
                }
                let (output, is_error) = match outcome {
                    Ok((Ok(output), elapsed)) => {
                        (format!("{output}\n\n{:.2}s", elapsed.as_secs_f32()), false)
                    }
                    Ok((Err(error), _)) => (error.to_string(), true),
                    Err(error) => (error.to_string(), true),
                };
                if is_error && !this.run_stopped {
                    this.run_error = Some(output.clone());
                }
                this.session.messages.push(Message::tool_result(
                    id.clone(),
                    "bash",
                    output,
                    is_error,
                ));
                if is_error {
                    this.expanded_tools.insert(id);
                }
                this.finish_run(cx);
            });
        })
        .detach();
    }
    fn start_agent_turn(&mut self, cx: &mut Context<Self>) {
        let generation = self.run_generation;
        let mut messages = self.session.messages.clone();
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        let model = ModelConfig {
            provider: "openai-compatible".into(),
            model: self.model_identifier.clone(),
            api_base: Some(self.api_base.clone()),
            api_key: Some(self.api_key.clone()),
            temperature: 0.2,
            max_tokens: Some(8192),
        };
        let agent = PiAgent::new(self.workspace_dir.clone(), model);
        self.workflow_controls = agent.workflow_controls.clone();
        let worker = self.runtime.spawn(async move {
            let outcome = agent.run_turn(&mut messages, tx).await;
            (outcome, messages)
        });
        self.running = Some(worker.abort_handle());
        cx.spawn(async move |this, cx| {
            while let Some(event) = rx.recv().await {
                if this
                    .update(cx, |this, cx| {
                        if this.run_generation == generation {
                            this.apply_agent_event(event, cx);
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
            let outcome = worker.await;
            let _ = this.update(cx, |this, cx| {
                if this.run_generation != generation {
                    return;
                }
                match outcome {
                    Ok((Ok(()), messages)) => {
                        this.session.messages = messages
                            .into_iter()
                            .filter(|m| m.role != pi_agent_core::Role::System)
                            .collect()
                    }
                    Ok((Err(error), _)) if !this.run_stopped => {
                        this.run_error = Some(error.to_string())
                    }
                    Err(error) if !this.run_stopped => this.run_error = Some(error.to_string()),
                    _ => {}
                }
                this.finish_run(cx);
            });
        })
        .detach();
    }
    fn apply_agent_event(&mut self, event: AgentEvent, cx: &mut Context<Self>) {
        match event {
            AgentEvent::WorkflowUpdated(snapshot) => {
                if self.session.messages.iter().any(|message| {
                    message.id == snapshot.parent_message_id
                        && message.role == pi_agent_core::Role::User
                }) {
                    self.task_states
                        .entry(self.session.id.clone())
                        .or_default()
                        .update_workflow(snapshot);
                }
            }
            AgentEvent::TextDelta(text) => {
                if self.session.messages.last().is_none_or(|m| {
                    m.role != pi_agent_core::Role::Assistant || m.tool_calls.is_some()
                }) {
                    self.session.messages.push(Message::assistant(""));
                }
                self.session
                    .messages
                    .last_mut()
                    .unwrap()
                    .content
                    .push_str(&text);
            }
            AgentEvent::Thought(text) => {
                if self.session.messages.last().is_none_or(|m| {
                    m.role != pi_agent_core::Role::Assistant || m.tool_calls.is_some()
                }) {
                    self.session.messages.push(Message::assistant(""));
                }
                self.session
                    .messages
                    .last_mut()
                    .unwrap()
                    .reasoning
                    .get_or_insert_default()
                    .push_str(&text);
            }
            AgentEvent::ToolCallStarted {
                id,
                name,
                arguments,
            } => {
                self.status_message = format!("{}: {name}", self.tr("运行工具", "Running tool"));
                let mut message = Message::assistant("");
                message.tool_calls = Some(vec![pi_agent_core::ToolCall {
                    id,
                    name,
                    arguments,
                }]);
                self.session.messages.push(message);
            }
            AgentEvent::ToolCallFinished {
                id,
                name,
                output,
                is_error,
            } => {
                if is_error {
                    self.expanded_tools.insert(id.clone());
                }
                self.session
                    .messages
                    .push(Message::tool_result(id, name, output, is_error));
            }
            AgentEvent::Completed { total_tokens } => {
                if let Some(tokens) = total_tokens {
                    self.tokens_used = tokens as usize;
                }
            }
            AgentEvent::Error(error) if !self.run_stopped => self.run_error = Some(error),
            AgentEvent::Error(_) => {}
        }
        if self.scroll_pinned {
            self.scroll.scroll_to_bottom();
        }
        self.schedule_save(cx);
        cx.notify();
    }
    fn finish_run(&mut self, cx: &mut Context<Self>) {
        self.running = None;
        if let Some(state) = self.task_states.get_mut(&self.session.id) {
            for workflow in &mut state.workflows {
                workflow.interrupt();
            }
        }
        reconcile_interrupted_tools(&mut self.session.messages);
        self.is_executing = false;
        self.status_message = self.tr("已完成", "Completed").into();
        if self.run_stopped {
            self.status_message = self.tr("已停止", "Stopped").into();
        } else if let Some(error) = &self.run_error {
            self.status_message = self.tr("执行失败", "Failed").into();
            self.task_states
                .entry(self.session.id.clone())
                .or_default()
                .interrupted(error.clone());
        } else {
            self.task_states
                .entry(self.session.id.clone())
                .or_default()
                .error = None;
        }
        if let Some(state) = self.task_states.get_mut(&self.session.id) {
            if let Some(goal) = &mut state.goal {
                if self.run_stopped {
                    goal.status = "paused".into();
                } else if self.run_error.is_some() {
                    goal.status = "failed".into();
                } else {
                    goal.iteration += 1;
                    goal.status = "completed".into();
                }
            }
        }
        self.capture_task_state();
        self.persist_active_session();
        if self.run_error.is_none() {
            if let Some(prompt) = self
                .task_states
                .entry(self.session.id.clone())
                .or_default()
                .next_prompt()
            {
                self.submit_prompt(prompt, cx);
            }
        }
        if self.review_visible {
            self.refresh_review(cx);
        }
        cx.notify();
    }
    pub fn stop_run(&mut self, cx: &mut Context<Self>) {
        if let Some(task) = self.running.take() {
            task.abort();
        }
        self.status_message = self.tr("正在停止", "Stopping").into();
        self.run_stopped = true;
        // Keep the run busy until the join callback observes cancellation and cleanup.
        self.run_error = Some(
            self.tr(
                "任务已停止；请检查已执行的工具结果。",
                "Stopped. Review any tool actions already performed.",
            )
            .into(),
        );
        self.task_states
            .entry(self.session.id.clone())
            .or_default()
            .interrupted(self.run_error.clone().unwrap_or_default());
        self.persist_active_session();
        cx.notify();
    }

    pub fn stop_workflow(&mut self, workflow_id: String, cx: &mut Context<Self>) {
        if self.workflow_controls.stop(&workflow_id) {
            // The card owns its pending-stop label. The parent turn continues.
            cx.notify();
        }
    }
    pub fn retry_run(&mut self, cx: &mut Context<Self>) {
        if self.is_executing {
            return;
        }
        if let Some(last) = self
            .session
            .messages
            .iter()
            .rposition(|m| m.role == pi_agent_core::Role::User)
        {
            let prompt = self.session.messages[last].content.clone();
            self.submit_prompt(prompt, cx);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::{local_command, mask_api_key, Language};

    #[test]
    fn local_command_dispatch_preserves_existing_routes() {
        assert_eq!(local_command("$ git status").as_deref(), Some("git status"));
        assert_eq!(local_command("cargo test").as_deref(), Some("cargo test"));
        assert_eq!(
            local_command("测试").as_deref(),
            Some("cargo test -p pi-agent-core")
        );
        assert_eq!(
            local_command("lint").as_deref(),
            Some("cargo clippy -p pi-agent-core")
        );
        assert_eq!(
            local_command("build").as_deref(),
            Some("cargo check -p pi-desktop")
        );
        assert_eq!(local_command("hello"), None);
    }

    #[test]
    fn interrupted_tools_are_reconciled_without_losing_completed_results() {
        use super::*;
        let mut assistant = Message::assistant("partial");
        assistant.tool_calls = Some(vec![
            pi_agent_core::ToolCall {
                id: "done".into(),
                name: "bash".into(),
                arguments: serde_json::json!({}),
            },
            pi_agent_core::ToolCall {
                id: "pending".into(),
                name: "bash".into(),
                arguments: serde_json::json!({}),
            },
        ]);
        let mut messages = vec![
            assistant,
            Message::tool_result("done", "bash", "actual output", false),
        ];
        reconcile_interrupted_tools(&mut messages);
        assert_eq!(messages.len(), 3);
        assert!(messages[1].tool_result.as_ref().unwrap().is_error);
        assert_eq!(messages[2].content, "actual output");
        reconcile_interrupted_tools(&mut messages);
        assert_eq!(messages.len(), 3);
    }

    #[test]
    fn masks_long_short_and_unicode_credentials() {
        for key in [
            "dummy-test-credential",
            "短密钥",
            "12345678",
            "测试用的长字符串凭证",
        ] {
            let masked = mask_api_key(key, Language::English);
            assert!(!masked.contains(key));
            assert!(masked.contains("••••"));
        }
        assert_eq!(mask_api_key("", Language::English), "Not configured");
    }
}
