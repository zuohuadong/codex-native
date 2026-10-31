use super::chevron::{ChevronKind, WorkflowChevron};
use super::entrance::WorkflowEntrance;
use crate::theme::ThemeColors;
use gpui_kit::assets::IconName as Glyph;
use gpui_kit::base::{transition, Button, Transition};
use gpui_kit::component::{Icon, StyledExt};
use gpui_kit::{
    div, px, rgb, rgba, AnyElement, App, Div, ElementId, FocusHandle, HoverListenerMode,
    IntoElement, ParentElement, Pixels, RenderOnce, Rgba, SharedString, StatefulInteractiveElement,
    Styled, Subscription, Window,
};
use std::time::Duration;

const HOVER_TRAVEL: Duration = Duration::from_millis(160);

#[derive(IntoElement)]
pub(super) struct RosterMore {
    pub id: SharedString,
    pub button: Button,
    pub faces: Vec<AnyElement>,
    pub row: Div,
    pub colors: ThemeColors,
    pub door: Option<bool>,
    pub enter_delay: Duration,
    pub reduced_motion: bool,
}

struct Interaction {
    hovered: bool,
    focus: FocusHandle,
    _focus_in: Subscription,
    _focus_out: Subscription,
}

struct Presentation {
    margin: Pixels,
    background: Rgba,
    foreground: Rgba,
    tail: Rgba,
}

fn is_engaged(hovered: bool, focused: bool, keyboard: bool) -> bool {
    hovered || (keyboard && focused)
}

fn presentation(colors: &ThemeColors, expanded: bool, engagement: f32) -> Presentation {
    let engagement = engagement.clamp(0., 1.);
    let mut background = rgba(colors.workflow_node_bg);
    let mut foreground = rgba(colors.foreground_subtle);
    let mut tail = rgba(colors.foreground_subtlest);
    tail.a += (rgba(colors.foreground_subtle).a - tail.a) * engagement;
    if expanded {
        background = rgba(colors.workflow_node_hover);
        foreground = rgb(colors.text_primary);
    } else {
        background.a += (rgba(colors.workflow_node_hover).a - background.a) * engagement;
        foreground.a += (1. - foreground.a) * engagement;
    }
    Presentation {
        margin: px(-5. + 3. * engagement),
        background,
        foreground,
        tail,
    }
}

