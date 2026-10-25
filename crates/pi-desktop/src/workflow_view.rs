use super::*;
use gpui_kit::assets::IconName as Glyph;
use gpui_kit::component::button::{Button, ButtonCustomVariant, ButtonVariants};
use gpui_kit::prelude::FluentBuilder;
use pi_agent_core::workflow::{Node, NodeKind, Stage, Status, WorkflowSnapshot};

/// Persisted disclosure belongs to a chat, including while its detail dialog is open.
#[derive(Clone, Copy)]
struct WorkflowScope<'a> {
    session: &'a str,
    workflow: &'a str,
}
impl WorkflowScope<'_> {
    fn key(self, parts: &[&str]) -> String {
        let mut key = String::from("workflow-v2:");
        // Length prefixes prevent IDs containing separators from colliding.
        for part in [self.session, self.workflow]
            .into_iter()
            .chain(parts.iter().copied())
        {
            use std::fmt::Write;
            write!(key, "{}:{part}", part.len()).expect("write to String");
        }
        key
    }
}

/// Only restore legacy disclosure keys when they identify a single target.
/// Shared legacy IDs cannot be attributed to a particular chat safely.
pub(super) fn migrate_disclosures(saved: &mut desktop_state::SavedDesktop) {
    let mut targets: HashMap<String, Option<String>> = HashMap::new();
    let mut register = |legacy: String, qualified: String| {
        if !saved.expanded_tools.contains(&legacy) {
            return;
        }
        targets
            .entry(legacy)
            .and_modify(|target| {
                if target.as_ref() != Some(&qualified) {
                    *target = None;
                }
            })
            .or_insert(Some(qualified));
    };
    for (session, state) in &saved.task_states {
        for workflow in &state.workflows {
            let scope = WorkflowScope {
                session,
                workflow: &workflow.id,
            };
            for kind in ["collapse", "delivery", "changes"] {
                register(
                    format!("workflow-{kind}-{}", workflow.id),
                    scope.key(&[kind]),
                );
            }
            for stage in &workflow.stages {
                for node in &stage.nodes {
                    register(
                        format!("workflow-node-{}-{}-{}", workflow.id, stage.id, node.id),
                        scope.key(&["node", &stage.id, &node.id]),
                    );
                }
            }
        }
    }
    for (legacy, target) in targets {
        // Consume recognized old keys once, so a later collapse stays collapsed.
        saved.expanded_tools.remove(&legacy);
        if let Some(target) = target {
            saved.expanded_tools.insert(target);
        }
    }
}

fn status_label(status: Status, chinese: bool) -> &'static str {
    match (status, chinese) {
        (Status::Waiting, true) => "等待中",
        (Status::Waiting, false) => "Waiting",
        (Status::Running, true) => "运行中",
        (Status::Running, false) => "Running",
        (Status::Completed, true) => "已完成",
        (Status::Completed, false) => "Completed",
        (Status::Failed, true) => "失败",
        (Status::Failed, false) => "Failed",
        (Status::Interrupted, true) => "已中断",
        (Status::Interrupted, false) => "Interrupted",
    }
}

fn workflow_status_label(status: Status, chinese: bool) -> &'static str {
    match (status, chinese) {
        (Status::Waiting, true) => "工作流已启动",
        (Status::Waiting, false) => "Workflow started",
        (Status::Running, true) => "工作流运行中",
        (Status::Running, false) => "Workflow running",
        (Status::Completed, true) => "工作流已完成",
        (Status::Completed, false) => "Workflow completed",
        (Status::Failed, true) => "工作流出错",
        (Status::Failed, false) => "Workflow errored",
        (Status::Interrupted, true) => "工作流已停止",
        (Status::Interrupted, false) => "Workflow stopped",
    }
}

/// The digest card itself is interactive only when the stage rail exists.
fn digest_pointer_toggles(has_rail: bool, keyboard_click: bool) -> bool {
    has_rail && !keyboard_click
}

/// Enter/Space activate the card only while the card itself owns focus.
/// Descendant controls keep their own keyboard activation.
fn digest_key_toggles(
    has_rail: bool,
    card_focused: bool,
    key: &str,
    modifiers_modified: bool,
) -> bool {
    has_rail && card_focused && !modifiers_modified && matches!(key, "enter" | "space")
}

/// Match the reference `focus-visible:ring-2 ring-ring/40`: a neutral 2px
/// band outside the card, never the warning colour and never a replacement
/// surface fill. The ring token itself is already light/dark aware.
fn digest_focus_ring(ring: Hsla) -> Vec<BoxShadow> {
    vec![BoxShadow::new(px(0.), px(0.), ring.alpha(ring.a * 0.4))
        .blur_radius(px(0.))
        .spread_radius(px(2.))]
}

const WORKFLOW_STAGE_HEADING_HEIGHT: f32 = 24.;
const WORKFLOW_STAGE_TO_PILL_GAP: f32 = 8.;
const WORKFLOW_AGENT_PILL_HEIGHT: f32 = 32.;
const WORKFLOW_AGENT_PILL_PADDING_LEFT: f32 = 8.;
const WORKFLOW_AGENT_PILL_PADDING_RIGHT: f32 = 10.;

#[derive(Clone, Copy, Debug, PartialEq)]
struct WorkflowStageGeometry {
    heading_height: f32,
    stage_to_pill_gap: f32,
    agent_pill_height: f32,
    agent_pill_padding_left: f32,
    agent_pill_padding_right: f32,
}

fn workflow_stage_geometry() -> WorkflowStageGeometry {
    WorkflowStageGeometry {
        heading_height: WORKFLOW_STAGE_HEADING_HEIGHT,
        stage_to_pill_gap: WORKFLOW_STAGE_TO_PILL_GAP,
        agent_pill_height: WORKFLOW_AGENT_PILL_HEIGHT,
        agent_pill_padding_left: WORKFLOW_AGENT_PILL_PADDING_LEFT,
        agent_pill_padding_right: WORKFLOW_AGENT_PILL_PADDING_RIGHT,
    }
}

impl PiDesktopApp {
    fn workflow_card_focus(&self, key: String, cx: &mut Context<Self>) -> FocusHandle {
        self.workflow_card_focus
            .borrow_mut()
            .entry(key)
            .or_insert_with(|| cx.focus_handle().tab_stop(true))
            .clone()
    }

    pub(super) fn workflow_card(
        &self,
        workflow: &WorkflowSnapshot,
        c: &ThemeColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.render_workflow_card(&self.session.id, workflow, c, false, cx)
    }

