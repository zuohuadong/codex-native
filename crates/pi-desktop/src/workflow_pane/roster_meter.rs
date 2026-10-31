use super::{status_color, status_label, Language, Stage, Status, ThemeColors};
use gpui_kit::base::{transition, Transition};
use gpui_kit::component::StyledExt;
use gpui_kit::{
    div, px, rgba, App, Div, InteractiveElement, IntoElement, ParentElement, RenderOnce, Role,
    SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::time::Duration;

const SETTLEMENT_TRAVEL: Duration = Duration::from_millis(320);
const ORDER: [Status; 5] = [
    Status::Completed,
    Status::Failed,
    Status::Running,
    Status::Waiting,
    Status::Interrupted,
];

#[derive(IntoElement)]
pub(super) struct RosterMeter {
    pub id: SharedString,
    pub counts: [usize; 5],
    pub colors: ThemeColors,
    pub language: Language,
    pub reduced_motion: bool,
}

pub(super) fn counts(stage: &Stage) -> [usize; 5] {
    ORDER.map(|status| {
        stage
            .nodes
            .iter()
            .filter(|node| node.status == status)
            .count()
    })
}

fn frame() -> Div {
    div()
        .w(px(44.))
        .h(px(3.))
        .flex_shrink_0()
        .rounded(px(2.))
        .overflow_hidden()
        .h_flex()
        .gap(px(1.))
}

fn segment(status: Status, weight: f32, colors: &ThemeColors) -> Div {
    div()
        .min_w(px(2.))
        .flex_basis(px(0.))
        .flex_grow(weight)
        .h_full()
        .bg(if status == Status::Waiting {
            rgba(colors.card_border)
        } else {
            status_color(status, colors)
        })
}

impl RenderOnce for RosterMeter {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let label = ORDER
            .into_iter()
            .zip(self.counts)
            .filter(|(_, count)| *count > 0)
            .map(|(status, count)| {
                format!(
                    "{} {}",
                    count,
                    status_label(status, self.language == Language::Chinese)
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        let policy = Transition::new(if self.reduced_motion {
            Duration::ZERO
        } else {
            SETTLEMENT_TRAVEL
        })
        .ease(gpui_kit::base::animation::cubic_bezier(
            0.22, 0.61, 0.36, 1.,
        ));
        frame()
            .id(self.id.clone())
            .role(Role::Image)
            .aria_label(label)
            .children(
                ORDER
                    .into_iter()
                    .zip(self.counts)
                    .filter_map(|(status, count)| {
                        if count == 0 {
                            return None;
                        }
                        let weight = transition(
                            (self.id.clone(), format!("count-{status:?}")),
                            count as f32,
                            policy.clone(),
                            window,
                            cx,
                        );
                        Some(segment(status, weight, &self.colors))
                    }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::{counts, frame, segment, ORDER, SETTLEMENT_TRAVEL};
    use crate::theme::ThemeColors;
    use gpui_kit::{px, rgba, Styled};
    use pi_agent_core::workflow::{Node, NodeKind, Stage, Status};
    use std::time::Duration;

    #[test]
    fn meter_uses_observed_node_counts_and_keeps_interrupted_distinct() {
        let mut stage = Stage {
            id: "phase".into(),
            label: "Phase".into(),
            status: Status::Running,
            progress: None,
            nodes: [
                Status::Waiting,
                Status::Completed,
                Status::Interrupted,
                Status::Running,
                Status::Failed,
                Status::Completed,
                Status::Waiting,
            ]
            .into_iter()
            .enumerate()
            .map(|(index, status)| Node {
                id: index.to_string(),
                label: index.to_string(),
                kind: NodeKind::Agent,
                avatar_index: None,
                status,
                detail: String::new(),
            })
            .collect(),
        };
        assert_eq!(counts(&stage), [2, 1, 1, 2, 1]);
        stage.nodes[3].status = Status::Completed;
        assert_eq!(counts(&stage), [3, 1, 0, 2, 1]);
        stage.nodes.clear();
        assert_eq!(counts(&stage), [0; 5]);
    }

    #[test]
    fn meter_frame_keeps_the_reference_size_gap_and_duration() {
        assert_eq!(SETTLEMENT_TRAVEL, Duration::from_millis(320));
        let mut meter = frame();
        let style = meter.style();
        assert_eq!(style.size.width, Some(px(44.).into()));
        assert_eq!(style.size.height, Some(px(3.).into()));
        assert_eq!(style.flex_shrink, Some(0.));
        assert_eq!(style.gap.width, Some(px(1.).into()));
    }

    #[test]
    fn meter_segments_keep_fractional_weights_and_theme_status_colors() {
        for colors in [ThemeColors::light(), ThemeColors::dark()] {
            for status in ORDER {
                let mut part = segment(status, 2.5, &colors);
                let style = part.style();
                assert_eq!(style.flex_grow, Some(2.5));
                assert_eq!(style.flex_basis, Some(px(0.).into()));
                assert_eq!(style.min_size.width, Some(px(2.).into()));
                let expected = if status == Status::Waiting {
                    rgba(colors.card_border)
                } else {
                    super::status_color(status, &colors)
                };
                assert_eq!(style.background, Some(expected.into()));
            }
        }
    }
}
