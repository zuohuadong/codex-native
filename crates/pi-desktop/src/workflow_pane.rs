use super::*;
use crate::workbench::icon_button;
use crate::workflow_view::{
    digest_focus_ring, role_badge, stage_marker, stage_roster, status_label, workflow_status_label,
    NodeSurface, WorkflowScope,
};
use gpui_kit::assets::IconName as Glyph;
use gpui_kit::prelude::FluentBuilder;
use pi_agent_core::workflow::{Stage, Status};

const ATTENTION_ORDER: [Status; 5] = [
    Status::Failed,
    Status::Running,
    Status::Waiting,
    Status::Interrupted,
    Status::Completed,
];

#[derive(Default)]
pub(super) struct PhaseState {
    pub open: HashSet<String>,
    pub listed: HashSet<String>,
    running: HashSet<String>,
}

impl PhaseState {
    fn sync(&mut self, stages: &[Stage]) {
        let running: HashSet<_> = stages
            .iter()
            .filter(|stage| stage.status == Status::Running)
            .map(|stage| stage.id.clone())
            .collect();
        if self.running != running {
            self.open.extend(running.iter().cloned());
            self.running = running;
        }
        self.open
            .retain(|id| stages.iter().any(|stage| &stage.id == id));
        self.listed
            .retain(|id| stages.iter().any(|stage| &stage.id == id));
    }

    fn land(&mut self, stages: &[Stage], stage_id: &str) -> Option<usize> {
        let index = stages.iter().position(|stage| stage.id == stage_id)?;
        self.open.insert(stage_id.to_owned());
        self.listed.insert(stage_id.to_owned());
        Some(index)
    }

    pub fn toggle_listed(&mut self, stage_id: &str) {
        toggle(&mut self.listed, stage_id);
    }
}

fn toggle(set: &mut HashSet<String>, id: &str) {
    if !set.remove(id) {
        set.insert(id.to_owned());
    }
}

pub(super) struct WorkflowPane {
    pub session_id: String,
    pub workflow_id: String,
    pub phases: PhaseState,
    scroll: ScrollHandle,
    focus: FocusHandle,
    return_focus: Option<FocusHandle>,
}

fn groups(stage: &Stage, indexes: &[usize]) -> Vec<(Status, Vec<usize>)> {
    ATTENTION_ORDER
        .into_iter()
        .filter_map(|status| {
            let nodes: Vec<_> = indexes
                .iter()
                .copied()
                .filter(|&index| stage.nodes[index].status == status)
                .collect();
            (!nodes.is_empty()).then_some((status, nodes))
        })
        .collect()
}

fn status_color(status: Status, c: &ThemeColors) -> Rgba {
    match status {
        Status::Completed => rgb(c.accent_green),
        Status::Running => rgb(c.workflow_active),
        Status::Failed => rgb(c.workflow_error),
        Status::Waiting | Status::Interrupted => rgba(c.foreground_subtlest),
    }
}

fn tally_item(status: Status, count: usize, c: &ThemeColors, language: Language) -> AnyElement {
    let label = format!(
        "{} {}",
        count,
        status_label(status, language == Language::Chinese)
    );
    div()
        .id(format!("tally-{status:?}"))
        .h_flex()
        .items_center()
        .flex_shrink_0()
        .gap(px(3.))
        .text_size(px(10.))
        .text_color(status_color(status, c))
        .tooltip(move |window, cx| tooltip::Tooltip::new(label.clone()).build(window, cx))
        .child(
            Icon::new(match status {
                Status::Completed => Glyph::CircleCheck,
                Status::Running => Glyph::LoaderCircle,
                Status::Failed => Glyph::CircleX,
                Status::Waiting => Glyph::Circle,
                Status::Interrupted => Glyph::Square,
            })
            .size(px(10.)),
        )
        .child(div().font_medium().child(count.to_string()))
        .into_any_element()
}

pub(super) fn roster_tally(
    stage: &Stage,
    indexes: &[usize],
    c: &ThemeColors,
    language: Language,
) -> AnyElement {
    div()
        .h_flex()
        .items_center()
        .gap(px(10.))
        .children(
            [
                Status::Completed,
                Status::Running,
                Status::Failed,
                Status::Waiting,
                Status::Interrupted,
            ]
            .into_iter()
            .filter_map(|status| {
                let count = indexes
                    .iter()
                    .filter(|&&i| stage.nodes[i].status == status)
                    .count();
                (count > 0).then(|| tally_item(status, count, c, language))
            }),
        )
        .into_any_element()
}

