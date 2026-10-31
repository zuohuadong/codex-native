use crate::theme::ThemeColors;
use gpui_kit::base::{transition, Transition};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::StyledExt;
use gpui_kit::{
    div, rgba, App, Div, ElementId, Entity, Focusable, HoverListenerMode, Hsla, InteractiveElement,
    IntoElement, ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Subscription,
    Window,
};
use std::time::Duration;

const BORDER_TRAVEL: Duration = Duration::from_millis(150);

fn border_target(colors: &ThemeColors, hovered: bool, focused: bool) -> Hsla {
    rgba(if hovered || focused {
        colors.dock_border_active
    } else {
        colors.settings_border
    })
    .into()
}

fn duration(reduced_motion: bool) -> Duration {
    if reduced_motion {
        Duration::ZERO
    } else {
        BORDER_TRAVEL
    }
}

fn field_shell() -> Div {
    div().v_flex().w_full().min_w_0().flex_shrink_0()
}

struct Interaction {
    hovered: bool,
    _focus_in: Subscription,
    _focus_out: Subscription,
}

#[derive(IntoElement)]
pub(super) struct SettingsField {
    state: Entity<InputState>,
    input: Input,
    colors: ThemeColors,
    reduced_motion: bool,
}

impl SettingsField {
    pub(super) fn new(
        state: &Entity<InputState>,
        input: Input,
        colors: &ThemeColors,
        reduced_motion: bool,
    ) -> Self {
        Self {
            state: state.clone(),
            input,
            colors: *colors,
            reduced_motion,
        }
    }
}

impl RenderOnce for SettingsField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = ElementId::from(("settings-field", self.state.entity_id()));
        let focus = self.state.focus_handle(cx);
        let interaction =
            window.use_keyed_state((id.clone(), "interaction"), cx, |window, cx| Interaction {
                hovered: false,
                _focus_in: window.on_focus_in(&focus, cx, |window, _| window.refresh()),
                _focus_out: window.on_focus_out(&focus, cx, |_, window, _| window.refresh()),
            });
        let border = transition(
            (id.clone(), "border"),
            border_target(
                &self.colors,
                interaction.read(cx).hovered,
                focus.is_focused(window),
            ),
            Transition::new(duration(self.reduced_motion))
                .ease(gpui_kit::base::animation::cubic_bezier(0.4, 0., 0.2, 1.)),
            window,
            cx,
        );
        // Keep the native editor's frame; the shell owns no border or tab stop.
        field_shell()
            .id(id)
            .hover_listener_mode(HoverListenerMode::InputModalityIndependent)
            .on_hover(move |hovered, window, cx| {
                if interaction.read(cx).hovered != *hovered {
                    interaction.update(cx, |state, _| state.hovered = *hovered);
                    window.refresh();
                }
            })
            .child(self.input.focus_bordered(false).border_color(border))
    }
}

#[cfg(test)]
mod tests {
    use super::{border_target, duration, field_shell, BORDER_TRAVEL};
    use crate::theme::ThemeColors;
    use gpui_kit::base::Interpolate;
    use gpui_kit::{px, relative, rgba, Hsla, Styled};
    use std::time::Duration;

    #[test]
    fn hover_and_native_editor_focus_share_the_reference_border() {
        for colors in [ThemeColors::light(), ThemeColors::dark()] {
            let idle = Hsla::from(rgba(colors.settings_border));
            let engaged = Hsla::from(rgba(colors.dock_border_active));
            assert_eq!(border_target(&colors, false, false), idle);
            assert_eq!(border_target(&colors, true, false), engaged);
            assert_eq!(border_target(&colors, false, true), engaged);
            assert_eq!(border_target(&colors, true, true), engaged);
            assert!(engaged.a > idle.a);
            assert_eq!(engaged.h, idle.h);
            assert_eq!(engaged.s, idle.s);
            assert_eq!(engaged.l, idle.l);
        }
    }

    #[test]
    fn border_motion_keeps_reference_timing_and_reduced_motion_settles_immediately() {
        assert_eq!(BORDER_TRAVEL, Duration::from_millis(150));
        assert_eq!(duration(false), BORDER_TRAVEL);
        assert_eq!(duration(true), Duration::ZERO);
        for colors in [ThemeColors::light(), ThemeColors::dark()] {
            let from = border_target(&colors, false, false);
            let to = border_target(&colors, true, false);
            for progress in [0., 0.25, 0.5, 0.75, 1.] {
                let color = from.interpolate(&to, progress);
                assert!(color.a >= from.a && color.a <= to.a);
                assert_eq!(color.h, from.h);
                assert_eq!(color.s, from.s);
                assert_eq!(color.l, from.l);
            }
        }
    }

    #[test]
    fn observation_shell_adds_no_second_frame_or_fixed_height() {
        let mut shell = field_shell();
        let style = shell.style();
        assert_eq!(style.size.width, Some(relative(1.).into()));
        assert!(style.size.height.is_none());
        assert_eq!(style.min_size.width, Some(px(0.).into()));
        assert_eq!(style.flex_shrink, Some(0.));
        assert!(style.background.is_none());
        assert!(style.border_widths.top.is_none());
        assert!(style.padding.top.is_none());
        assert!(style.padding.right.is_none());
        assert!(style.padding.bottom.is_none());
        assert!(style.padding.left.is_none());
        assert!(style.gap.height.is_none());
    }
}
