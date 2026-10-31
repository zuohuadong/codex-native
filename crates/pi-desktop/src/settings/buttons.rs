use crate::theme::ThemeColors;
use crate::Language;
use gpui_kit::assets::IconName as Glyph;
use gpui_kit::component::button::{Button, ButtonCustomVariant, ButtonVariants};
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{Icon, Selectable, Sizable, StyledExt};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    div, px, rgb, rgba, App, Div, ElementId, InteractiveElement, ParentElement, Pixels, Styled,
};

fn action_content(label: &'static str, icon: Glyph, icon_size: Pixels, font_size: Pixels) -> Div {
    div()
        .h_flex()
        .items_center()
        .justify_center()
        .gap(px(4.))
        .text_size(font_size)
        .line_height(px(20.))
        .font_normal()
        .whitespace_nowrap()
        .child(Icon::new(icon).size(icon_size).flex_shrink_0())
        .child(label)
}

/// Form actions size to their content; navigation rows deliberately fill theirs.
pub(super) fn save_button(label: &'static str, font_size: Pixels) -> Button {
    Button::new("save-settings")
        .primary()
        .large()
        .h(px(32.))
        .px(px(10.))
        .border_1()
        .border_color(rgba(0x00000000))
        .rounded(px(8.))
        .self_start()
        .child(action_content(label, Glyph::Check, px(16.), font_size))
        .accessibility_label(label)
}

pub(super) fn secondary_button(
    id: impl Into<ElementId>,
    label: &'static str,
    icon: Glyph,
    font_size: Pixels,
) -> Button {
    Button::new(id)
        .ghost()
        .small()
        .h(px(28.))
        .px(px(8.))
        .border_1()
        .border_color(rgba(0x00000000))
        .rounded(px(6.))
        .child(action_content(label, icon, px(14.), font_size))
        .accessibility_label(label)
}

fn api_key_visibility_label(masked: bool, language: Language) -> &'static str {
    match (masked, language) {
        (true, Language::Chinese) => "显示 API Key",
        (false, Language::Chinese) => "隐藏 API Key",
        (true, _) => "Show API key",
        (false, _) => "Hide API key",
    }
}

fn native_tooltip(mut button: Button, label: &'static str) -> Button {
    // Managed component tooltips would replace the color motion's hover listener.
    button
        .interactivity()
        .tooltip(move |window, cx| Tooltip::new(label).build(window, cx));
    button
}

pub(super) fn api_key_visibility_button(masked: bool, language: Language) -> Button {
    let label = api_key_visibility_label(masked, language);
    let button = Button::new("settings-key-visibility")
        .ghost()
        .small()
        .w(px(24.))
        .h(px(24.))
        .p_0()
        .border_1()
        .border_color(rgba(0x00000000))
        .rounded(px(6.))
        .child(
            Icon::new(if masked { Glyph::Eye } else { Glyph::EyeOff })
                .size(px(14.))
                .flex_shrink_0(),
        )
        .accessibility_label(label);
    native_tooltip(button, label)
}

pub(super) fn nav_content(
    label: &'static str,
    icon: Glyph,
    compact: bool,
    font_size: Pixels,
) -> Div {
    div()
        .w_full()
        .min_w_0()
        .h_flex()
        .items_center()
        .text_size(font_size)
        .line_height(px(20.))
        .font_normal()
        .when(compact, |content| content.justify_center())
        .when(!compact, |content| content.gap(px(8.)))
        .child(Icon::new(icon).size(px(16.)).flex_shrink_0())
        .when(!compact, |content| {
            content.child(div().flex_1().min_w_0().truncate().child(label))
        })
}

pub(super) fn nav_button(
    id: impl Into<ElementId>,
    label: &'static str,
    compact: bool,
    content: Div,
) -> Button {
    let button = Button::new(id)
        .ghost()
        .small()
        .w_full()
        .h(px(32.))
        .px(px(10.))
        .rounded(px(12.))
        .when(compact, |button| {
            button.w(px(40.)).h(px(40.)).p_0().mx_auto()
        })
        .child(content)
        .accessibility_label(label);
    native_tooltip(button, label)
}

fn navigation_variant(colors: &ThemeColors, cx: &App) -> ButtonCustomVariant {
    ButtonCustomVariant::new(cx)
        .foreground(rgb(colors.text_primary).into())
        .hover(rgba(colors.selected).into())
        .active(rgba(colors.selected).into())
}

fn back_frame(button: Button, compact: bool, colors: &ThemeColors) -> Button {
    button
        .bg(rgba(colors.card_user_bg))
        .w(px(if compact { 40. } else { 244. }))
        .when(!compact, |button| button.px(px(6.)))
        .m(px(4.))
}

pub(super) fn section_button(
    id: impl Into<ElementId>,
    label: &'static str,
    compact: bool,
    content: Div,
    selected: bool,
    colors: &ThemeColors,
    cx: &App,
) -> Button {
    nav_button(id, label, compact, content)
        .custom(navigation_variant(colors, cx))
        .selected(selected)
        .toggled(selected)
}