    fn render_workflow_card(
        &self,
        origin_session_id: &str,
        workflow: &WorkflowSnapshot,
        c: &ThemeColors,
        expanded_view: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let workflow_id = workflow.id.clone();
        let session_id = origin_session_id.to_owned();
        let scope = WorkflowScope {
            session: origin_session_id,
            workflow: &workflow.id,
        };
        let chinese = self.language == Language::Chinese;
        let collapse_key = scope.key(&["collapse"]);
        let collapsed = self.expanded_tools.contains(&collapse_key);
        let card_collapse_key = collapse_key.clone();
        let has_rail = !workflow.stages.is_empty();
        let card_focus = has_rail.then(|| {
            self.workflow_card_focus(
                scope.key(&["card-focus", if expanded_view { "dialog" } else { "chat" }]),
                cx,
            )
        });
        let summary = format!(
            "{} · {}",
            workflow_status_label(workflow.status, chinese),
            workflow.title
        );
        let border: Hsla = rgba(c.settings_border).into();
        let mut card = div()
            .id(scope.key(&["card", if expanded_view { "dialog" } else { "chat" }]))
            .w_full()
            .min_w_0()
            .v_flex()
            .gap_1()
            .px(px(14.))
            .pt(px(6.))
            .pb(px(8.))
            .rounded(px(12.))
            .border_1()
            .border_color(border.alpha(border.a * 0.7))
            .bg(rgb(c.workflow_bg))
            .child(
                div()
                    .h_flex()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .child(
                        // The reference header is ordinary content, not a
                        // second collapse control. With a rail, blank header
                        // space bubbles to the card; without one there is no
                        // card toggle at all.
                        div()
                            .h_flex()
                            .flex_shrink_0()
                            .items_center()
                            .gap_2()
                            .text_size(px(14.))
                            .font_medium()
                            .text_color(rgb(c.text_primary))
                            .child(
                                Icon::new(Glyph::Workflow)
                                    .size(px(16.))
                                    .text_color(rgba(c.foreground_subtle)),
                            )
                            // The digest header names the run status once; the
                            // reference call passes neither a status slot nor a
                            // second toggle to WorkflowCardHeader.
                            .child(workflow_status_label(workflow.status, chinese)),
                    )
                    .child(
                        div()
                            .min_w(px(80.))
                            .flex_1()
                            .truncate()
                            .text_size(px(14.))
                            .text_color(rgba(c.foreground_subtle))
                            .child(workflow.title.clone()),
                    )
                    .child(
                        div()
                            .h_flex()
                            .items_center()
                            .gap_2()
                            .min_w_0()
                            .ml_auto()
                            .child(
                                div()
                                    .min_w_0()
                                    .flex_shrink_1()
                                    .truncate()
                                    .text_size(px(14.))
                                    .text_color(rgba(c.foreground_subtlest))
                                    .child(format!(
                                        "{} {} · {} {} · {}",
                                        workflow.stages.len(),
                                        self.tr("个阶段", "stages"),
                                        workflow.running_agents(),
                                        self.tr("个子代理工作中", "agents running"),
                                        workflow.model
                                    )),
                            )
                            .when(workflow.status == Status::Running, |actions| {
                                let workflow_id = workflow.id.clone();
                                let stop_state = if origin_session_id == self.session.id {
                                    self.workflow_controls.stopping(&workflow.id)
                                } else {
                                    None
                                };
                                let stop_label = if stop_state == Some(true) {
                                    self.tr("正在停止…", "Stopping…")
                                } else {
                                    self.tr("停止工作流", "Stop workflow")
                                };
                                actions.child(
                                    Button::new(format!("workflow-stop-{}", workflow.id))
                                        .ghost()
                                        .small()
                                        .w(px(28.))
                                        .h(px(28.))
                                        .disabled(stop_state != Some(false))
                                        .tooltip(stop_label)
                                        .accessibility_label(stop_label)
                                        .child(
                                            div()
                                                .size(px(11.))
                                                .rounded(px(2.))
                                                .bg(rgb(c.text_primary)),
                                        )
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.stop_workflow(workflow_id.clone(), cx);
                                            cx.stop_propagation();
                                        })),
                                )
                            })
                            .when(!expanded_view, |actions| {
                                actions.child(
                                    Button::new(format!("workflow-expand-{}", workflow.id))
                                        .ghost()
                                        .small()
                                        .w(px(28.))
                                        .h(px(28.))
                                        .tooltip(self.tr("展开工作流", "Expand workflow"))
                                        .accessibility_label(
                                            self.tr("展开工作流", "Expand workflow"),
                                        )
                                        .child(Icon::new(Glyph::Maximize2).size(px(14.)))
                                        .on_click(cx.listener(move |this, _, window, cx| {
                                            let app = cx.entity();
                                            let workflow_id = workflow_id.clone();
                                            let session_id = session_id.clone();
                                            let title = this.tr("工作流详情", "Workflow details");
                                            let detail = cx.new(|cx| WorkflowDetail {
                                                app: app.downgrade(),
                                                workflow_id,
                                                session_id,
                                                _subscription: cx
                                                    .observe(&app, |_, _, cx| cx.notify()),
                                            });
                                            window.open_dialog(cx, move |dialog, window, _| {
                                                dialog
                                                    .title(title)
                                                    .width(
                                                        (window.viewport_size().width - px(48.))
                                                            .min(px(1152.)),
                                                    )
                                                    .close_button(true)
                                                    .keyboard(true)
                                                    .child(
                                                        div()
                                                            .id("workflow-detail-scroll")
                                                            .w_full()
                                                            .min_w_0()
                                                            .max_h(
                                                                (window.viewport_size().height
                                                                    - px(160.))
                                                                .max(px(160.)),
                                                            )
                                                            .overflow_y_scroll()
                                                            .on_scroll_wheel(|_, _, cx| {
                                                                cx.stop_propagation()
                                                            })
                                                            .child(detail.clone()),
                                                    )
                                            });
                                            cx.stop_propagation();
                                        })),
                                )
                            }),
                    ),
            )
            .when(has_rail, |card| {
                let focus = card_focus
                    .as_ref()
                    .expect("rail cards have a focus handle")
                    .clone();
                let key = card_collapse_key.clone();
                let click_key = key.clone();
                let focus_ring = cx.theme().ring;
                card.track_focus(&focus)
                    .role(Role::Button)
                    .aria_label(summary.clone())
                    .aria_expanded(!collapsed)
                    .cursor_pointer()
                    .focus_visible(move |style| style.shadow(digest_focus_ring(focus_ring)))
                    .on_click(cx.listener(move |this, event: &gpui::ClickEvent, _, cx| {
                        if digest_pointer_toggles(has_rail, event.is_keyboard()) {
                            this.toggle_tool_expansion(&click_key);
                            cx.notify();
                        }
                    }))
                    .on_key_down(cx.listener(
                        move |this, event: &gpui::KeyDownEvent, window, cx| {
                            let stroke = &event.keystroke;
                            if digest_key_toggles(
                                has_rail,
                                focus.is_focused(window),
                                &stroke.key,
                                stroke.modifiers.modified(),
                            ) {
                                window.prevent_default();
                                cx.stop_propagation();
                                this.toggle_tool_expansion(&key);
                                cx.notify();
                            }
                        },
                    ))
            });
        // The digest always keeps its stage rail visible. Collapsing hides only
        // node rows, so progress and deliverables remain available during a run.
        if !workflow.stages.is_empty() {
            card = card.child(self.workflow_timeline(
                scope,
                workflow,
                !collapsed,
                expanded_view,
                c,
                cx,
            ));
        }
        if !workflow.delivery.is_empty() {
            let key = scope.key(&["delivery"]);
            let expanded = self.expanded_tools.contains(&key);
            card = card
                .child(
                    crate::workbench::action(
                        key.clone(),
                        self.tr("交付看板", "Deliverables"),
                        Glyph::SquareKanban,
                    )
                    .w(px(112.))
                    .h(px(32.))
                    .bg(rgba(c.workflow_node_bg))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.toggle_tool_expansion(&key);
                        cx.stop_propagation();
                        cx.notify();
                    })),
                )
                .when(expanded, |card| {
                    let content_key = scope.key(&["delivery-content"]);
                    card.child(
                        div()
                            .id(scope.key(&["delivery-scroll"]))
                            .w_full()
                            .p_2()
                            .max_h(px(160.))
                            .min_w_0()
                            .overflow_y_scroll()
                            .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| {
                                cx.stop_propagation()
                            })
                            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                            .child(gpui_kit::base::SelectableText::new(
                                content_key,
                                workflow.delivery.clone(),
                            )),
                    )
                });
        }
        div()
            .w_full()
            .min_w_0()
            .v_flex()
            .gap_5()
            .child(card)
            .when(!workflow.changes.is_empty(), |column| {
                column.child(self.workflow_changes(scope, workflow, c, cx))
            })
            .into_any_element()
    }

    fn workflow_timeline(
        &self,
        scope: WorkflowScope<'_>,
        workflow: &WorkflowSnapshot,
        show_nodes: bool,
        expanded_view: bool,
        c: &ThemeColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        use crate::workflow_timeline::{fold_stations, ledge_width, FADE};
        // A dialog and its originating chat must not fight over one viewport.
        let key = scope.key(&["viewport", if expanded_view { "dialog" } else { "chat" }]);
        let viewport = {
            let mut handles = self.workflow_scroll.borrow_mut();
            if !handles.contains_key(&key) {
                // Retain all viewports whose workflow still exists. Arbitrary
                // capacity eviction can reset mounted cards in long transcripts.
                let live: Vec<_> = self
                    .task_states
                    .iter()
                    .flat_map(|(session, state)| {
                        state.workflows.iter().map(move |workflow| {
                            WorkflowScope {
                                session,
                                workflow: &workflow.id,
                            }
                            .key(&["viewport"])
                        })
                    })
                    .collect();
                let current = scope.key(&["viewport"]);
                handles.retain(|stored, _| {
                    stored.starts_with(&current)
                        || live.iter().any(|prefix| stored.starts_with(prefix))
                });
            }
            handles.entry(key.clone()).or_default().clone()
        };
        let scroll = viewport.scroll.clone();
        let running_index = workflow
            .stages
            .iter()
            .rposition(|stage| stage.status == Status::Running);
        viewport.reset_drag_if_invalid(
            f32::from(scroll.bounds().size.width),
            f32::from(scroll.max_offset().x),
        );
        viewport.prepare_camera(running_index);
        let width = f32::from(scroll.bounds().size.width);
        let offset = -f32::from(scroll.offset().x);
        let maximum = f32::from(scroll.max_offset().x);
        let fold = fold_stations(workflow.stages.len(), offset, width);
        let entity_id = cx.entity_id();
        let measured_scroll = scroll.clone();
        let wheel_scroll = scroll.clone();
        let wheel_viewport = viewport.clone();
        let camera_viewport = viewport.clone();
        let hover_viewport = viewport.clone();
        let measured_width = width;
        let measured_offset = offset;
        let measured_maximum = maximum;
        let mut timeline = div()
            .id(scope.key(&["timeline", if expanded_view { "dialog" } else { "chat" }]))
            .relative()
            .w_full()
            .min_w_0()
            // 4px outer breathing + the reference's 6px air above the rail.
            .pt(px(10.))
            .pb(px(12.))
            .on_hover(cx.listener(move |_, hovered: &bool, _, cx| {
                hover_viewport.set_hovered(*hovered);
                cx.notify();
            }))
            .child(
                div()
                    .id(key.clone())
                    .w_full()
                    .min_w_0()
                    .overflow_x_scroll()
                    .track_scroll(&scroll)
                    .on_scroll_wheel(cx.listener(
                        move |_, event: &gpui::ScrollWheelEvent, window, cx| {
                            let delta = event.delta.pixel_delta(window.line_height());
                            // A horizontal-only timeline consumes the gesture only when
                            // it currently has horizontal range. With no range, let the
                            // transcript/detail dialog receive vertical scrolling.
                            let horizontal_range = wheel_scroll.max_offset().x > px(0.);
                            if horizontal_range && (!delta.x.is_zero() || !delta.y.is_zero()) {
                                wheel_viewport.mark_user_interaction();
                                cx.stop_propagation();
                                cx.notify();
                            }
                        },
                    ))
                    .child(div().h_flex().items_start().gap_6().children(
                        workflow.stages.iter().enumerate().map(|(index, stage)| {
                            self.workflow_stage(
                                scope,
                                stage,
                                index + 1 < workflow.stages.len(),
                                show_nodes,
                                !fold.left.contains(&index) && !fold.right.contains(&index),
                                c,
                                cx,
                            )
                        }),
                    )),
            )
            // Scroll bounds are populated during prepaint. A size change needs
            // one further frame to recompute folded stations, including resize.
            .child(
                gpui::canvas(
                    |_, _, _| (),
                    move |_, _, window, cx| {
                        let current_width = f32::from(measured_scroll.bounds().size.width);
                        let current_offset = -f32::from(measured_scroll.offset().x);
                        let current_maximum = f32::from(measured_scroll.max_offset().x);
                        // Use post-layout bounds, including a newly added stage.
                        // Subsequent renders of the same running stage preserve
                        // the user's manual scroll position.
                        camera_viewport.reset_drag_if_invalid(current_width, current_maximum);
                        if let Some(target) =
                            camera_viewport.camera_target(current_width, current_maximum)
                        {
                            measured_scroll
                                .set_offset(point(px(-target), measured_scroll.offset().y));
                            cx.notify(entity_id);
                        }
                        // GPUI stores these listeners in the next frame and clears
                        // them on frame reuse; registering during paint is required.
                        let pointer_viewport = camera_viewport.clone();
                        window.on_mouse_event(move |_: &gpui::MouseDownEvent, phase, _, _| {
                            if phase.capture() {
                                pointer_viewport.begin_pointer_sequence();
                            }
                        });
                        let touch_viewport = camera_viewport.clone();
                        window.on_mouse_event(move |event: &gpui::TouchDragEvent, phase, _, _| {
                            if phase.capture() && event.phase == gpui::TouchPhase::Started {
                                touch_viewport.begin_pointer_sequence();
                            }
                        });
                        if (current_width - measured_width).abs() > 0.5
                            || (current_offset - measured_offset).abs() > 0.5
                            || (current_maximum - measured_maximum).abs() > 0.5
                        {
                            cx.notify(entity_id);
                        }
                    },
                )
                .absolute()
                .size_full(),
            );
        for (left, indexes) in [(true, &fold.left), (false, &fold.right)] {
            let overflow = if left {
                offset > 0.5
            } else {
                maximum - offset > 0.5
            };
            if !overflow && indexes.is_empty() {
                continue;
            }
            // GPUI has no CSS mask: composite the same card surface over the
            // outer 40px. Ledge masks affect only the 24px rail, not the pills.
            let surface: Hsla = rgb(c.workflow_bg).into();
            let fade = |rail: bool| {
                let extra = if rail && !indexes.is_empty() {
                    ledge_width(indexes.len()) + 8.
                } else {
                    0.
                };
                div()
                    .absolute()
                    .top(px(10.))
                    .when(rail, |d| d.h(px(24.)))
                    .when(!rail, |d| d.top(px(34.)).bottom(px(12.)))
                    .w(px(FADE))
                    .when(left, |d| d.left(px(extra)))
                    .when(!left, |d| d.right(px(extra)))
                    .bg(gpui::linear_gradient(
                        if left { 90. } else { 270. },
                        gpui::linear_color_stop(surface, 0.),
                        gpui::linear_color_stop(surface.alpha(0.), 1.),
                    ))
            };
            timeline = timeline.child(fade(false)).child(fade(true));
            if !indexes.is_empty() {
                timeline = timeline
                    .child(self.workflow_ledge(scope, workflow, indexes, left, &viewport, c, cx));
            }
        }
        if maximum > 0. {
            timeline = timeline.child(viewport.scrollbar(scope.key(&["timeline-scrollbar"]), c));
        }
        timeline.into_any_element()
    }

    fn workflow_ledge(
        &self,
        scope: WorkflowScope<'_>,
        workflow: &WorkflowSnapshot,
        indexes: &[usize],
        left: bool,
        viewport: &crate::workflow_timeline::Viewport,
        c: &ThemeColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let more = indexes.len().saturating_sub(3);
        let keyboard_viewport = viewport.clone();
        let shown = if left {
            &indexes[more..]
        } else {
            &indexes[..indexes.len().min(3)]
        };
        let count = || {
            div()
                .w(px(26.))
                .flex_shrink_0()
                .text_size(px(11.))
                .text_color(rgba(c.foreground_subtlest))
                .child(format!("+{more}"))
        };
        div()
            .id(scope.key(&["ledge-keys", if left { "left" } else { "right" }]))
            .absolute()
            .top(px(10.))
            .h(px(24.))
            .when(left, |d| d.left_0())
            .when(!left, |d| d.right_0())
            .w(px(crate::workflow_timeline::ledge_width(indexes.len())))
            .px(px(8.))
            .h_flex()
            .items_center()
            .bg(rgb(c.workflow_bg))
            .on_key_down(cx.listener(move |_, event: &gpui::KeyDownEvent, _, cx| {
                let modifiers = &event.keystroke.modifiers;
                if modifiers.control || modifiers.alt || modifiers.platform {
                    return;
                }
                let scroll = &keyboard_viewport.scroll;
                if let Some(target) = crate::workflow_timeline::keyboard_stage_offset(
                    &event.keystroke.key,
                    -f32::from(scroll.offset().x),
                    f32::from(scroll.max_offset().x),
                ) {
                    keyboard_viewport.mark_user_interaction();
                    scroll.set_offset(point(px(-target), scroll.offset().y));
                    cx.stop_propagation();
                    cx.notify();
                }
            }))
            .when(left && more > 0, |d| d.child(count()))
            .children(shown.iter().enumerate().map(|(position, &index)| {
                let stage = &workflow.stages[index];
                let handle = viewport.scroll.clone();
                let manual_viewport = viewport.clone();
                let label = format!(
                    "{} · {}",
                    stage.label,
                    status_label(stage.status, self.language == Language::Chinese)
                );
                div()
                    .h_flex()
                    .items_center()
                    .flex_shrink_0()
                    .when(position > 0, |row| {
                        row.child(div().w(px(6.)).h(px(1.)).bg(rgba(c.foreground_subtlest)))
                    })
                    .child(
                        Button::new(scope.key(&[
                            "ledge",
                            if left { "left" } else { "right" },
                            &stage.id,
                        ]))
                        .ghost()
                        .small()
                        .p_0()
                        .w(px(10.))
                        .min_w(px(10.))
                        .h(px(10.))
                        .rounded_full()
                        .accessibility_label(label.clone())
                        .tooltip(label)
                        .child(stage_marker(
                            stage.status,
                            c,
                            self.preferences.reduced_motion,
                        ))
                        .on_click(cx.listener(move |_, _, _, cx| {
                            manual_viewport.mark_user_interaction();
                            let target = crate::workflow_timeline::camera_left(
                                index,
                                f32::from(handle.bounds().size.width),
                                f32::from(handle.max_offset().x),
                            );
                            handle.set_offset(point(px(-target), handle.offset().y));
                            cx.stop_propagation();
                            cx.notify();
                        })),
                    )
            }))
            .when(!left && more > 0, |d| d.child(count()))
            .into_any_element()
    }

    fn workflow_stage(
        &self,
        scope: WorkflowScope<'_>,
        stage: &Stage,
        has_next: bool,
        show_nodes: bool,
        show_heading: bool,
        c: &ThemeColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let geometry = workflow_stage_geometry();
        div()
            .relative()
            .w(px(168.))
            .flex_shrink_0()
            .v_flex()
            // The reference leaves 8px between the 24px station row and pills.
            .gap(px(geometry.stage_to_pill_gap))
            .child(
                div()
                    .h(px(geometry.heading_height))
                    .w(px(if has_next { 192. } else { 168. }))
                    .h_flex()
                    .gap_0()
                    .items_center()
                    // Only the lamp, name and progress fold. The rail is a
                    // sibling so a station hidden under a ledge never cuts the
                    // track between its open neighbours.
                    .child(
                        div()
                            .h_flex()
                            .items_center()
                            .when(!show_heading, |header| header.opacity(0.))
                            .child(
                                // Lamp centre sits at 17px; the reference's
                                // 12px margins around the 10px lamp put the
                                // name at 34px.
                                div()
                                    .w(px(34.))
                                    .pl(px(9.))
                                    .flex_shrink_0()
                                    .h_flex()
                                    .justify_start()
                                    .child(stage_marker(
                                        stage.status,
                                        c,
                                        self.preferences.reduced_motion,
                                    )),
                            )
                            .child(
                                div()
                                    .flex_shrink_1()
                                    .min_w_0()
                                    .truncate()
                                    .text_size(px(13.))
                                    .font_medium()
                                    .text_color(if stage.status == Status::Waiting {
                                        rgba(c.foreground_subtle)
                                    } else {
                                        rgb(c.text_primary)
                                    })
                                    .child(stage.label.clone()),
                            )
                            .when_some(stage.visible_progress(), |header, progress| {
                                header.child(
                                    div()
                                        .ml(px(8.))
                                        .flex_shrink_0()
                                        .text_size(px(12.))
                                        .font_family(cx.theme().mono_font_family.clone())
                                        .text_color(rgba(c.foreground_subtlest))
                                        .child(format!(
                                            "{}/{}",
                                            progress.settled, progress.observed
                                        )),
                                )
                            }),
                    )
                    .when(has_next, |header| {
                        header.child(
                            div()
                                .flex_1()
                                .min_w(px(12.))
                                .ml(px(10.))
                                .mr(px(-6.))
                                .h(px(1.))
                                .bg(rgb(c.workflow_connector)),
                        )
                    }),
            )
            .when(show_nodes, |column| {
                // Header-to-pills is 8px; pills themselves stay on the
                // reference's tighter 6px rhythm.
                column.child(
                    div().w_full().v_flex().gap(px(6.)).children(
                        stage
                            .nodes
                            .iter()
                            .map(|node| self.workflow_node(scope, stage, node, geometry, c, cx)),
                    ),
                )
            })
            .into_any_element()
    }

    fn workflow_node(
        &self,
        scope: WorkflowScope<'_>,
        stage: &Stage,
        node: &Node,
        geometry: WorkflowStageGeometry,
        c: &ThemeColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let chinese = self.language == Language::Chinese;
        let key = scope.key(&["node", &stage.id, &node.id]);
        let expanded = self.expanded_tools.contains(&key);
        let label = format!("{} · {}", node.label, status_label(node.status, chinese));
        div()
            .w_full()
            .min_w_0()
            .v_flex()
            .gap_1()
            .child(
                Button::new(key.clone())
                    .custom(
                        ButtonCustomVariant::new(cx)
                            .color(rgba(c.workflow_node_bg).into())
                            .hover(rgba(c.workflow_node_hover).into())
                            .active(rgba(c.workflow_node_hover).into()),
                    )
                    .small()
                    .w_full()
                    .h(px(geometry.agent_pill_height))
                    .pl(px(geometry.agent_pill_padding_left))
                    .pr(px(geometry.agent_pill_padding_right))
                    .rounded(px(16.))
                    .focus(|style| style.bg(rgba(c.workflow_node_hover)))
                    .justify_start()
                    .accessibility_label(label.clone())
                    .tooltip(label)
                    .child(
                        div()
                            .h_flex()
                            .items_center()
                            .gap_2()
                            .w_full()
                            .min_w_0()
                            .child(role_badge(node, c, self.preferences.reduced_motion))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    .text_size(px(12.))
                                    .text_color(if node.status == Status::Waiting {
                                        rgba(c.foreground_subtle)
                                    } else {
                                        rgb(c.text_primary)
                                    })
                                    .child(node.label.clone()),
                            )
                            .when(node.status != Status::Waiting, |row| {
                                row.child(node_status_mark(
                                    node.status,
                                    self.preferences.reduced_motion,
                                    c,
                                ))
                            }),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.toggle_tool_expansion(&key);
                        cx.stop_propagation();
                        cx.notify();
                    })),
            )
            .when(expanded, |row| {
                row.child(
                    div()
                        .id(scope.key(&["node-detail-scroll", &stage.id, &node.id]))
                        .w_full()
                        .p_2()
                        .min_w_0()
                        .max_h(px(160.))
                        .overflow_y_scroll()
                        .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                        .text_size(px(12.))
                        .child(gpui_kit::base::SelectableText::new(
                            scope.key(&["node-detail", &stage.id, &node.id]),
                            if node.detail.is_empty() {
                                status_label(node.status, chinese).to_string()
                            } else {
                                node.detail.clone()
                            },
                        )),
                )
            })
            .into_any_element()
    }

    fn workflow_changes(
        &self,
        scope: WorkflowScope<'_>,
        workflow: &WorkflowSnapshot,
        c: &ThemeColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let key = scope.key(&["changes"]);
        let expanded = self.expanded_tools.contains(&key);
        let (added, removed, uncounted) = workflow.change_totals();
        let label = format!(
            "{} {}",
            workflow.changes.len(),
            self.tr("个文件已更改", "files changed")
        );
        div()
            .w_full()
            .min_w_0()
            .v_flex()
            .rounded(px(12.))
            .border_1()
            .border_color(rgba(c.settings_border))
            .bg(rgba(c.workflow_node_bg))
            .child(
                div()
                    .h_flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .min_h(px(40.))
                    .child(
                        Button::new(key.clone())
                            .ghost()
                            .small()
                            .min_w_0()
                            .accessibility_label(label.clone())
                            .tooltip(self.tr("展开或收起文件列表", "Toggle changed files"))
                            .child(
                                Icon::new(if expanded {
                                    Glyph::ChevronDown
                                } else {
                                    Glyph::ChevronRight
                                })
                                .size(px(12.)),
                            )
                            .child(label)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.toggle_tool_expansion(&key);
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(rgb(c.accent_green))
                            .child(format!("+{added}")),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(rgb(c.workflow_error))
                            .child(format!("-{removed}")),
                    )
                    .when(uncounted > 0, |row| {
                        row.child(
                            div()
                                .text_size(px(12.))
                                .text_color(rgba(c.foreground_subtle))
                                .child(format!(
                                    "{uncounted} {}",
                                    self.tr("项未统计行数", "uncounted")
                                )),
                        )
                    })
                    .child(div().flex_1())
                    .child(
                        Button::new(format!("workflow-undo-{}", workflow.id))
                            .ghost()
                            .small()
                            .disabled(true)
                            .child(Icon::new(Glyph::Undo2).size(px(12.)))
                            .child(self.tr("撤销", "Undo"))
                            .tooltip(self.tr(
                                "尚未接入此工作流的变更撤销",
                                "Workflow-scoped undo is not connected",
                            ))
                            .accessibility_label(self.tr("撤销（暂不可用）", "Undo (unavailable)")),
                    ),
            )
            .when(expanded, |view| {
                view.child(div().v_flex().px_3().pb_3().gap_2().children(
                    workflow.changes.iter().map(|file| {
                        div()
                            .h_flex()
                            .items_start()
                            .gap_2()
                            .child(Icon::new(Glyph::File).size(px(14.)))
                            .child(div().flex_1().min_w_0().child(file.path.clone()))
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .text_color(rgba(c.foreground_subtle))
                                    .child(match (file.added, file.removed) {
                                        (Some(a), Some(r)) => format!("+{a} -{r}"),
                                        _ => self.tr("未统计行数", "Uncounted").into(),
                                    }),
                            )
                    }),
                ))
            })
            .into_any_element()
    }
}

