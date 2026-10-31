use gpui_kit::base::{animate_keyframes, Easing, Keyframe, Keyframes, SignedDuration, Timing};
use gpui_kit::{px, App, Div, IntoElement, RenderOnce, SharedString, Styled, Window};
use std::time::Duration;

const ENTER_TRAVEL: Duration = Duration::from_millis(200);

#[derive(Clone, Copy)]
enum Kind {
    Arrive,
    Unfold,
}

pub(crate) fn pill_delay(ordinal: usize) -> Duration {
    Duration::from_millis((ordinal as u64).saturating_mul(30))
}

pub(crate) fn roll_delay(ordinal: usize) -> Duration {
    Duration::from_millis(ordinal.min(50) as u64 * 8)
}

fn timing(delay: Duration, reduced_motion: bool) -> Timing {
    if reduced_motion {
        return Timing::new(Duration::ZERO);
    }
    Timing::new(ENTER_TRAVEL)
        .delay(SignedDuration::positive(delay))
        .ease(Easing::CubicBezier {
            x1: 0.22,
            y1: 0.61,
            x2: 0.36,
            y2: 1.,
        })
}

fn presented(body: Div, kind: Kind, progress: f32) -> Div {
    let progress = progress.clamp(0., 1.);
    let from = match kind {
        Kind::Arrive => 4.,
        Kind::Unfold => -3.,
    };
    // Relative positioning preserves the final layout box and sibling spacing.
    body.relative()
        .top(px(from * (1. - progress)))
        .opacity(progress)
}

#[derive(IntoElement)]
pub(crate) struct WorkflowEntrance {
    id: SharedString,
    body: Div,
    kind: Kind,
    delay: Duration,
    reduced_motion: bool,
}

impl WorkflowEntrance {
    pub(crate) fn arrive(
        id: impl Into<SharedString>,
        body: Div,
        delay: Duration,
        reduced_motion: bool,
    ) -> Self {
        Self {
            id: id.into(),
            body,
            kind: Kind::Arrive,
            delay,
            reduced_motion,
        }
    }

    pub(crate) fn unfold(id: impl Into<SharedString>, body: Div, reduced_motion: bool) -> Self {
        Self {
            id: id.into(),
            body,
            kind: Kind::Unfold,
            delay: Duration::ZERO,
            reduced_motion,
        }
    }
}

impl RenderOnce for WorkflowEntrance {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let frames = Keyframes::try_new([Keyframe::new(0., 0.), Keyframe::new(1., 1.)])
            .expect("entrance track has fixed ordered endpoints");
        let progress = animate_keyframes(
            (self.id, "entrance"),
            &frames,
            timing(self.delay, self.reduced_motion),
            window,
            cx,
        )
        .value;
        presented(self.body, self.kind, progress)
    }
}

#[cfg(test)]
mod tests {
    use super::{pill_delay, presented, roll_delay, timing, Kind, ENTER_TRAVEL};
    use gpui_kit::base::MotionPhase;
    use gpui_kit::{div, px, Position, Styled};
    use std::time::Duration;

    #[test]
    fn stagger_matches_card_and_roll_reference_without_overflow() {
        assert_eq!(pill_delay(0), Duration::ZERO);
        assert_eq!(pill_delay(5), Duration::from_millis(150));
        assert_eq!(pill_delay(20), Duration::from_millis(600));
        assert_eq!(roll_delay(0), Duration::ZERO);
        assert_eq!(roll_delay(3), Duration::from_millis(24));
        assert_eq!(roll_delay(50), Duration::from_millis(400));
        assert_eq!(roll_delay(usize::MAX), Duration::from_millis(400));
    }

    #[test]
    fn delayed_entries_hold_the_start_frame_before_the_full_200ms_travel() {
        assert_eq!(ENTER_TRAVEL, Duration::from_millis(200));
        let timeline = timing(Duration::from_millis(24), false);
        let before = timeline.sample(Duration::from_millis(23));
        assert_eq!(before.phase, MotionPhase::Before);
        assert_eq!(before.directed_progress, 0.);
        let start = timeline.sample(Duration::from_millis(24));
        assert_eq!(start.phase, MotionPhase::Active);
        assert_eq!(start.directed_progress, 0.);
        let middle = timeline.sample(Duration::from_millis(124));
        assert!(middle.directed_progress > 0.5 && middle.directed_progress < 1.);
        let end = timeline.sample(Duration::from_millis(224));
        assert_eq!(end.phase, MotionPhase::After);
        assert_eq!(end.directed_progress, 1.);
    }

    #[test]
    fn reduced_motion_removes_both_the_delay_and_travel() {
        let sample = timing(Duration::from_secs(10), true).sample(Duration::ZERO);
        assert_eq!(sample.phase, MotionPhase::After);
        assert_eq!(sample.directed_progress, 1.);
        assert!(sample.finished);
    }

    #[test]
    fn entrance_offsets_do_not_change_the_final_layout_dimensions() {
        for (kind, start) in [(Kind::Arrive, 4.), (Kind::Unfold, -3.)] {
            for progress in [0., 0.5, 1.] {
                let mut body = presented(div().w(px(168.)).h(px(32.)), kind, progress);
                let style = body.style();
                assert_eq!(style.position, Some(Position::Relative));
                assert_eq!(style.inset.top, Some(px(start * (1. - progress)).into()));
                assert_eq!(style.opacity, Some(progress));
                assert_eq!(style.size.width, Some(px(168.).into()));
                assert_eq!(style.size.height, Some(px(32.).into()));
            }
        }
    }
}
