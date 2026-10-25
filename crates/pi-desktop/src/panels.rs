use super::*;
use crate::workbench::{action, heading, icon_button};
use gpui_kit::assets::IconName as Glyph;
use gpui_kit::component::text::TextView;

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

    pub fn toggle_review(&mut self, cx: &mut Context<Self>) {
        self.review_visible = !self.review_visible;
        if self.review_visible {
            self.refresh_review(cx);
        }
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
        let mut view = div()
            .v_flex()
            .flex_1()
            .min_h_0()
            .h_full()
            .min_w_0()
            .p_3()
            .gap_3()
            .child(
                div()
                    .h_flex()
                    .items_center()
                    .gap_2()
                    .child(heading(self.tr("审查变更", "Review changes")))
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
                            .on_click(cx.listener(|this, _, _, cx| this.toggle_review(cx))),
                    ),
            )
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
                view = view.child(self.review_status_surface(
                    colors,
                    self.tr("正在读取变更…", "Loading changes…"),
                    false,
                ));
            }
            ReviewPanelState::Error => {
                view = view.child(self.review_status_surface(
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
                view = view.child(self.review_status_surface(colors, message, false));
            }
            ReviewPanelState::Ready => {
                let files_scroll = self.review_files_scroll.clone();
                view = view
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
                            .border_color(rgba(colors.settings_border))
                            .overflow_hidden()
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .px_3()
                                    .py_2()
                                    .border_b_1()
                                    .border_color(rgba(colors.settings_border))
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
                            .child(self.review_diff_body()),
                    );
            }
        }
        view.into_any_element()
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
                0xb4231833
            } else {
                colors.settings_border
            }))
            .bg(rgb(colors.card_bg))
            .p_3()
            .text_size(px(13.))
            .text_color(rgba(if error {
                0xb42318
            } else {
                colors.foreground_subtle
            }))
            .child(message.into())
    }

    fn review_diff_body(&self) -> AnyElement {
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
            .child(
                TextView::markdown("review-text", fenced(&self.review.diff, "diff"))
                    .selectable(true),
            )
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
                            0xb4231833
                        } else {
                            colors.settings_border
                        };
                        let text = if tool.is_error {
                            0xb42318
                        } else {
                            colors.foreground_subtle
                        };
                        div()
                            .flex_shrink_0()
                            .rounded(px(8.))
                            .border_1()
                            .border_color(rgba(border))
                            .bg(rgb(colors.root_bg))
                            .p_2()
                            .child(
                                div()
                                    .font_family(cx.theme().mono_font_family.clone())
                                    .text_size(px(14.))
                                    .line_height(px(20.))
                                    .text_color(rgba(text))
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
            .border_color(rgb(colors.card_border))
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
            .border_color(rgba(colors.settings_border))
            .bg(rgb(colors.root_bg))
            .p_3()
            .text_size(px(13.))
            .text_color(rgba(colors.foreground_subtle))
            .child(message.into())
            .into_any_element()
    }

    pub fn queue_view(&self, colors: &ThemeColors, cx: &mut Context<Self>) -> AnyElement {
        let state = self
            .task_states
            .get(&self.session.id)
            .cloned()
            .unwrap_or_default();
        if state.queue.is_empty() {
            return div().into_any_element();
        }
        div()
            .id("pending-queue")
            .v_flex()
            .max_h(px(140.))
            .overflow_y_scroll()
            .px_4()
            .pb_2()
            .gap_1()
            .child(
                div()
                    .h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(rgba(colors.foreground_subtle))
                            .child(format!(
                                "{} · {}",
                                self.tr("待发送", "Queued"),
                                state.queue.len()
                            )),
                    )
                    .child(
                        action("queue-resume", self.tr("继续", "Resume"), Glyph::Play)
                            .disabled(self.is_executing)
                            .on_click(cx.listener(|this, _, _, cx| this.resume_queue(cx))),
                    ),
            )
            .children(state.queue.into_iter().enumerate().map(|(index, prompt)| {
                div()
                    .h_flex()
                    .gap_2()
                    .items_center()
                    .child(div().flex_1().min_w_0().truncate().child(prompt))
                    .child(
                        action(
                            format!("queue-remove-{index}"),
                            self.tr("移除", "Remove"),
                            Glyph::X,
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.task_states
                                .entry(this.session.id.clone())
                                .or_default()
                                .queue
                                .remove(index);
                            this.persist_active_session();
                            cx.notify();
                        })),
                    )
            }))
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
}

fn fenced(text: &str, language: &str) -> String {
    let max_run = text
        .split(|character| character != '`')
        .map(str::len)
        .max()
        .unwrap_or(0);
    let fence = "`".repeat(max_run.max(2) + 1);
    format!("{fence}{language}\n{text}\n{fence}")
}