fn stage_marker(status: Status, c: &ThemeColors, reduced_motion: bool) -> AnyElement {
    let color = match status {
        Status::Running => c.workflow_active,
        Status::Completed => c.accent_green,
        Status::Failed => c.workflow_error,
        Status::Waiting | Status::Interrupted => c.workflow_connector,
    };
    let hollow = matches!(status, Status::Waiting | Status::Interrupted);
    div()
        .w(px(16.))
        .h(px(16.))
        .flex_shrink_0()
        .h_flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(if status == Status::Failed { 14. } else { 16. }))
                .h(px(if status == Status::Failed { 14. } else { 16. }))
                .rounded_full()
                .h_flex()
                .items_center()
                .justify_center()
                .when(status == Status::Running, |ring| {
                    ring.bg(rgba((color << 8) | 0x33))
                })
                .when(status == Status::Failed, |ring| {
                    ring.bg(rgba((color << 8) | 0x4d))
                })
                .child({
                    let dot = div()
                        .w(px(10.))
                        .h(px(10.))
                        .rounded_full()
                        .when(hollow, |dot| dot.border(px(1.5)).border_color(rgb(color)))
                        .when(!hollow, |dot| dot.bg(rgb(color)));
                    if status == Status::Running && !reduced_motion {
                        dot.with_animation(
                            "workflow-stage-running",
                            Animation::new(std::time::Duration::from_millis(1600)).repeat(),
                            move |dot, progress| {
                                // Match the reference's 0/12/40 percent beat landmarks.
                                // The enclosing 16px slot stays fixed to avoid moving labels.
                                let scale = if progress < 0.12 {
                                    1.0 + 0.12 * gpui::ease_in_out(progress / 0.12)
                                } else if progress < 0.4 {
                                    1.12 - 0.12 * gpui::ease_in_out((progress - 0.12) / 0.28)
                                } else {
                                    1.0
                                };
                                let wave = (progress / 0.7).min(1.0);
                                let alpha = (0.35 * (1.0 - wave) * 255.0).round() as u32;
                                dot.w(px(10.0 * scale)).h(px(10.0 * scale)).shadow(vec![
                                    BoxShadow::new(
                                        px(0.),
                                        px(0.),
                                        rgba((color << 8) | alpha).into(),
                                    )
                                    .spread_radius(px(3.0 + 5.0 * wave)),
                                ])
                            },
                        )
                        .into_any_element()
                    } else {
                        dot.into_any_element()
                    }
                }),
        )
        .into_any_element()
}