fn roster_meter(stage: &Stage, c: &ThemeColors) -> AnyElement {
    div()
        .w(px(44.))
        .h(px(3.))
        .rounded(px(2.))
        .overflow_hidden()
        .h_flex()
        .gap(px(1.))
        .children(
            [
                Status::Completed,
                Status::Failed,
                Status::Running,
                Status::Waiting,
                Status::Interrupted,
            ]
            .into_iter()
            .filter_map(|status| {
                let count = stage
                    .nodes
                    .iter()
                    .filter(|node| node.status == status)
                    .count();
                (count > 0).then(|| {
                    div()
                        .min_w(px(2.))
                        .flex_basis(px(0.))
                        .flex_grow(count as f32)
                        .h_full()
                        .bg(if status == Status::Waiting {
                            rgba(c.card_border)
                        } else {
                            status_color(status, c)
                        })
                })
            }),
        )
        .into_any_element()
}

impl PiDesktopApp {
    pub(super) fn workflow_pane_active(&self) -> bool {
        !self.show_settings
            && self.selected_nav == "chat"
            && self
                .workflow_pane
                .as_ref()
                .is_some_and(|pane| pane.session_id == self.session.id)
    }

    pub(super) fn sync_workflow_pane(&mut self) {
        if let Some(pane) = self.workflow_pane.as_mut() {
            if let Some(workflow) = self.task_states.get(&pane.session_id).and_then(|state| {
                state
                    .workflows
                    .iter()
                    .find(|workflow| workflow.id == pane.workflow_id)
            }) {
                pane.phases.sync(&workflow.stages);
            }
        }
    }

    pub(super) fn open_workflow_pane(
        &mut self,
        session_id: &str,
        workflow_id: &str,
        stage_id: Option<&str>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(workflow) = self.task_states.get(session_id).and_then(|state| {
            state
                .workflows
                .iter()
                .find(|workflow| workflow.id == workflow_id)
        }) else {
            return;
        };
        if session_id != self.session.id {
            return;
        }
        let same_run = self
            .workflow_pane
            .as_ref()
            .is_some_and(|pane| pane.session_id == session_id && pane.workflow_id == workflow_id);
        if !same_run {
            let return_focus = self
                .workflow_pane
                .as_ref()
                .and_then(|pane| pane.return_focus.clone())
                .or_else(|| window.focused(cx));
            self.workflow_pane = Some(WorkflowPane {
                session_id: session_id.to_owned(),
                workflow_id: workflow_id.to_owned(),
                phases: PhaseState::default(),
                scroll: ScrollHandle::new(),
                focus: cx.focus_handle(),
                return_focus,
            });
        }
        let pane = self.workflow_pane.as_mut().expect("pane was initialized");
        pane.phases.sync(&workflow.stages);
        if let Some(index) = stage_id.and_then(|id| pane.phases.land(&workflow.stages, id)) {
            // GPUI resolves the child after layout, including a newly opened body.
            pane.scroll.scroll_to_top_of_item(index);
        }
        pane.focus.focus(window, cx);
        self.review_visible = false;
        self.selected_nav = "chat";
        self.show_settings = false;
        cx.notify();
    }

    pub(super) fn close_workflow_pane(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(pane) = self.workflow_pane.take() {
            pane.return_focus
                .unwrap_or_else(|| self.root_focus.clone())
                .focus(window, cx);
        }
        cx.notify();
    }