impl RenderOnce for RosterMore {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let interaction = window.use_keyed_state(
            (ElementId::Name(self.id.clone()), "interaction"),
            cx,
            |window, cx| {
                let focus = cx.focus_handle().tab_stop(true);
                let focus_in = window.on_focus_in(&focus, cx, |window, _| window.refresh());
                let focus_out = window.on_focus_out(&focus, cx, |_, window, _| window.refresh());
                Interaction {
                    hovered: false,
                    focus,
                    _focus_in: focus_in,
                    _focus_out: focus_out,
                }
            },
        );
        let state = interaction.read(cx);
        let focus = state.focus.clone();
        let engaged = is_engaged(
            state.hovered,
            focus.is_focused(window),
            window.last_input_was_keyboard(),
        );
        let policy = Transition::new(if self.reduced_motion {
            Duration::ZERO
        } else {
            HOVER_TRAVEL
        })
        .ease(gpui_kit::base::animation::cubic_bezier(
            0.22, 0.61, 0.36, 1.,
        ));
        let engagement = transition(
            (self.id.clone(), "engagement"),
            if engaged { 1. } else { 0. },
            policy,
            window,
            cx,
        );
        let frame = presentation(&self.colors, self.door == Some(true), engagement);
        let tail = match self.door {
            None => Icon::new(Glyph::ArrowUpRight)
                .size(px(14.))
                .flex_shrink_0()
                .text_color(frame.tail)
                .into_any_element(),
            Some(expanded) => WorkflowChevron {
                id: self.id.clone(),
                kind: ChevronKind::Roster,
                expanded,
                reduced_motion: self.reduced_motion,
                color: frame.tail,
            }
            .into_any_element(),
        };
        let button = self
            .button
            .track_focus(&focus)
            .bg(frame.background)
            .hover_listener_mode(HoverListenerMode::InputModalityIndependent)
            .on_hover(move |hovered, window, cx| {
                interaction.update(cx, |state, _| state.hovered = *hovered);
                window.refresh();
            })
            .child(
                div()
                    .w_full()
                    .min_w_0()
                    .h_flex()
                    .items_center()
                    .gap(px(8.))
                    .child(div().h_flex().items_center().flex_shrink_0().children(
                        self.faces.into_iter().enumerate().map(|(index, face)| {
                            div()
                                .flex_shrink_0()
                                .ml(if index == 0 { px(0.) } else { frame.margin })
                                .child(face)
                        }),
                    ))
                    .child(self.row.flex_1().text_color(frame.foreground))
                    .child(tail),
            );
        WorkflowEntrance::arrive(
            self.id,
            div().w_full().min_w_0().flex_shrink_0().child(button),
            self.enter_delay,
            self.reduced_motion,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{is_engaged, presentation, HOVER_TRAVEL};
    use crate::theme::ThemeColors;
    use gpui_kit::{px, rgb, rgba};
    use std::time::Duration;

    #[test]
    fn hover_and_keyboard_visible_focus_engage_independently() {
        for keyboard in [false, true] {
            for focused in [false, true] {
                assert!(is_engaged(true, focused, keyboard));
            }
        }
        assert!(is_engaged(false, true, true));
        assert!(!is_engaged(false, true, false));
        assert!(!is_engaged(false, false, true));
        assert!(!is_engaged(false, false, false));

        // Tabbing away must not retract faces beneath a stationary pointer.
        assert!(is_engaged(true, false, true));
    }

    #[test]
    fn roster_faces_fan_out_by_three_pixels_over_the_reference_hover_duration() {
        assert_eq!(HOVER_TRAVEL, Duration::from_millis(160));
        let colors = ThemeColors::light();
        assert_eq!(presentation(&colors, false, 0.).margin, px(-5.));
        assert_eq!(presentation(&colors, false, 0.5).margin, px(-3.5));
        assert_eq!(presentation(&colors, false, 1.).margin, px(-2.));
        assert_eq!(presentation(&colors, false, -1.).margin, px(-5.));
        assert_eq!(presentation(&colors, false, 2.).margin, px(-2.));
    }

    #[test]
    fn hover_uses_the_actual_surface_alpha_and_foreground_in_both_themes() {
        for colors in [ThemeColors::light(), ThemeColors::dark()] {
            let idle = presentation(&colors, false, 0.);
            assert_eq!(idle.background, rgba(colors.workflow_node_bg));
            assert_eq!(idle.foreground, rgba(colors.foreground_subtle));
            assert_eq!(idle.tail, rgba(colors.foreground_subtlest));
            let hovered = presentation(&colors, false, 1.);
            assert_eq!(hovered.background, rgba(colors.workflow_node_hover));
            assert_eq!(hovered.foreground, rgb(colors.text_primary));
            assert_eq!(hovered.tail, rgba(colors.foreground_subtle));
            let halfway = presentation(&colors, false, 0.5);
            assert!(halfway.background.a > idle.background.a);
            assert!(halfway.background.a < hovered.background.a);
            assert!(halfway.foreground.a > idle.foreground.a);
            assert!(halfway.foreground.a < hovered.foreground.a);
            assert!(halfway.tail.a > idle.tail.a);
            assert!(halfway.tail.a < hovered.tail.a);
        }
    }

    #[test]
    fn an_open_roster_keeps_its_emphasis_without_forcing_the_faces_apart() {
        for colors in [ThemeColors::light(), ThemeColors::dark()] {
            for engagement in [0., 0.5, 1.] {
                let frame = presentation(&colors, true, engagement);
                assert_eq!(frame.background, rgba(colors.workflow_node_hover));
                assert_eq!(frame.foreground, rgb(colors.text_primary));
                assert_eq!(
                    frame.margin,
                    presentation(&colors, false, engagement).margin
                );
            }
        }
    }
}