fn role_badge(node: &Node, c: &ThemeColors, reduced_motion: bool) -> AnyElement {
    let badge = div()
        .w(px(20.))
        .h(px(20.))
        .rounded(px(7.))
        .flex_shrink_0()
        .h_flex()
        .items_center()
        .justify_center()
        .gap(px(3.));
    if node.kind == NodeKind::Script {
        return badge
            .w(px(16.))
            .h(px(16.))
            .child(
                Icon::new(Glyph::Terminal)
                    .size(px(16.))
                    .text_color(rgba(c.foreground_subtle)),
            )
            .into_any_element();
    }
    // ZCode uses an executor-provided avatar index when available and falls back
    // to the role name hash for legacy snapshots.
    let color = c.workflow_avatars[avatar_slot(node.avatar_index, &node.label)];
    let status = node.status;
    let face = face_canvas(status, 0.0);
    let face = if !reduced_motion
        && matches!(
            status,
            Status::Waiting | Status::Running | Status::Completed | Status::Failed
        ) {
        let animation = Animation::new(std::time::Duration::from_millis(match status {
            Status::Running => 4640,
            Status::Completed => 8530,
            Status::Failed => 8000,
            _ => 2400,
        }));
        let animation = animation.repeat();
        face.with_animation(
            format!("workflow-face-{}-{status:?}", node.id),
            animation,
            move |_, progress| face_canvas(status, progress),
        )
        .into_any_element()
    } else {
        face.into_any_element()
    };
    badge.bg(rgb(color)).child(face).into_any_element()
}

