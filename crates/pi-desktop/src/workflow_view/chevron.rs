use gpui_kit::assets::IconName as Glyph;
use gpui_kit::base::{transition, Transition};
use gpui_kit::component::Icon;
use gpui_kit::{px, radians, App, IntoElement, RenderOnce, Rgba, SharedString, Styled, Window};
use std::time::Duration;

#[derive(Clone, Copy)]
pub(crate) enum ChevronKind {
    Phase,
    Roster,
}

impl ChevronKind {
    fn glyph(self) -> Glyph {
        match self {
            Self::Phase => Glyph::ChevronRight,
            Self::Roster => Glyph::ChevronDown,
        }
    }

    fn angle(self, expanded: bool) -> f32 {
        if !expanded {
            return 0.;
        }
        match self {
            Self::Phase => std::f32::consts::FRAC_PI_2,
            Self::Roster => std::f32::consts::PI,
        }
    }

    fn duration(self, reduced_motion: bool) -> Duration {
        if reduced_motion {
            return Duration::ZERO;
        }
        Duration::from_millis(match self {
            Self::Phase => 150,
            Self::Roster => 160,
        })
    }

    fn easing(self) -> [f32; 4] {
        match self {
            Self::Phase => [0.4, 0., 0.2, 1.],
            Self::Roster => [0.22, 0.61, 0.36, 1.],
        }
    }
}

#[derive(IntoElement)]
pub(crate) struct WorkflowChevron {
    pub id: SharedString,
    pub kind: ChevronKind,
    pub expanded: bool,
    pub reduced_motion: bool,
    pub color: Rgba,
}

impl RenderOnce for WorkflowChevron {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let [x1, y1, x2, y2] = self.kind.easing();
        let angle = transition(
            (self.id, "angle"),
            self.kind.angle(self.expanded),
            Transition::new(self.kind.duration(self.reduced_motion))
                .ease(gpui_kit::base::animation::cubic_bezier(x1, y1, x2, y2)),
            window,
            cx,
        );
        Icon::new(self.kind.glyph())
            .size(px(14.))
            .flex_shrink_0()
            .text_color(self.color)
            .rotate(radians(angle))
    }
}

#[cfg(test)]
mod tests {
    use super::ChevronKind;
    use gpui_kit::assets::IconName as Glyph;
    use std::time::Duration;

    #[test]
    fn phase_rotates_right_to_down_and_roster_rotates_down_to_up() {
        assert!(matches!(ChevronKind::Phase.glyph(), Glyph::ChevronRight));
        assert!(matches!(ChevronKind::Roster.glyph(), Glyph::ChevronDown));
        assert_eq!(ChevronKind::Phase.angle(false), 0.);
        assert_eq!(ChevronKind::Phase.angle(true), std::f32::consts::FRAC_PI_2);
        assert_eq!(ChevronKind::Roster.angle(false), 0.);
        assert_eq!(ChevronKind::Roster.angle(true), std::f32::consts::PI);
    }

    #[test]
    fn reference_timing_policies_settle_immediately_with_reduced_motion() {
        assert_eq!(
            ChevronKind::Phase.duration(false),
            Duration::from_millis(150)
        );
        assert_eq!(ChevronKind::Phase.easing(), [0.4, 0., 0.2, 1.]);
        assert_eq!(
            ChevronKind::Roster.duration(false),
            Duration::from_millis(160)
        );
        assert_eq!(ChevronKind::Roster.easing(), [0.22, 0.61, 0.36, 1.]);
        assert_eq!(ChevronKind::Phase.duration(true), Duration::ZERO);
        assert_eq!(ChevronKind::Roster.duration(true), Duration::ZERO);
    }
}
