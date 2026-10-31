use crate::theme::ThemeColors;
use gpui_kit::base::{transition, Transition};
use gpui_kit::component::button::{Button, ButtonCustomVariant, ButtonVariants};
use gpui_kit::component::{ActiveTheme, Disableable};
use gpui_kit::{
    rgb, rgba, App, ElementId, HoverListenerMode, Hsla, InteractiveElement, IntoElement,
    RenderOnce, SharedString, Styled, Window,
};
use std::time::Duration;

const COLOR_TRAVEL: Duration = Duration::from_millis(150);

#[derive(Clone, Copy)]
pub(super) enum CommandKind {
    Primary,
    Neutral,
    Navigation { selected: bool },
    Back,
}

struct Paint {
    background: Hsla,
    foreground: Hsla,
}

impl CommandKind {
    fn paint(self, colors: &ThemeColors, dark: bool, hovered: bool, disabled: bool) -> Paint {
        let hovered = hovered && !disabled;
        match self {
            Self::Primary => {
                let mut background = rgb(if dark { 0xffffff } else { 0x000000 });
                background.a = if hovered { 0.8 } else { 1. };
                Paint {
                    background: background.into(),
                    foreground: rgb(if dark { 0x000000 } else { 0xffffff }).into(),
                }
            }
            Self::Neutral => {
                let mut background = rgba(colors.hover);
                if !hovered {
                    background.a = 0.;
                }
                Paint {
                    background: background.into(),
                    foreground: rgb(colors.text_primary).into(),
                }
            }
            Self::Navigation { selected } => {
                let mut background = rgba(colors.selected);
                if !selected && !hovered {
                    background.a = 0.;
                }
                Paint {
                    background: background.into(),
                    foreground: rgb(colors.text_primary).into(),
                }
            }
            Self::Back => Paint {
                background: rgba(if hovered {
                    colors.selected
                } else {
                    colors.card_user_bg
                })
                .into(),
                foreground: rgb(if hovered {
                    colors.text_primary
                } else {
                    colors.remote_foreground_muted
                })
                .into(),
            },
        }
    }
}

fn duration(reduced_motion: bool, disabled: bool) -> Duration {
    if reduced_motion || disabled {
        Duration::ZERO
    } else {
        COLOR_TRAVEL
    }
}

fn command_frame(button: Button, paint: &Paint, disabled: bool) -> Button {
    // Instance colors also win over the component's selected/disabled palette.
    button
        .bg(paint.background)
        .text_color(paint.foreground)
        .opacity(if disabled { 0.5 } else { 1. })
}

#[derive(IntoElement)]
pub(super) struct SettingsCommand {
    id: SharedString,
    button: Button,
    kind: CommandKind,
    colors: ThemeColors,
    reduced_motion: bool,
    disabled: bool,
}

impl SettingsCommand {
    pub(super) fn new(
        id: impl Into<SharedString>,
        button: Button,
        kind: CommandKind,
        colors: &ThemeColors,
        reduced_motion: bool,
    ) -> Self {
        Self {
            id: id.into(),
            button,
            kind,
            colors: *colors,
            reduced_motion,
            disabled: false,
        }
    }

    pub(super) fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for SettingsCommand {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let pointer = window.use_keyed_state(
            (ElementId::Name(self.id.clone()), "settings-command-pointer"),
            cx,
            |_, _| false,
        );
        let paint = self.kind.paint(
            &self.colors,
            cx.theme().is_dark(),
            *pointer.read(cx),
            self.disabled,
        );
        let policy = Transition::new(duration(self.reduced_motion, self.disabled))
            .ease(gpui_kit::base::animation::cubic_bezier(0.4, 0., 0.2, 1.));
        let background = transition(
            (self.id.clone(), "settings-command-background"),
            paint.background,
            policy.clone(),
            window,
            cx,
        );
        let foreground = transition(
            (self.id, "settings-command-foreground"),
            paint.foreground,
            policy,
            window,
            cx,
        );
        let mut button = command_frame(
            self.button.disabled(self.disabled).custom(
                ButtonCustomVariant::new(cx)
                    .color(background)
                    .foreground(foreground)
                    .hover(background)
                    .active(background),
            ),
            &Paint {
                background,
                foreground,
            },
            self.disabled,
        );
        // Observe the real pointer even while disabled, without changing activation.
        button
            .interactivity()
            .hover_listener_mode(HoverListenerMode::InputModalityIndependent);
        button.interactivity().on_hover(move |hovered, window, cx| {
            if pointer.read(cx) != hovered {
                pointer.update(cx, |state, _| *state = *hovered);
                window.refresh();
            }
        });
        button
    }
}

#[cfg(test)]
mod tests {
    use super::{command_frame, duration, CommandKind, COLOR_TRAVEL};
    use crate::theme::ThemeColors;
    use gpui_kit::base::Interpolate;
    use gpui_kit::component::button::Button;
    use gpui_kit::{px, rgb, rgba, Hsla, Styled};
    use std::time::Duration;

    const KINDS: [CommandKind; 5] = [
        CommandKind::Primary,
        CommandKind::Neutral,
        CommandKind::Navigation { selected: false },
        CommandKind::Navigation { selected: true },
        CommandKind::Back,
    ];

    #[test]
    fn primary_keeps_black_white_ink_and_fades_only_the_fill_on_hover() {
        for (colors, dark, background, foreground) in [
            (ThemeColors::light(), false, 0x000000, 0xffffff),
            (ThemeColors::dark(), true, 0xffffff, 0x000000),
        ] {
            let idle = CommandKind::Primary.paint(&colors, dark, false, false);
            let hovered = CommandKind::Primary.paint(&colors, dark, true, false);
            assert_eq!(idle.background, Hsla::from(rgb(background)));
            assert_eq!(hovered.background, Hsla::from(rgb(background).alpha(0.8)));
            assert_eq!(idle.foreground, Hsla::from(rgb(foreground)));
            assert_eq!(hovered.foreground, idle.foreground);
        }
    }

