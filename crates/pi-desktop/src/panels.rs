use super::*;
use crate::workbench::{action, heading, icon_button};
use gpui_kit::assets::IconName as Glyph;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::prelude::FluentBuilder;

pub(super) const REVIEW_SIDE_PANE_WIDTH: Pixels = px(380.);
pub(super) const REVIEW_SINGLE_COLUMN_BREAKPOINT: Pixels = px(1180.);
const COMMAND_DRAWER_MAX_HEIGHT: Pixels = px(210.);
const COMMAND_DRAWER_VIEWPORT_RATIO: f32 = 0.3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReviewPanelState {
    Loading,
    Error,
    Empty,
    Ready,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CommandDrawerState {
    Empty,
    Running,
    Ready,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum QueuePanelStatus {
    Paused,
    Active(usize),
}

pub(super) fn queue_panel_status(paused: bool, count: usize) -> QueuePanelStatus {
    if paused {
        QueuePanelStatus::Paused
    } else {
        QueuePanelStatus::Active(count)
    }
}

/// Below this width Review replaces the conversation instead of sharing the row.
pub(super) fn review_uses_single_column(width: Pixels) -> bool {
    width < REVIEW_SINGLE_COLUMN_BREAKPOINT
}

/// Keep the command runner as a shallow bottom drawer, never a full terminal.
pub(super) fn command_drawer_height(viewport_height: Pixels) -> Pixels {
    (viewport_height * COMMAND_DRAWER_VIEWPORT_RATIO).min(COMMAND_DRAWER_MAX_HEIGHT)
}

pub(super) fn review_panel_state(
    loading: bool,
    has_error: bool,
    visible_file_count: usize,
) -> ReviewPanelState {
    if loading {
        ReviewPanelState::Loading
    } else if has_error {
        ReviewPanelState::Error
    } else if visible_file_count == 0 {
        ReviewPanelState::Empty
    } else {
        ReviewPanelState::Ready
    }
}

pub(super) fn command_drawer_state(has_output: bool, running: bool) -> CommandDrawerState {
    if has_output {
        CommandDrawerState::Ready
    } else if running {
        CommandDrawerState::Running
    } else {
        CommandDrawerState::Empty
    }
}

impl PiDesktopApp {
    pub fn capture_task_state(&mut self) {
        let state = self.task_states.entry(self.session.id.clone()).or_default();
        state.error = self.run_error.clone();
        state.tokens = self.tokens_used;
    }

    pub fn restore_task_state(&mut self) {
        let state = self
            .task_states
            .get(&self.session.id)
            .cloned()
            .unwrap_or_default();
        self.run_error = state.error;
        self.tokens_used = state.tokens;
    }

    pub fn run_terminal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let command = self.terminal_input.read(cx).value().trim().to_string();
        if command.is_empty() || self.is_executing {
            return;
        }
        self.terminal_input
            .update(cx, |input, cx| input.clean(window, cx));
        self.submit_prompt(format!("$ {command}"), cx);
        self.terminal_visible = true;
    }

    pub fn toggle_review(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.workflow_pane.is_some() {
            self.close_workflow_pane(window, cx);
        }
        self.review_visible = !self.review_visible;
        // The old side-pane control may unmount in the single-column layout.
        self.root_focus.focus(window, cx);
        if self.review_visible {
            self.refresh_review(cx);
        }
        cx.notify();
    }

    pub fn open_review_file(&mut self, file: String, window: &mut Window, cx: &mut Context<Self>) {
        if self.workflow_pane.is_some() {
            self.close_workflow_pane(window, cx);
        }
        self.review_visible = true;
        self.root_focus.focus(window, cx);
        let Some(relative) = review::workspace_relative_path(&self.workspace_dir, &file) else {
            self.review.generation += 1;
            self.review.loading = false;
            self.review.selected = None;
            self.review.diff.clear();
            self.review.error = Some(
                self.tr(
                    "文件不在当前工作区内",
                    "File is outside the current workspace",
                )
                .into(),
            );
            cx.notify();
            return;
        };
        self.review_search
            .update(cx, |input, cx| input.clean(window, cx));
        self.review.selected = Some(relative);
        self.refresh_review(cx);
        cx.notify();
    }

    pub fn toggle_terminal(&mut self, cx: &mut Context<Self>) {
        self.terminal_visible = !self.terminal_visible;
        cx.notify();
    }

    pub fn refresh_review(&mut self, cx: &mut Context<Self>) {
        self.review.loading = true;
        self.review.error = None;
        self.review.generation += 1;
        let generation = self.review.generation;
        let workspace = self.workspace_dir.clone();
        let mode = self.review.mode;
        let selected = self.review.selected.clone();
        let worker = self
            .runtime
            .spawn(async move { review::snapshot(&workspace, mode, selected).await });
        cx.spawn(async move |this, cx| {
            let result = worker.await;
            let _ = this.update(cx, |this, cx| {
                if this.review.generation != generation {
                    return;
                }
                this.review.loading = false;
                match result {
                    Ok(Ok(snapshot)) => {
                        this.review.files = snapshot.files;
                        this.review.selected = snapshot.selected;
                        this.review.diff = snapshot.diff;
                    }
                    Ok(Err(error)) => this.review.error = Some(error.to_string()),
                    Err(error) => this.review.error = Some(error.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    pub fn resume_queue(&mut self, cx: &mut Context<Self>) {
        if self.is_executing {
            return;
        }
        let state = self.task_states.entry(self.session.id.clone()).or_default();
        state.queue_paused = false;
        state.error = None;
        self.run_error = None;
        if let Some(prompt) = state.next_prompt() {
            self.submit_prompt(prompt, cx);
        }
        self.persist_active_session();
        cx.notify();
    }

    pub fn review_view(&self, colors: &ThemeColors, cx: &mut Context<Self>) -> AnyElement {
        let query = self.review_search.read(cx).value().to_lowercase();
        let files: Vec<String> = self
            .review
            .files
            .iter()
            .filter(|path| path.to_lowercase().contains(&query))
            .cloned()
            .collect();
        let state = review_panel_state(
            self.review.loading,
            self.review.error.is_some(),
            files.len(),
        );
        let header = div()
            .h_flex()
            .items_center()
            .h(px(48.))
            .px_3()
            .border_b_1()
            .border_color(rgba(colors.card_border))
            .gap_2()
            .child(
                div()
                    .text_size(self.ui_text(14.))
                    .font_semibold()
                    .text_color(rgb(colors.text_primary))
                    .child(self.tr("审查变更", "Review changes")),
            )
            .child(div().flex_1())
            .child(
                icon_button(
                    "refresh-review",
                    self.tr("刷新", "Refresh"),
                    Glyph::RefreshCw,
                )
                .disabled(self.review.loading)
                .on_click(cx.listener(|this, _, _, cx| this.refresh_review(cx))),
            )
            .child(
                icon_button("close-review", self.tr("关闭", "Close"), Glyph::X)
                    .on_click(cx.listener(|this, _, window, cx| this.toggle_review(window, cx))),
            );
        let mut content = div()
            .v_flex()
            .flex_1()
            .min_h_0()
            .h_full()
            .min_w_0()
            .p_3()
            .gap_3()
            .child(
                div().h_flex().w_full().flex_wrap().gap_1().children(
                    [
                        (review::ReviewMode::Unstaged, self.tr("未暂存", "Unstaged")),
                        (review::ReviewMode::Staged, self.tr("已暂存", "Staged")),
                        (
                            review::ReviewMode::Untracked,
                            self.tr("未跟踪", "Untracked"),
                        ),
                    ]
                    .into_iter()
                    .enumerate()
                    .map(|(index, (mode, label))| {
                        action(format!("review-mode-{index}"), label, Glyph::File)
                            .selected(self.review.mode == mode)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.review.mode = mode;
                                this.review.selected = None;
                                this.refresh_review(cx);
                            }))
                    }),
                ),
            )
            .child(Input::new(&self.review_search).h(px(32.)));
        match state {
            ReviewPanelState::Loading => {
                content = content.child(self.review_status_surface(
                    colors,
                    self.tr("正在读取变更…", "Loading changes…"),
                    false,
                ));
            }
            ReviewPanelState::Error => {
                content = content.child(self.review_status_surface(
                    colors,
                    self.review.error.clone().unwrap_or_else(|| {
                        self.tr("无法读取变更", "Could not read changes")
                            .to_string()
                    }),
                    true,
                ));
            }
            ReviewPanelState::Empty => {
                let message = if self.review.files.is_empty() {
                    self.tr("此分类没有变更", "No changes in this category")
                } else {
                    self.tr("没有匹配的文件", "No matching files")
                };
                content = content.child(self.review_status_surface(colors, message, false));
            }
            ReviewPanelState::Ready => {
                let files_scroll = self.review_files_scroll.clone();
                content = content
                    .child(
                        div()
                            .v_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .text_color(rgba(colors.foreground_subtle))
                                    .child(format!(
                                        "{} · {}",
                                        self.tr("文件", "Files"),
                                        files.len()
                                    )),
                            )
                            .child(
                                div()
                                    .id("review-files")
                                    .v_flex()
                                    .gap(px(2.))
                                    .max_h(px(150.))
                                    .overflow_y_scroll()
                                    .track_scroll(&self.review_files_scroll)
                                    .on_scroll_wheel(
                                        move |event: &gpui::ScrollWheelEvent, window, cx| {
                                            let delta_y =
                                                event.delta.pixel_delta(window.line_height()).y
                                                    / px(1.);
                                            let offset = -files_scroll.offset().y / px(1.);
                                            let maximum = files_scroll.max_offset().y / px(1.);
                                            if crate::transcript_scroll::nested_wheel_captures(
                                                delta_y, offset, maximum,
                                            ) {
                                                cx.stop_propagation();
                                            }
                                        },
                                    )
                                    .children(files.into_iter().map(|path| {
                                        let action_path = path.clone();
                                        action(format!("review-file-{path}"), path, Glyph::File)
                                            .w_full()
                                            .selected(
                                                self.review.selected.as_ref() == Some(&action_path),
                                            )
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.review.selected = Some(action_path.clone());
                                                this.refresh_review(cx);
                                            }))
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .v_flex()
                            .flex_1()
                            .min_h_0()
                            .rounded(px(8.))
                            .border_1()
                            .border_color(rgba(colors.card_border))
                            .overflow_hidden()
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .px_3()
                                    .py_2()
                                    .border_b_1()
                                    .border_color(rgba(colors.card_border))
                                    .bg(rgba(colors.card_user_bg))
                                    .font_family(cx.theme().mono_font_family.clone())
                                    .text_size(px(12.))
                                    .text_color(rgba(colors.foreground_subtle))
                                    .truncate()
                                    .child(
                                        self.review
                                            .selected
                                            .clone()
                                            .unwrap_or_else(|| self.tr("差异", "Diff").to_string()),
                                    ),
                            )
                            .child(if self.review.selected.is_some() {
                                self.review_diff_body(colors, cx)
                            } else {
                                self.review_status_surface(
                                    colors,
                                    self.tr(
                                        "所选文件不在此分类中",
                                        "Selected file is not in this category",
                                    ),
                                    false,
                                )
                                .into_any_element()
                            }),
                    );
            }
        }
        div()
            .v_flex()
            .flex_1()
            .min_h_0()
            .h_full()
            .min_w_0()
            .bg(rgb(colors.root_bg))
            .child(header)
            .child(content)
            .into_any_element()
    }

    fn review_status_surface(
        &self,
        colors: &ThemeColors,
        message: impl Into<SharedString>,
        error: bool,
    ) -> Div {
        div()
            .rounded(px(8.))
            .border_1()
            .border_color(rgba(if error {
                colors.destructive_border
            } else {
                colors.card_border
            }))
            .bg(if error {
                rgba(colors.destructive_surface)
            } else {
                rgb(colors.card_bg)
            })
            .p_3()
            .text_size(px(13.))
            .text_color(if error {
                rgb(colors.workflow_error)
            } else {
                rgba(colors.foreground_subtle)
            })
            .child(message.into())
    }

    fn review_diff_body(&self, colors: &ThemeColors, cx: &Context<Self>) -> AnyElement {
        let diff_scroll = self.review_diff_scroll.clone();
        div()
            .id("review-diff")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&self.review_diff_scroll)
            .on_scroll_wheel(move |event: &gpui::ScrollWheelEvent, window, cx| {
                let delta_y = event.delta.pixel_delta(window.line_height()).y / px(1.);
                let offset = -diff_scroll.offset().y / px(1.);
                let maximum = diff_scroll.max_offset().y / px(1.);
                if crate::transcript_scroll::nested_wheel_captures(delta_y, offset, maximum) {
                    cx.stop_propagation();
                }
            })
            .p_3()
            .font_family(cx.theme().mono_font_family.clone())
            .text_size(self.ui_text(14.))
            .line_height(px(20.))
            .text_color(rgb(colors.text_primary))
            .child(gpui_kit::base::SelectableText::new(
                "review-text",
                self.review.diff.clone(),
            ))
            .into_any_element()
    }

    pub fn terminal_drawer(
        &self,
        colors: &ThemeColors,
        height: Pixels,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let outputs: Vec<&pi_agent_core::ToolResult> = self
            .session
            .messages
            .iter()
            .filter_map(|message| message.tool_result.as_ref())
            .filter(|tool| tool.name == "bash")
            .collect();
        let state = command_drawer_state(!outputs.is_empty(), self.is_executing);
        let close = icon_button("terminal-close", self.tr("关闭", "Close"), Glyph::X)
            .on_click(cx.listener(|this, _, _, cx| this.toggle_terminal(cx)));
        let body = match state {
            CommandDrawerState::Empty | CommandDrawerState::Running if outputs.is_empty() => self
                .terminal_status_surface(
                    colors,
                    if state == CommandDrawerState::Running {
                        self.tr("正在运行命令…", "Running command…")
                    } else {
                        self.tr(
                            "尚无命令输出，请在下方运行命令。",
                            "No command output yet. Run a command below.",
                        )
                    },
                ),
            _ => {
                let output_scroll = self.terminal_output_scroll.clone();
                div()
                    .id("terminal-output")
                    .v_flex()
                    .flex_1()
                    .min_h_0()
                    .gap_2()
                    .overflow_y_scroll()
                    .track_scroll(&self.terminal_output_scroll)
                    .on_scroll_wheel(move |event: &gpui::ScrollWheelEvent, window, cx| {
                        let delta_y = event.delta.pixel_delta(window.line_height()).y / px(1.);
                        let offset = -output_scroll.offset().y / px(1.);
                        let maximum = output_scroll.max_offset().y / px(1.);
                        if crate::transcript_scroll::nested_wheel_captures(delta_y, offset, maximum)
                        {
                            cx.stop_propagation();
                        }
                    })
                    .children(outputs.into_iter().map(|tool| {
                        let border = if tool.is_error {
                            colors.destructive_border
                        } else {
                            colors.settings_border
                        };
                        let text_color = if tool.is_error {
                            rgb(colors.workflow_error)
                        } else {
                            rgba(colors.foreground_subtle)
                        };
                        let background = if tool.is_error {
                            rgba(colors.destructive_surface)
                        } else {
                            rgb(colors.root_bg)
                        };
                        div()
                            .flex_shrink_0()
                            .rounded(px(8.))
                            .border_1()
                            .border_color(rgba(border))
                            .bg(background)
                            .p_2()
                            .child(
                                div()
                                    .font_family(cx.theme().mono_font_family.clone())
                                    .text_size(px(14.))
                                    .line_height(px(20.))
                                    .text_color(text_color)
                                    .child(gpui_kit::base::SelectableText::new(
                                        format!("terminal-{}", tool.tool_call_id),
                                        tool.output.clone(),
                                    )),
                            )
                    }))
                    .into_any_element()
            }
        };
        div()
            .w_full()
            .h(height)
            .flex_shrink_0()
            .overflow_hidden()
            .p_3()
            .border_t_1()
            .border_color(rgba(colors.card_border))
            .bg(rgb(colors.card_bg))
            .v_flex()
            .gap_2()
            .child(
                div()
                    .h_flex()
                    .items_center()
                    .gap_2()
                    .child(heading(self.tr("命令输出", "Command output")))
                    .child(div().flex_1())
                    .child(close),
            )
            .child(body)
            .child(
                div()
                    .h_flex()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(Input::new(&self.terminal_input)),
                    )
                    .child(
                        action("terminal-run", self.tr("运行", "Run"), Glyph::Play)
                            .disabled(self.is_executing)
                            .on_click(
                                cx.listener(|this, _, window, cx| this.run_terminal(window, cx)),
                            ),
                    ),
            )
            .into_any_element()
    }

    fn terminal_status_surface(
        &self,
        colors: &ThemeColors,
        message: impl Into<SharedString>,
    ) -> AnyElement {
        div()
            .flex_1()
            .min_h_0()
            .rounded(px(8.))
            .border_1()
            .border_color(rgba(colors.card_border))
            .bg(rgb(colors.root_bg))
            .p_3()
            .text_size(px(13.))
            .text_color(rgba(colors.foreground_subtle))
            .child(message.into())
            .into_any_element()
    }

    pub fn queue_view(
        &self,
        content_width: Pixels,
        colors: &ThemeColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let state = self
            .task_states
            .get(&self.session.id)
            .cloned()
            .unwrap_or_default();
        if state.queue.is_empty() {
            return div().into_any_element();
        }
        let queue_len = state.queue.len();
        let paused = state.queue_paused;
        let empty = !self
            .session
            .messages
            .iter()
            .any(|m| m.role != pi_agent_core::Role::System);
        let target_width = if empty {
            content_width.min(px(672.))
        } else {
            crate::workbench::conversation_composer_width(content_width)
        };
        div()
            .id("pending-queue")
            .w_full()
            .px_4()
            .pb_2()
            .h_flex()
            .justify_center()
            .child(
                div()
                    .w_full()
                    .max_w(target_width)
                    .v_flex()
                    .gap_2()
                    .p_3()
                    .rounded(px(12.))
                    .border_1()
                    .border_color(rgba(colors.settings_border))
                    .bg(rgb(colors.dock_bg))
                    .shadow_xs()
                    .child(
                        div()
                            .h_flex()
                            .items_center()
                            .justify_between()
                            .gap_2()
                            .child(
                                div()
                                    .h_flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        Icon::new(if paused {
                                            Glyph::Pause
                                        } else {
                                            Glyph::ListOrdered
                                        })
                                        .size(px(14.))
                                        .text_color(rgba(colors.foreground_subtle)),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(12.))
                                            .font_medium()
                                            .text_color(rgba(colors.foreground_subtle))
                                            .child(match queue_panel_status(paused, queue_len) {
                                                QueuePanelStatus::Paused => self
                                                    .tr("队列已暂停", "Queue paused")
                                                    .to_string(),
                                                QueuePanelStatus::Active(count) => {
                                                    format!(
                                                        "{} · {}",
                                                        self.tr("待发送队列", "Queued prompts"),
                                                        count
                                                    )
                                                }
                                            }),
                                    ),
                            )
                            .when(paused, |header| {
                                header.child(
                                    action("queue-resume", self.tr("继续", "Resume"), Glyph::Play)
                                        .disabled(self.is_executing)
                                        .on_click(
                                            cx.listener(|this, _, _, cx| this.resume_queue(cx)),
                                        ),
                                )
                            }),
                    )
                    .child(
                        div()
                            .id("pending-queue-items")
                            .v_flex()
                            .gap_1()
                            .max_h(px(140.))
                            .overflow_y_scroll()
                            .children(state.queue.into_iter().enumerate().map(
                                |(index, prompt)| {
                                    div()
                                        .h_flex()
                                        .items_center()
                                        .gap_2()
                                        .px_3()
                                        .py_2()
                                        .rounded(px(8.))
                                        .border_1()
                                        .border_color(rgba(colors.card_border))
                                        .bg(rgba(colors.card_user_bg))
                                        .child(
                                            div()
                                                .flex_1()
                                                .min_w_0()
                                                .truncate()
                                                .text_size(px(14.))
                                                .child(prompt),
                                        )
                                        .child(
                                            icon_button(
                                                format!("queue-edit-{index}"),
                                                self.tr("编辑", "Edit"),
                                                Glyph::SquarePen,
                                            )
                                            .on_click(
                                                cx.listener(move |this, _, window, cx| {
                                                    if let Some(text) = this
                                                        .task_states
                                                        .entry(this.session.id.clone())
                                                        .or_default()
                                                        .queue
                                                        .remove(index)
                                                    {
                                                        this.input_state.update(cx, |input, cx| {
                                                            input.set_value(
                                                                text.clone(),
                                                                window,
                                                                cx,
                                                            );
                                                        });
                                                        this.drafts
                                                            .insert(this.session.id.clone(), text);
                                                        this.persist_active_session();
                                                        this.focus_composer(window, cx);
                                                        cx.notify();
                                                    }
                                                }),
                                            ),
                                        )
                                        .child(
                                            icon_button(
                                                format!("queue-remove-{index}"),
                                                self.tr("移除", "Remove"),
                                                Glyph::X,
                                            )
                                            .on_click(
                                                cx.listener(move |this, _, _, cx| {
                                                    this.task_states
                                                        .entry(this.session.id.clone())
                                                        .or_default()
                                                        .queue
                                                        .remove(index);
                                                    this.persist_active_session();
                                                    cx.notify();
                                                }),
                                            ),
                                        )
                                },
                            )),
                    ),
            )
            .into_any_element()
    }

    pub fn artifact_pane_view(&self, colors: &ThemeColors, cx: &mut Context<Self>) -> AnyElement {
        let Some(pane) = &self.artifact_pane else {
            return div().into_any_element();
        };

        let title = pane.title.clone();
        let kind = pane.kind.clone();
        let content = pane.content.clone();
        let view_mode = self.artifact_view_mode;
        let is_web = matches!(
            kind.to_lowercase().as_str(),
            "html" | "html5" | "canvas" | "svg" | "web"
        );

        let header = div()
            .h_flex()
            .items_center()
            .justify_between()
            .h(px(48.))
            .px_3()
            .border_b_1()
            .border_color(rgba(colors.card_border))
            .bg(rgb(colors.root_bg))
            .child(
                div()
                    .h_flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .px(px(6.))
                            .py(px(1.))
                            .rounded(px(4.))
                            .bg(rgba(colors.hover))
                            .text_size(self.ui_text(10.))
                            .font_semibold()
                            .text_color(rgb(colors.link))
                            .child(kind.to_uppercase()),
                    )
                    .child(
                        div()
                            .text_size(self.ui_text(13.))
                            .font_semibold()
                            .text_color(rgb(colors.text_primary))
                            .child(title.clone()),
                    ),
            )
            .child(
                div()
                    .h_flex()
                    .items_center()
                    .gap_1()
                    .child(
                        Button::new("artifact-toggle-mode")
                            .h(px(26.))
                            .rounded(px(6.))
                            .px_2()
                            .ghost()
                            .child(
                                div()
                                    .text_size(self.ui_text(11.))
                                    .child(if view_mode == ArtifactViewMode::Preview {
                                        self.tr("源码", "Source")
                                    } else {
                                        self.tr("预览", "Preview")
                                    }),
                            )
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.artifact_view_mode = match this.artifact_view_mode {
                                    ArtifactViewMode::Preview => ArtifactViewMode::Code,
                                    ArtifactViewMode::Code => ArtifactViewMode::Preview,
                                };
                                cx.notify();
                            })),
                    )
                    .when(is_web, |actions| {
                        let content_c = content.clone();
                        let title_c = title.clone();
                        actions.child(
                            Button::new("artifact-open-web-win")
                                .h(px(26.))
                                .rounded(px(6.))
                                .px_2()
                                .ghost()
                                .child(
                                    div()
                                        .text_size(self.ui_text(11.))
                                        .child(self.tr("Web视窗", "Web Window")),
                                )
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    let temp_dir = std::env::temp_dir();
                                    let file_path = temp_dir.join("volt_artifact_web.html");
                                    if std::fs::write(&file_path, &content_c).is_ok() {
                                        #[cfg(target_os = "macos")]
                                        let _ = std::process::Command::new("open").arg(&file_path).spawn();
                                        #[cfg(target_os = "windows")]
                                        let _ = std::process::Command::new("cmd").args(["/c", "start", file_path.to_str().unwrap()]).spawn();
                                        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
                                        let _ = std::process::Command::new("xdg-open").arg(&file_path).spawn();
                                        this.status_message = format!("已在独立 Web 视窗中打开【{}】", title_c);
                                        cx.notify();
                                    }
                                })),
                        )
                    })
                    .child(
                        Button::new("artifact-close-btn")
                            .h(px(26.))
                            .rounded(px(6.))
                            .px_2()
                            .ghost()
                            .child(
                                div()
                                    .text_size(self.ui_text(12.))
                                    .child("✕"),
                            )
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.artifact_visible = false;
                                cx.notify();
                            })),
                    ),
            );

        let body = match view_mode {
            ArtifactViewMode::Preview => {
                let content_c = content.clone();
                let title_c = title.clone();
                div()
                    .flex_1()
                    .p_3()
                    .v_flex()
                    .gap_3()
                    .child(
                        div()
                            .w_full()
                            .p_3()
                            .rounded(px(10.))
                            .bg(rgb(colors.card_bg))
                            .border_1()
                            .border_color(rgba(colors.card_border))
                            .v_flex()
                            .gap_2()
                            .child(
                                div()
                                    .h_flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .text_size(self.ui_text(13.))
                                            .font_semibold()
                                            .text_color(rgb(colors.text_primary))
                                            .child(self.tr("Web / HTML5 沙箱", "Web / HTML5 Sandbox")),
                                    )
                                    .child(
                                        div()
                                            .px(px(6.))
                                            .py(px(2.))
                                            .rounded(px(4.))
                                            .bg(rgba(colors.accent_green & 0x00ffffff | 0x20000000))
                                            .text_size(self.ui_text(11.))
                                            .text_color(rgb(colors.accent_green))
                                            .child(self.tr("沙箱就绪", "Ready")),
                                    ),
                            )
                            .child(
                                div()
                                    .text_size(self.ui_text(12.))
                                    .text_color(rgba(colors.foreground_subtle))
                                    .child(format!(
                                        "{}，大小 {} 字节。点击下方按钮即可在独立 GPU 硬件加速 Web 视窗中执行 HTML5、WebGL 与 Canvas。",
                                        kind.to_uppercase(),
                                        content.len()
                                    )),
                            )
                            .child(
                                Button::new("artifact-launch-btn")
                                    .h(px(30.))
                                    .rounded(px(6.))
                                    .secondary()
                                    .child(
                                        div()
                                            .text_size(self.ui_text(12.))
                                            .font_medium()
                                            .child(self.tr("🌐 启动独立 Web 视窗运行", "🌐 Launch in Independent Web Window")),
                                    )
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        let temp_dir = std::env::temp_dir();
                                        let file_path = temp_dir.join("volt_artifact_web.html");
                                        if std::fs::write(&file_path, &content_c).is_ok() {
                                            #[cfg(target_os = "macos")]
                                            let _ = std::process::Command::new("open").arg(&file_path).spawn();
                                            #[cfg(target_os = "windows")]
                                            let _ = std::process::Command::new("cmd").args(["/c", "start", file_path.to_str().unwrap()]).spawn();
                                            #[cfg(not(any(target_os = "macos", target_os = "windows")))]
                                            let _ = std::process::Command::new("xdg-open").arg(&file_path).spawn();
                                            this.status_message = format!("已在独立 Web 视窗中启动【{}】", title_c);
                                            cx.notify();
                                        }
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .p_3()
                            .rounded(px(8.))
                            .bg(rgba(colors.inline_code_bg))
                            .overflow_hidden()
                            .child(
                                div()
                                    .text_size(self.ui_text(11.))
                                    .font_family(".AppleSystemUIFontMonospaced")
                                    .text_color(rgba(colors.foreground_subtle))
                                    .child(content),
                            ),
                    )
            }
            ArtifactViewMode::Code => div()
                .flex_1()
                .p_3()
                .bg(rgba(colors.inline_code_bg))
                .overflow_hidden()
                .child(
                    div()
                        .text_size(self.ui_text(12.))
                        .font_family(".AppleSystemUIFontMonospaced")
                        .text_color(rgb(colors.text_primary))
                        .child(content),
                ),
        };

        div()
            .w_full()
            .h_full()
            .v_flex()
            .bg(rgb(colors.root_bg))
            .child(header)
            .child(body)
            .into_any_element()
    }
}