fn face_canvas(status: Status, progress: f32) -> impl IntoElement {
    gpui::canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            // The reference face is a 20-unit tile. Motion is deliberately
            // bounded to eye-level so the badge never changes its layout box.
            let cycle_ms = match status {
                Status::Completed => 8530.,
                Status::Failed => 8000.,
                _ => 4640.,
            };
            let time = progress * cycle_ms;
            let scanning_phase = time % 2320.;
            let leftward = time < 2320.;
            let ease = gpui_kit::base::animation::cubic_bezier(0.42, 0., 0.58, 1.);
            // CSS keyframes interpolate each segment independently.
            let keyed = |t: f32, frames: &[(f32, f32)], smooth: bool| {
                frames
                    .windows(2)
                    .find_map(|pair| {
                        let [(start, from), (end, to)] = pair else {
                            unreachable!()
                        };
                        (t >= *start && t <= *end).then(|| {
                            let fraction = (t - start) / (end - start);
                            from + (to - from) * if smooth { ease(fraction) } else { fraction }
                        })
                    })
                    .unwrap_or(0.)
            };
            let failed_time = time % 4000.;
            let shake = status == Status::Failed
                && time >= 4000.
                && (2280.0..2550.0).contains(&failed_time);
            let terminal_y = match status {
                Status::Completed if (1500.0..1980.0).contains(&time) => keyed(
                    ((time - 1500.) % 240.) / 240.,
                    &[(0., 0.), (0.35, -1.), (0.7, 0.7), (1., 0.)],
                    true,
                ),
                Status::Failed if (1500.0..2100.0).contains(&failed_time) => keyed(
                    (failed_time - 1500.) / 600.,
                    &[(0., 0.), (0.3, -0.5), (0.7, 0.5), (1., 0.)],
                    true,
                ),
                Status::Failed if shake => keyed(
                    ((failed_time - 2280.) % 90.) / 90.,
                    &[(0., 0.), (0.25, -0.35), (0.75, 0.35), (1., 0.)],
                    false,
                ),
                _ => 0.,
            };
            let glance = if status == Status::Running {
                let travel = ease(((scanning_phase - 1960.) / 360.).clamp(0., 1.));
                if leftward {
                    -4. * travel
                } else {
                    -4. * (1. - travel)
                }
            } else if shake {
                keyed(
                    ((failed_time - 2280.) % 90.) / 90.,
                    &[(0., 0.), (0.25, -0.9), (0.75, 0.9), (1., 0.)],
                    false,
                )
            } else {
                0.
            };
            let blink_phase = if status == Status::Completed {
                (time - 2230.) / 300.
            } else {
                (scanning_phase - 1500.) / 230.
            };
            let mut blink = if matches!(status, Status::Running | Status::Completed)
                && (0.0..if status == Status::Completed {
                    1.0
                } else {
                    2.0
                })
                    .contains(&blink_phase)
            {
                let beat = blink_phase.fract();
                if beat < 0.35 {
                    ease(beat / 0.35)
                } else if beat < 0.7 {
                    1. - ease((beat - 0.35) / 0.35)
                } else {
                    0.
                }
            } else {
                0.
            };
            let focused = status == Status::Failed
                && time >= 4000.
                && (2180.0..3380.0).contains(&failed_time);
            if status == Status::Failed && time >= 4000. {
                for start in [2100., 3300.] {
                    let morph = (failed_time - start) / 180.;
                    if (0.0..1.0).contains(&morph) {
                        blink = keyed(morph, &[(0., 0.), (0.4, 1.), (0.5, 1.), (1., 0.)], true);
                    }
                }
            }
            let pill = status == Status::Completed && (1980.0..2530.0).contains(&time);
            let p = |x: f32, y: f32| {
                let y = if status == Status::Failed {
                    9. + (y - 9.) * (1. - blink * 2. / 3.)
                } else {
                    y
                };
                bounds.origin + point(px(x + glance), px(y + terminal_y))
            };
            let mut eyes = gpui::PathBuilder::fill();
            if matches!(status, Status::Waiting | Status::Interrupted) {
                for (index, x) in [5., 10., 15.].into_iter().enumerate() {
                    let wave = (progress * 2400. - 1500. - index as f32 * 200.) / 500.;
                    let offset = if status == Status::Waiting && (0.0..1.0).contains(&wave) {
                        -1.5 * if wave < 0.5 {
                            ease(wave * 2.)
                        } else {
                            1. - ease((wave - 0.5) * 2.)
                        }
                    } else {
                        0.
                    };
                    let y = 10. + offset;
                    eyes.move_to(p(x - 1.5, y));
                    eyes.arc_to(point(px(1.5), px(1.5)), px(0.), false, true, p(x + 1.5, y));
                    eyes.arc_to(point(px(1.5), px(1.5)), px(0.), false, true, p(x - 1.5, y));
                    eyes.close();
                }
            } else {
                for (index, x) in [7., 13.].into_iter().enumerate() {
                    let radius = point(px(2.), px(2.));
                    match status {
                        Status::Completed if !pill => {
                            eyes.move_to(p(x, 11.));
                            eyes.line_to(p(x, 8.));
                            eyes.arc_to(radius, px(0.), false, true, p(x + 4., 8.));
                            eyes.line_to(p(x + 4., 11.));
                        }
                        Status::Failed => {
                            let (left, right) = if (index == 0) != focused {
                                (8., 6.)
                            } else {
                                (6., 8.)
                            };
                            let dy = if focused { 0. } else { 0.3 };
                            eyes.move_to(p(x, left + dy));
                            eyes.line_to(p(x + 4., right + dy));
                            eyes.line_to(p(x + 4., 10. + dy));
                            eyes.arc_to(
                                point(px(2.), px(2. * (1. - blink * 2. / 3.))),
                                px(0.),
                                false,
                                true,
                                p(x, 10. + dy),
                            );
                        }
                        _ => {
                            let scale = 1. - blink * (2. / 3.);
                            let top = 9. - 3. * scale;
                            let radius = point(px(2.), px(2. * scale));
                            eyes.move_to(p(x, top + 2. * scale));
                            eyes.arc_to(radius, px(0.), false, true, p(x + 4., top + 2. * scale));
                            eyes.line_to(p(x + 4., top + 4. * scale));
                            eyes.arc_to(radius, px(0.), false, true, p(x, top + 4. * scale));
                        }
                    }
                    eyes.close();
                }
            }
            if let Ok(path) = eyes.build() {
                window.paint_path(path, rgba((0xffffff << 8) | ((1. - blink) * 255.) as u32));
            }
            if blink > 0. {
                let p = |x: f32, y: f32| bounds.origin + point(px(x + glance), px(y + terminal_y));
                let mut closed = gpui::PathBuilder::fill();
                for x in [7., 13.] {
                    closed.move_to(p(x + 1., 8.));
                    closed.line_to(p(x + 3., 8.));
                    closed.arc_to(point(px(1.), px(1.)), px(0.), false, true, p(x + 3., 10.));
                    closed.line_to(p(x + 1., 10.));
                    closed.arc_to(point(px(1.), px(1.)), px(0.), false, true, p(x + 1., 8.));
                    closed.close();
                }
                if let Ok(path) = closed.build() {
                    window.paint_path(path, rgba((0xffffff << 8) | (blink * 255.) as u32));
                }
            }
        },
    )
    .w(px(20.))
    .h(px(20.))
}