    #[test]
    fn neutral_uses_the_hover_token_instead_of_accent_or_selected() {
        for (colors, dark) in [(ThemeColors::light(), false), (ThemeColors::dark(), true)] {
            let idle = CommandKind::Neutral.paint(&colors, dark, false, false);
            let hovered = CommandKind::Neutral.paint(&colors, dark, true, false);
            assert_eq!(idle.background.a, 0.);
            assert_eq!(hovered.background, Hsla::from(rgba(colors.hover)));
            assert_eq!(idle.foreground, Hsla::from(rgb(colors.text_primary)));
            assert_eq!(hovered.foreground, idle.foreground);
            if dark {
                assert_ne!(hovered.background, Hsla::from(rgba(colors.selected)));
            }
        }
    }

    #[test]
    fn navigation_selection_stays_emphasized_after_the_pointer_leaves() {
        for (colors, dark) in [(ThemeColors::light(), false), (ThemeColors::dark(), true)] {
            for selected in [false, true] {
                let kind = CommandKind::Navigation { selected };
                let idle = kind.paint(&colors, dark, false, false);
                let hovered = kind.paint(&colors, dark, true, false);
                assert_eq!(hovered.background, Hsla::from(rgba(colors.selected)));
                assert_eq!(idle.foreground, Hsla::from(rgb(colors.text_primary)));
                assert_eq!(hovered.foreground, idle.foreground);
                if selected {
                    assert_eq!(idle.background, hovered.background);
                } else {
                    assert_eq!(idle.background.a, 0.);
                }
            }
        }
    }

    #[test]
    fn back_transitions_surface_and_muted_ink_together() {
        for (colors, dark) in [(ThemeColors::light(), false), (ThemeColors::dark(), true)] {
            let idle = CommandKind::Back.paint(&colors, dark, false, false);
            let hovered = CommandKind::Back.paint(&colors, dark, true, false);
            assert_eq!(idle.background, Hsla::from(rgba(colors.card_user_bg)));
            assert_eq!(hovered.background, Hsla::from(rgba(colors.selected)));
            assert_eq!(
                idle.foreground,
                Hsla::from(rgb(colors.remote_foreground_muted))
            );
            assert_eq!(hovered.foreground, Hsla::from(rgb(colors.text_primary)));
            for progress in [0., 0.25, 0.5, 0.75, 1.] {
                let foreground = idle.foreground.interpolate(&hovered.foreground, progress);
                assert_eq!(foreground.a, 1.);
                assert!(foreground.l >= idle.foreground.l.min(hovered.foreground.l));
                assert!(foreground.l <= idle.foreground.l.max(hovered.foreground.l));
            }
        }
    }

    #[test]
    fn disabled_targets_are_idle_even_when_the_pointer_remains_inside() {
        for (colors, dark) in [(ThemeColors::light(), false), (ThemeColors::dark(), true)] {
            for kind in KINDS {
                let idle = kind.paint(&colors, dark, false, false);
                let disabled = kind.paint(&colors, dark, true, true);
                assert_eq!(disabled.background, idle.background);
                assert_eq!(disabled.foreground, idle.foreground);
            }
        }
    }

    #[test]
    fn disabled_commands_dim_the_whole_idle_palette_without_changing_geometry() {
        for (colors, dark) in [(ThemeColors::light(), false), (ThemeColors::dark(), true)] {
            for kind in KINDS {
                for disabled in [false, true] {
                    let paint = kind.paint(&colors, dark, false, disabled);
                    let mut button = command_frame(
                        Button::new("settings-command-frame")
                            .h(px(32.))
                            .px(px(10.))
                            .border_1()
                            .border_color(rgba(0x00000000)),
                        &paint,
                        disabled,
                    );
                    let style = button.style();
                    assert_eq!(style.opacity, Some(if disabled { 0.5 } else { 1. }));
                    assert_eq!(style.background, Some(paint.background.into()));
                    assert_eq!(style.text.color, Some(paint.foreground));
                    assert_eq!(style.size.height, Some(px(32.).into()));
                    assert!(style.size.width.is_none());
                    assert_eq!(style.padding.left, Some(px(10.).into()));
                    assert_eq!(style.padding.right, Some(px(10.).into()));
                    assert_eq!(style.border_color, Some(rgba(0x00000000).into()));
                    assert_eq!(style.border_widths.top, Some(px(1.).into()));
                }
            }
        }
    }

    #[test]
    fn reference_timing_snaps_when_reduced_or_disabled_and_alpha_stays_bounded() {
        assert_eq!(COLOR_TRAVEL, Duration::from_millis(150));
        assert_eq!(duration(false, false), COLOR_TRAVEL);
        assert_eq!(duration(true, false), Duration::ZERO);
        assert_eq!(duration(false, true), Duration::ZERO);
        assert_eq!(duration(true, true), Duration::ZERO);
        for (colors, dark) in [(ThemeColors::light(), false), (ThemeColors::dark(), true)] {
            for kind in KINDS {
                let from = kind.paint(&colors, dark, false, false).background;
                let to = kind.paint(&colors, dark, true, false).background;
                let halfway = from.interpolate(&to, 0.5);
                assert!(halfway.a >= from.a.min(to.a));
                assert!(halfway.a <= from.a.max(to.a));
                assert!((halfway.a - (from.a + to.a) / 2.).abs() < 0.0001);
            }
        }
    }
}