#[cfg(test)]
mod layout_tests {
    use super::{
        command_drawer_height, command_drawer_state, review_panel_state, review_uses_single_column,
        CommandDrawerState, ReviewPanelState,
    };
    use gpui_kit::px;

    #[test]
    fn review_switches_to_a_single_column_below_1180() {
        assert!(review_uses_single_column(px(1179.)));
        assert!(!review_uses_single_column(px(1180.)));
        assert!(!review_uses_single_column(px(1440.)));
    }

    #[test]
    fn command_drawer_is_thirty_percent_of_the_viewport_capped_at_210() {
        assert_eq!(command_drawer_height(px(480.)), px(144.));
        assert_eq!(command_drawer_height(px(760.)), px(210.));
        assert_eq!(command_drawer_height(px(1200.)), px(210.));
    }

    #[test]
    fn review_states_prefer_loading_then_error_then_empty() {
        assert_eq!(review_panel_state(true, true, 3), ReviewPanelState::Loading);
        assert_eq!(review_panel_state(false, true, 3), ReviewPanelState::Error);
        assert_eq!(review_panel_state(false, false, 0), ReviewPanelState::Empty);
        assert_eq!(review_panel_state(false, false, 1), ReviewPanelState::Ready);
    }

    #[test]
    fn command_drawer_distinguishes_empty_running_and_output() {
        assert_eq!(
            command_drawer_state(false, false),
            CommandDrawerState::Empty
        );
        assert_eq!(
            command_drawer_state(false, true),
            CommandDrawerState::Running
        );
        assert_eq!(command_drawer_state(true, true), CommandDrawerState::Ready);
        assert_eq!(command_drawer_state(true, false), CommandDrawerState::Ready);
    }

    #[test]
    fn queue_panel_distinguishes_paused_and_active_count() {
        assert_eq!(
            super::queue_panel_status(true, 5),
            super::QueuePanelStatus::Paused
        );
        assert_eq!(
            super::queue_panel_status(false, 3),
            super::QueuePanelStatus::Active(3)
        );
    }
}