/// Observe the owning chat so an open detail view follows snapshots and disclosure changes.
struct WorkflowDetail {
    app: WeakEntity<PiDesktopApp>,
    workflow_id: String,
    session_id: String,
    _subscription: Subscription,
}

impl Render for WorkflowDetail {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = if Theme::global(cx).is_dark() {
            ThemeColors::dark()
        } else {
            ThemeColors::light()
        };
        self.app
            .update(cx, |app, cx| {
                let workflow = app.task_states.get(&self.session_id).and_then(|state| {
                    state
                        .workflows
                        .iter()
                        .find(|workflow| workflow.id == self.workflow_id)
                });
                match workflow {
                    Some(workflow) => {
                        app.render_workflow_card(&self.session_id, workflow, &colors, true, cx)
                    }
                    None => div()
                        .p_3()
                        .child(app.tr("此工作流已不可用", "This workflow is no longer available"))
                        .into_any_element(),
                }
            })
            .unwrap_or_else(|_| div().into_any_element())
    }
}

fn avatar_slot(index: Option<i32>, name: &str) -> usize {
    match index {
        Some(index) => index.rem_euclid(9) as usize,
        None => {
            name.chars()
                .fold(0u32, |hash, ch| (hash * 31 + ch as u32) % 360) as usize
                % 9
        }
    }
}