pub(super) fn back_button(
    label: &'static str,
    compact: bool,
    font_size: Pixels,
    colors: &ThemeColors,
    cx: &App,
) -> Button {
    back_frame(
        nav_button(
            "settings-back",
            label,
            compact,
            nav_content(label, Glyph::ArrowLeft, compact, font_size),
        )
        .custom(
            navigation_variant(colors, cx)
                .color(rgba(colors.card_user_bg).into())
                .foreground(rgb(colors.remote_foreground_muted).into()),
        ),
        compact,
        colors,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        action_content, api_key_visibility_button, api_key_visibility_label, back_frame,
        save_button, secondary_button,
    };
    use crate::theme::ThemeColors;
    use crate::Language;
    use gpui_kit::assets::IconName as Glyph;
    use gpui_kit::component::button::Button;
    use gpui_kit::{px, rgba, FontWeight, JustifyContent, Styled};

    #[test]
    fn command_frames_reserve_the_reference_one_pixel_transparent_border() {
        for language in [Language::Chinese, Language::English] {
            for font_size in [12., 14., 20.] {
                let commands = [
                    save_button(language.text().save_btn, px(font_size)),
                    secondary_button(
                        "secondary-border",
                        "Use in task",
                        Glyph::Plus,
                        px(font_size),
                    ),
                    api_key_visibility_button(true, language),
                    api_key_visibility_button(false, language),
                ];
                for mut button in commands {
                    let style = button.style();
                    let border = &style.border_widths;
                    assert_eq!(border.top, Some(px(1.).into()));
                    assert_eq!(border.right, Some(px(1.).into()));
                    assert_eq!(border.bottom, Some(px(1.).into()));
                    assert_eq!(border.left, Some(px(1.).into()));
                    assert_eq!(style.border_color, Some(rgba(0x00000000).into()));
                }
            }
        }
    }

    #[test]
    fn secondary_actions_keep_content_width_and_reference_default_geometry() {
        for label in [
            "用于任务",
            "Use in task",
            "安装或修复快捷方式",
            "Install or repair shortcuts",
        ] {
            for font_size in [12., 14., 20.] {
                let mut button =
                    secondary_button("settings-action-test", label, Glyph::Plus, px(font_size));
                let style = button.style();
                assert_eq!(style.size.height, Some(px(28.).into()));
                assert!(style.size.width.is_none());
                assert!(style.align_self.is_none());
                assert_eq!(style.padding.left, Some(px(8.).into()));
                assert_eq!(style.padding.right, Some(px(8.).into()));
            }
        }
    }

    #[test]
    fn action_content_centers_interface_text_without_a_full_width_navigation_row() {
        for font_size in [12., 14., 20.] {
            for (label, icon, icon_size) in [
                ("保存配置", Glyph::Check, 16.),
                ("Save Preferences", Glyph::Check, 16.),
                ("用于任务", Glyph::Plus, 14.),
                ("Use in task", Glyph::Plus, 14.),
            ] {
                let mut content = action_content(label, icon, px(icon_size), px(font_size));
                let style = content.style();
                assert!(style.size.width.is_none());
                assert_eq!(style.justify_content, Some(JustifyContent::Center));
                assert_eq!(style.gap.width, Some(px(4.).into()));
                assert_eq!(style.text.font_size, Some(px(font_size).into()));
                assert_eq!(style.text.font_weight, Some(FontWeight::NORMAL));
                assert_eq!(style.text.line_height, Some(px(20.).into()));
            }
        }
    }

    #[test]
    fn key_visibility_button_keeps_fixed_geometry_in_both_states_and_languages() {
        for language in [Language::Chinese, Language::English] {
            for masked in [false, true] {
                let mut button = api_key_visibility_button(masked, language);
                let style = button.style();
                assert_eq!(style.size.width, Some(px(24.).into()));
                assert_eq!(style.size.height, Some(px(24.).into()));
                assert_eq!(style.padding.left, Some(px(0.).into()));
                assert_eq!(style.padding.right, Some(px(0.).into()));
            }
        }
        assert_eq!(
            api_key_visibility_label(true, Language::Chinese),
            "显示 API Key"
        );
        assert_eq!(
            api_key_visibility_label(false, Language::Chinese),
            "隐藏 API Key"
        );
        assert_eq!(
            api_key_visibility_label(true, Language::English),
            "Show API key"
        );
        assert_eq!(
            api_key_visibility_label(false, Language::English),
            "Hide API key"
        );
    }

    #[test]
    fn back_frame_keeps_the_reference_idle_surface_in_both_themes() {
        for colors in [ThemeColors::light(), ThemeColors::dark()] {
            for compact in [false, true] {
                let mut button = back_frame(Button::new("settings-back"), compact, &colors);
                assert_eq!(
                    button.style().background,
                    Some(rgba(colors.card_user_bg).into())
                );
                assert_eq!(
                    button.style().size.width,
                    Some(px(if compact { 40. } else { 244. }).into())
                );
                assert_eq!(button.style().margin.top, Some(px(4.).into()));
            }
        }
    }
}
