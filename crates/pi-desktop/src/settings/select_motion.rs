use crate::theme::ThemeColors;
use gpui_kit::assets::IconName as Glyph;
use gpui_kit::base::{transition, Transition};
use gpui_kit::component::select::Select;
use gpui_kit::component::Icon;
use gpui_kit::{
    div, px, rgba, App, Div, ElementId, HoverListenerMode, Hsla, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::time::Duration;

const BORDER_TRAVEL: Duration = Duration::from_millis(150);

fn hover_border(colors: &ThemeColors, hovered: bool) -> Hsla {
    rgba(if hovered {
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

fn select_shell() -> Div {
    div().min_w_0().h(px(32.)).flex_shrink_0()
}

fn select_indicator() -> Icon {
    // Select reapplies xsmall to custom icons; constraints preserve its 14px indicator.
    Icon::new(Glyph::ChevronDown)
        .min_w(px(14.))
        .max_w(px(14.))
        .min_h(px(14.))
        .max_h(px(14.))
}

#[derive(IntoElement)]
pub(super) struct SettingsSelect {
    id: SharedString,
    shell: Div,
    select: Select<Vec<SharedString>>,
    colors: ThemeColors,
    reduced_motion: bool,
}

impl SettingsSelect {
    pub(super) fn new(
        id: &'static str,
        select: Select<Vec<SharedString>>,
        colors: &ThemeColors,
        reduced_motion: bool,
    ) -> Self {
        Self {
            id: id.into(),
            shell: select_shell(),
            select: select.icon(select_indicator()),
            colors: *colors,
            reduced_motion,
        }
    }
}

impl Styled for SettingsSelect {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        self.shell.style()
    }
}

impl RenderOnce for SettingsSelect {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = ElementId::from(self.id);
        let pointer =
            window.use_keyed_state((id.clone(), "settings-select-pointer"), cx, |_, _| false);
        let border = transition(
            (id.clone(), "settings-select-border"),
            hover_border(&self.colors, *pointer.read(cx)),
            Transition::new(duration(self.reduced_motion))
                .ease(gpui_kit::base::animation::cubic_bezier(0.4, 0., 0.2, 1.)),
            window,
            cx,
        );
        // The native Select still owns focus, open-state paint and its popup.
        self.shell
            .id(id)
            .hover_listener_mode(HoverListenerMode::InputModalityIndependent)
            .on_hover(move |hovered, window, cx| {
                if pointer.read(cx) != hovered {
                    pointer.update(cx, |state, _| *state = *hovered);
                    window.refresh();
                }
            })
            .child(self.select.border_color(border))
    }
}

#[cfg(test)]
mod tests {
    use super::{duration, hover_border, select_indicator, select_shell};
    use crate::theme::ThemeColors;
    use gpui_kit::base::Interpolate;
    use gpui_kit::component::Sizable;
    use gpui_kit::{px, relative, rgba, Hsla, Styled};
    use std::time::Duration;

    #[test]
    fn indicator_keeps_14px_constraints_after_native_xsmall_adaptation() {
        let mut indicator = select_indicator().xsmall();
        let style = indicator.style();
        assert_eq!(style.min_size.width, Some(px(14.).into()));
        assert_eq!(style.max_size.width, Some(px(14.).into()));
        assert_eq!(style.min_size.height, Some(px(14.).into()));
        assert_eq!(style.max_size.height, Some(px(14.).into()));
        assert!(style.padding.left.is_none());
        assert!(style.padding.right.is_none());
        assert!(style.border_widths.left.is_none());
        assert!(style.background.is_none());
    }

    #[test]
    fn select_hover_uses_the_reference_neutral_border_in_both_themes() {
        for (colors, idle, hovered) in [
            (ThemeColors::light(), 0x0d0d0d1a, 0x0d0d0d26),
            (ThemeColors::dark(), 0xffffff1a, 0xffffff26),
        ] {
            assert_eq!(hover_border(&colors, false), Hsla::from(rgba(idle)));
            assert_eq!(hover_border(&colors, true), Hsla::from(rgba(hovered)));
        }
    }

    #[test]
    fn select_hover_motion_is_bounded_and_respects_reduced_motion() {
        assert_eq!(duration(false), Duration::from_millis(150));
        assert_eq!(duration(true), Duration::ZERO);
        for colors in [ThemeColors::light(), ThemeColors::dark()] {
            let from = hover_border(&colors, false);
            let to = hover_border(&colors, true);
            for progress in [0., 0.25, 0.5, 0.75, 1.] {
                let color = from.interpolate(&to, progress);
                assert!(color.a >= from.a && color.a <= to.a);
                assert_eq!((color.h, color.s, color.l), (from.h, from.s, from.l));
            }
        }
    }

    #[test]
    fn select_shell_reserves_one_trigger_frame_without_extra_paint_or_padding() {
        let mut shell = select_shell();
        let style = shell.style();
        assert!(style.size.width.is_none());
        assert_eq!(style.size.height, Some(px(32.).into()));
        assert_eq!(style.min_size.width, Some(px(0.).into()));
        assert_eq!(style.flex_shrink, Some(0.));
        assert!(style.background.is_none());
        assert!(style.border_widths.top.is_none());
        assert!(style.padding.top.is_none());
        assert!(style.padding.right.is_none());
        assert!(style.padding.bottom.is_none());
        assert!(style.padding.left.is_none());
        assert!(style.overflow.x.is_none());
        assert!(style.overflow.y.is_none());
    }

    #[test]
    fn select_shell_leaves_width_to_the_content_column_or_footer_pair() {
        let mut content = select_shell().w_full();
        assert_eq!(content.style().size.width, Some(relative(1.).into()));
        let mut footer = select_shell().flex_1();
        let style = footer.style();
        assert!(style.size.width.is_none());
        assert_eq!(style.flex_grow, Some(1.));
        assert_eq!(style.flex_shrink, Some(1.));
        assert_eq!(style.flex_basis, Some(relative(0.).into()));
        assert_eq!(style.min_size.width, Some(px(0.).into()));
        assert_eq!(style.size.height, Some(px(32.).into()));
    }
}