    pub(super) fn workflow_pane_view(&self, c: &ThemeColors, cx: &mut Context<Self>) -> AnyElement {
        let Some(pane) = self.workflow_pane.as_ref() else {
            return div().into_any_element();
        };
        let workflow = self.task_states.get(&pane.session_id).and_then(|state| {
            state
                .workflows
                .iter()
                .find(|workflow| workflow.id == pane.workflow_id)
        });
        let focus = pane.focus.clone();
        let keyboard_scroll = pane.scroll.clone();
        let mut view = div()
            .id("workflow-run-pane")
            .track_focus(&pane.focus)
            .on_key_down(move |event, window, cx| {
                let stroke = &event.keystroke;
                if !focus.is_focused(window)
                    || stroke.modifiers.control
                    || stroke.modifiers.platform
                    || stroke.modifiers.alt
                {
                    return;
                }
                if let Some((top, _)) = crate::transcript_scroll::keyboard_target(
                    &stroke.key,
                    stroke.modifiers.shift,
                    -f32::from(keyboard_scroll.offset().y),
                    f32::from(keyboard_scroll.max_offset().y),
                    f32::from(keyboard_scroll.bounds().size.height),
                ) {
                    keyboard_scroll.set_offset(point(px(0.), px(-top)));
                    window.prevent_default();
                    window.refresh();
                    cx.stop_propagation();
                }
            })
            .v_flex()
            .flex_1()
            .size_full()
            .min_w_0()
            .min_h_0()
            .bg(rgb(c.root_bg))
            .child(
                div()
                    .h_flex()
                    .items_center()
                    .flex_shrink_0()
                    .h(px(48.))
                    .px_3()
                    .gap_2()
                    .border_b_1()
                    .border_color(rgba(c.card_border))
                    .child(Icon::new(Glyph::Workflow).size(px(16.)))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .font_medium()
                            .child(self.tr("工作流详情", "Workflow details")),
                    )
                    .child(
                        icon_button("close-workflow-pane", self.tr("关闭", "Close"), Glyph::X)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.close_workflow_pane(window, cx);
                            })),
                    ),
            );
        let Some(workflow) = workflow else {
            return view
                .child(
                    div()
                        .p_4()
                        .child(self.tr("此工作流已不可用", "This workflow is no longer available")),
                )
                .into_any_element();
        };
        let scope = WorkflowScope {
            session: &pane.session_id,
            workflow: &pane.workflow_id,
        };
        let stop_state = self.workflow_controls.stopping(&workflow.id);
        let workflow_id = workflow.id.clone();
        view = view.child(
            div()
                .flex_shrink_0()
                .v_flex()
                .px_4()
                .py_3()
                .gap(px(6.))
                .border_b_1()
                .border_color(rgba(c.card_border))
                .child(
                    div()
                        .h_flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .truncate()
                                .font_family(cx.theme().mono_font_family.clone())
                                .font_medium()
                                .child(workflow.title.clone()),
                        )
                        .when(workflow.status == Status::Running, |row| {
                            row.child(
                                icon_button(
                                    "workflow-pane-stop",
                                    if stop_state == Some(true) {
                                        self.tr("正在停止…", "Stopping…")
                                    } else {
                                        self.tr("停止工作流", "Stop workflow")
                                    },
                                    Glyph::Square,
                                )
                                .disabled(stop_state != Some(false))
                                .on_click(cx.listener(
                                    move |this, _, _, cx| {
                                        this.stop_workflow(workflow_id.clone(), cx);
                                    },
                                )),
                            )
                        }),
                )
                .child(
                    div()
                        .h_flex()
                        .items_center()
                        .gap_2()
                        .child(stage_marker(
                            workflow.status,
                            c,
                            self.preferences.reduced_motion,
                        ))
                        .child(workflow_status_label(
                            workflow.status,
                            self.language == Language::Chinese,
                        )),
                )
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .text_size(px(12.))
                        .text_color(rgba(c.foreground_subtlest))
                        .child(format!(
                            "{} · {} {}",
                            workflow.model,
                            workflow.stages.len(),
                            self.tr("个阶段", "stages")
                        )),
                ),
        );
        view.child(
            div()
                .id(scope.key(&["pane-phases"]))
                .flex_1()
                .min_h_0()
                .w_full()
                .overflow_y_scroll()
                .track_scroll(&pane.scroll)
                .v_flex()
                .pt(px(10.))
                .pb_3()
                .children(workflow.stages.iter().enumerate().map(|(index, stage)| {
                    self.workflow_pane_stage(scope, stage, index + 1 < workflow.stages.len(), c, cx)
                }))
                .when(!workflow.delivery.trim().is_empty(), |list| {
                    list.child(
                        div()
                            .px_4()
                            .py_3()
                            .v_flex()
                            .gap_2()
                            .child(
                                div()
                                    .font_medium()
                                    .child(self.tr("交付看板", "Deliverables")),
                            )
                            .child(gpui_kit::base::SelectableText::new(
                                scope.key(&["pane-delivery"]),
                                workflow.delivery.clone(),
                            )),
                    )
                }),
        )
        .into_any_element()
    }

    fn workflow_pane_stage(
        &self,
        scope: WorkflowScope<'_>,
        stage: &Stage,
        has_next: bool,
        c: &ThemeColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let pane = self
            .workflow_pane
            .as_ref()
            .expect("pane stage has an owner");
        let open = pane.phases.open.contains(&stage.id);
        let listed = pane.phases.listed.contains(&stage.id);
        let roster = stage_roster(&stage.nodes);
        let stage_id = stage.id.clone();
        let session_id = scope.session.to_owned();
        let workflow_id = scope.workflow.to_owned();
        let ring = cx.theme().ring;
        div()
            .id(scope.key(&["pane-stage", &stage.id]))
            .relative()
            .flex_shrink_0()
            .w_full()
            .min_w_0()
            .when(has_next, |section| {
                section.child(
                    div()
                        .absolute()
                        .left(px(19.))
                        .top(px(18.))
                        .bottom(px(0.))
                        .w(px(1.))
                        .bg(rgb(c.workflow_connector)),
                )
            })
            .child(
                gpui_kit::base::Button::new(scope.key(&["pane-phase", &stage.id]))
                    .relative()
                    .h(px(36.))
                    .w_full()
                    .pl(px(39.))
                    .pr_3()
                    .justify_start()
                    .aria_expanded(open)
                    .accessibility_label(format!(
                        "{} · {} · {}",
                        stage.label,
                        status_label(stage.status, self.language == Language::Chinese),
                        if open {
                            self.tr("收起阶段", "Collapse stage")
                        } else {
                            self.tr("展开阶段", "Expand stage")
                        }
                    ))
                    .hover(|style| style.bg(rgba(c.workflow_node_bg)))
                    .focus_visible(move |style| style.shadow(digest_focus_ring(ring)))
                    .child(
                        div()
                            .absolute()
                            .left(px(12.))
                            .top(px(10.))
                            .child(stage_marker(
                                stage.status,
                                c,
                                self.preferences.reduced_motion,
                            )),
                    )
                    .child(
                        div()
                            .h_flex()
                            .items_center()
                            .w_full()
                            .min_w_0()
                            .gap_2()
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    .text_size(px(14.))
                                    .font_medium()
                                    .when(stage.status == Status::Waiting, |label| {
                                        label.font_normal().text_color(rgba(c.foreground_subtle))
                                    })
                                    .child(stage.label.clone()),
                            )
                            .when(!open, |row| {
                                if roster.is_some() {
                                    row.child(roster_meter(stage, c))
                                } else {
                                    row.child(
                                        div()
                                            .h_flex()
                                            .items_center()
                                            .gap(px(2.))
                                            .children(stage.nodes.iter().take(3).map(|node| {
                                                role_badge(node, c, self.preferences.reduced_motion)
                                            }))
                                            .when(stage.nodes.len() > 3, |cluster| {
                                                cluster.child(
                                                    div().text_size(px(10.)).child(format!(
                                                        "+{}",
                                                        stage.nodes.len() - 3
                                                    )),
                                                )
                                            }),
                                    )
                                }
                            })
                            .when_some(stage.visible_progress(), |row, progress| {
                                row.child(
                                    div()
                                        .text_size(px(10.))
                                        .text_color(rgba(c.foreground_subtlest))
                                        .font_family(cx.theme().mono_font_family.clone())
                                        .child(format!(
                                            "{}/{}",
                                            progress.settled, progress.observed
                                        )),
                                )
                            })
                            .child(
                                Icon::new(if open {
                                    Glyph::ChevronDown
                                } else {
                                    Glyph::ChevronRight
                                })
                                .size(px(14.))
                                .text_color(rgba(c.foreground_subtlest)),
                            ),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if let Some(pane) = this.workflow_pane.as_mut().filter(|pane| {
                            pane.session_id == session_id && pane.workflow_id == workflow_id
                        }) {
                            toggle(&mut pane.phases.open, &stage_id);
                        }
                        cx.notify();
                    })),
            )
            .when(open, |section| {
                let indexes = roster
                    .as_ref()
                    .map(|roster| roster.pinned.clone())
                    .unwrap_or_else(|| (0..stage.nodes.len()).collect());
                let mut body = div()
                    .pl(px(39.))
                    .pr_3()
                    .pt(px(2.))
                    .pb_3()
                    .v_flex()
                    .gap(px(6.))
                    .children(indexes.into_iter().map(|index| {
                        self.workflow_node(
                            scope,
                            stage,
                            &stage.nodes[index],
                            NodeSurface::Pane,
                            c,
                            cx,
                        )
                    }));
                if let Some(roster) = roster {
                    body = body.child(self.workflow_more_row(
                        scope,
                        stage,
                        &roster,
                        Some(listed),
                        c,
                        cx,
                    ));
                    if listed {
                        for (status, indexes) in groups(stage, &roster.rest) {
                            body = body.child(
                                div()
                                    .h_flex()
                                    .items_center()
                                    .h(px(22.))
                                    .gap(px(5.))
                                    .pl(px(4.))
                                    .font_family(cx.theme().mono_font_family.clone())
                                    .child(tally_item(status, indexes.len(), c, self.language))
                                    .child(
                                        div()
                                            .text_size(px(10.))
                                            .text_color(status_color(status, c))
                                            .child(status_label(
                                                status,
                                                self.language == Language::Chinese,
                                            )),
                                    )
                                    .child(div().flex_1().h(px(1.)).bg(rgba(c.card_border))),
                            );
                            body = body.child(div().grid().grid_cols(2).gap_x(px(8.)).children(
                                indexes.into_iter().map(|index| {
                                    self.workflow_node(
                                        scope,
                                        stage,
                                        &stage.nodes[index],
                                        NodeSurface::Roll,
                                        c,
                                        cx,
                                    )
                                }),
                            ));
                        }
                    }
                }
                section.child(body)
            })
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::{groups, stage_roster, PhaseState, Stage, Status};
    use pi_agent_core::workflow::{Node, NodeKind};

    fn stage(id: &str, status: Status) -> Stage {
        Stage {
            id: id.into(),
            label: id.into(),
            status,
            nodes: vec![],
            progress: None,
        }
    }

    #[test]
    fn every_running_stage_opens_but_repeated_snapshots_preserve_manual_collapse() {
        let mut phases = PhaseState::default();
        let mut stages = vec![stage("a", Status::Running), stage("b", Status::Running)];
        phases.sync(&stages);
        assert_eq!(phases.open.len(), 2);
        phases.open.remove("a");
        phases.sync(&stages);
        assert!(!phases.open.contains("a"));
        stages[1].status = Status::Completed;
        phases.sync(&stages);
        assert!(phases.open.contains("a"));
        assert!(phases.open.contains("b"));
    }

    #[test]
    fn landing_opens_the_stage_and_roster_each_time_and_rejects_missing_targets() {
        let mut phases = PhaseState::default();
        let stages = vec![stage("a", Status::Completed), stage("b", Status::Failed)];
        assert_eq!(phases.land(&stages, "b"), Some(1));
        assert!(phases.open.contains("b"));
        assert!(phases.listed.contains("b"));
        phases.toggle_listed("b");
        assert!(!phases.listed.contains("b"));
        assert_eq!(phases.land(&stages, "b"), Some(1));
        assert!(phases.listed.contains("b"));
        assert_eq!(phases.land(&stages, "missing"), None);
        assert!(!phases.open.contains("missing"));
        phases.sync(&stages[..1]);
        assert!(!phases.open.contains("b"));
        assert!(!phases.listed.contains("b"));
    }

    #[test]
    fn grouped_rest_preserves_each_node_once_and_does_not_disguise_interruptions() {
        let mut stage = stage("roster", Status::Failed);
        for (i, status) in [
            Status::Completed,
            Status::Failed,
            Status::Running,
            Status::Waiting,
            Status::Interrupted,
            Status::Failed,
            Status::Completed,
            Status::Completed,
            Status::Completed,
        ]
        .into_iter()
        .enumerate()
        {
            stage.nodes.push(Node {
                id: i.to_string(),
                label: i.to_string(),
                kind: NodeKind::Agent,
                avatar_index: None,
                status,
                detail: String::new(),
            });
        }
        let roster = stage_roster(&stage.nodes).unwrap();
        let groups = groups(&stage, &roster.rest);
        assert_eq!(
            groups.iter().map(|(status, _)| *status).collect::<Vec<_>>(),
            vec![Status::Interrupted, Status::Completed]
        );
        let mut all = roster.pinned.clone();
        all.extend(
            groups
                .iter()
                .flat_map(|(_, indexes)| indexes.iter().copied()),
        );
        all.sort_unstable();
        assert_eq!(all, (0..stage.nodes.len()).collect::<Vec<_>>());
        assert_eq!(groups.last().unwrap().1, vec![6, 7, 8]);
    }

    #[test]
    fn groups_follow_attention_order_and_keep_source_order_within_each_status() {
        let mut stage = stage("groups", Status::Running);
        for (index, status) in [
            Status::Completed,
            Status::Failed,
            Status::Running,
            Status::Waiting,
            Status::Interrupted,
            Status::Failed,
            Status::Completed,
        ]
        .into_iter()
        .enumerate()
        {
            stage.nodes.push(Node {
                id: index.to_string(),
                label: index.to_string(),
                kind: NodeKind::Agent,
                avatar_index: None,
                status,
                detail: String::new(),
            });
        }
        assert_eq!(
            groups(&stage, &(0..7).collect::<Vec<_>>()),
            vec![
                (Status::Failed, vec![1, 5]),
                (Status::Running, vec![2]),
                (Status::Waiting, vec![3]),
                (Status::Interrupted, vec![4]),
                (Status::Completed, vec![0, 6]),
            ]
        );
        assert!(groups(&stage, &[]).is_empty());
    }
}