#[cfg(test)]
mod visual_identity_tests {
    use super::{
        avatar_slot, digest_focus_ring, digest_key_toggles, digest_pointer_toggles,
        workflow_stage_geometry, WorkflowScope,
    };

    #[test]
    fn legacy_disclosure_migrates_once_without_cross_chat_attribution() {
        let workflow: super::WorkflowSnapshot = serde_json::from_str(include_str!(
            "../../../scripts/visual_parity/fixtures/multi-agent.json"
        ))
        .unwrap();
        let state = crate::task_state::TaskState {
            workflows: vec![workflow.clone()],
            ..Default::default()
        };
        let mut saved = crate::desktop_state::SavedDesktop::default();
        saved.task_states.insert("a".into(), state.clone());
        let legacy = format!("workflow-delivery-{}", workflow.id);
        let qualified = WorkflowScope {
            session: "a",
            workflow: &workflow.id,
        }
        .key(&["delivery"]);
        saved.expanded_tools.insert(legacy.clone());
        saved.expanded_tools.insert("unrelated-tool".into());
        super::migrate_disclosures(&mut saved);
        assert!(saved.expanded_tools.contains(&qualified));
        assert!(!saved.expanded_tools.contains(&legacy));
        saved.expanded_tools.remove(&qualified);
        super::migrate_disclosures(&mut saved);
        assert!(!saved.expanded_tools.contains(&qualified));
        saved.task_states.insert("b".into(), state);
        saved.expanded_tools.insert(legacy.clone());
        super::migrate_disclosures(&mut saved);
        assert_eq!(
            saved.expanded_tools,
            std::collections::HashSet::from(["unrelated-tool".into()])
        );
    }

    #[test]
    fn disclosure_keys_isolate_chats_and_ambiguous_ids() {
        let a = WorkflowScope {
            session: "chat-a",
            workflow: "w",
        };
        let b = WorkflowScope {
            session: "chat-b",
            workflow: "w",
        };
        for parts in [
            vec!["collapse"],
            vec!["delivery"],
            vec!["changes"],
            vec!["node", "stage", "agent"],
        ] {
            assert_ne!(a.key(&parts), b.key(&parts));
        }
        assert_ne!(a.key(&["node", "a-b", "c"]), a.key(&["node", "a", "b-c"]));
        assert_ne!(a.key(&["node", "a", "b"]), a.key(&["node", "ab"]));
    }

    #[test]
    fn executor_identity_is_independent_of_label_and_wraps_palette() {
        assert_eq!(avatar_slot(Some(10), "worker"), 1);
        assert_eq!(avatar_slot(Some(10), "renamed"), 1);
        assert_eq!(avatar_slot(Some(-1), "worker"), 8);
    }

    #[test]
    fn fallback_hashes_unicode_codepoints() {
        assert_eq!(avatar_slot(None, "A"), 2);
        assert_eq!(avatar_slot(None, "😀"), 1);
        assert_eq!(avatar_slot(None, ""), 0);
    }

    #[test]
    fn digest_click_toggles_only_blank_rail_cards() {
        assert!(digest_pointer_toggles(true, false));
        assert!(!digest_pointer_toggles(false, false));
        // Keyboard-generated clicks belong to the focused control, not the card.
        assert!(!digest_pointer_toggles(true, true));
    }

    #[test]
    fn digest_focus_ring_is_a_neutral_two_pixel_band() {
        let ring: super::Hsla = super::rgba(0x80808026).into();
        let shadows = digest_focus_ring(ring);
        assert_eq!(shadows.len(), 1);
        assert_eq!(shadows[0].blur_radius, super::px(0.));
        assert_eq!(shadows[0].spread_radius, super::px(2.));
        assert!((shadows[0].color.a - ring.a * 0.4).abs() < f32::EPSILON);
        assert!(!shadows[0].inset);
    }

    #[test]
    fn digest_keyboard_activation_requires_card_focus() {
        assert!(digest_key_toggles(true, true, "enter", false));
        assert!(digest_key_toggles(true, true, "space", false));
        assert!(!digest_key_toggles(true, false, "enter", false));
        assert!(!digest_key_toggles(true, false, "space", false));
        assert!(!digest_key_toggles(false, true, "enter", false));
        assert!(!digest_key_toggles(true, true, "enter", true));
        assert!(!digest_key_toggles(true, true, "space", true));
        assert!(!digest_key_toggles(true, true, "tab", false));
    }

    #[test]
    fn workflow_stage_geometry_matches_zcode_reference() {
        assert_eq!(
            workflow_stage_geometry(),
            super::WorkflowStageGeometry {
                heading_height: 24.,
                stage_to_pill_gap: 8.,
                agent_pill_height: 32.,
                agent_pill_padding_left: 8.,
                agent_pill_padding_right: 10.,
            }
        );
    }
}

fn node_status_mark(status: Status, reduced_motion: bool, c: &ThemeColors) -> AnyElement {
    let icon = Icon::new(match status {
        Status::Running => Glyph::LoaderCircle,
        Status::Completed => Glyph::CircleCheck,
        Status::Failed => Glyph::CircleX,
        Status::Interrupted => Glyph::Square,
        Status::Waiting => Glyph::Circle,
    })
    .size(px(14.))
    .flex_shrink_0()
    .text_color(if status == Status::Failed {
        rgb(c.workflow_error)
    } else {
        rgba(c.foreground_subtle)
    });
    if status == Status::Running && !reduced_motion {
        icon.with_animation(
            "workflow-node-running",
            Animation::new(std::time::Duration::from_secs(1)).repeat(),
            |icon, progress| icon.rotate(gpui::radians(progress * std::f32::consts::TAU)),
        )
        .into_any_element()
    } else {
        icon.into_any_element()
    }
}
