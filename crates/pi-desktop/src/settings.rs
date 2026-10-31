use super::*;
use gpui_kit::assets::IconName as Glyph;
use gpui_kit::base::{Switch as BaseSwitch, SwitchThumb, SwitchTrack};
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::Input;
use gpui_kit::component::select::Select;
use gpui_kit::prelude::FluentBuilder;
mod buttons;
mod command_motion;
mod field_motion;
mod select_motion;
#[cfg(test)]
use buttons::nav_button as settings_nav_button;
use buttons::{
    api_key_visibility_button, back_button as settings_back_button,
    nav_content as settings_nav_content, save_button as settings_save_button,
    secondary_button as settings_secondary_button, section_button as settings_section_button,
};
use command_motion::{CommandKind, SettingsCommand};
use field_motion::SettingsField;
use select_motion::SettingsSelect;

/// Application-owned geometry with the library's native switch semantics.
#[derive(IntoElement)]
struct AppearanceSwitch {
    checked: bool,
    reduced_motion: bool,
    label: SharedString,
    on_change: Box<dyn Fn(&bool, &mut Window, &mut App)>,
}

impl RenderOnce for AppearanceSwitch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        use gpui_kit::base::{transition, Transition};
        let id = "reduced-motion";
        let focus = window
            .use_keyed_state((ElementId::from(id), "focus"), cx, |_, cx| {
                cx.focus_handle()
            })
            .read(cx)
            .clone();
        let focused = focus.is_focused(window);
        let policy = Transition::new(std::time::Duration::from_millis(if self.reduced_motion {
            0
        } else {
            150
        }))
        .ease(gpui_kit::base::animation::cubic_bezier(0.4, 0., 0.2, 1.));
        let offset = transition(
            (id, "thumb-offset"),
            if self.checked { px(14.) } else { px(0.) },
            policy.clone(),
            window,
            cx,
        );
        let track_color = transition(
            (id, "track-color"),
            if self.checked {
                cx.theme().primary
            } else {
                cx.theme().switch
            },
            policy,
            window,
            cx,
        );
        BaseSwitch::new(id)
            .checked(self.checked)
            .accessibility_label(self.label)
            .track_focus(&focus)
            .on_change(move |next, _, window, cx| (self.on_change)(&next, window, cx))
            .relative()
            .w(px(32.))
            .h(px(18.))
            .flex_shrink_0()
            .cursor_pointer()
            // Expanded pointer target without changing the row's layout size.
            .child(
                div()
                    .absolute()
                    .left(px(-12.))
                    .top(px(-8.))
                    .w(px(56.))
                    .h(px(34.)),
            )
            .child(
                SwitchTrack::new((ElementId::from(id), "track"))
                    .checked(self.checked)
                    .w(px(32.))
                    .h(px(18.))
                    .p(px(1.))
                    .rounded_full()
                    .bg(track_color)
                    .when(focused, |track| track.focus_ring_style(window, cx))
                    .child(
                        SwitchThumb::new(self.checked)
                            .size(px(16.))
                            .rounded_full()
                            .left(offset)
                            .bg(cx.theme().switch_thumb),
                    ),
            )
    }
}

pub(super) fn theme_labels(language: Language) -> Vec<SharedString> {
    let text = language.text();
    vec![
        text.theme_light.into(),
        text.theme_dark.into(),
        text.theme_auto.into(),
    ]
}
pub(super) fn theme_index(theme: ThemePreference) -> usize {
    match theme {
        ThemePreference::Light => 0,
        ThemePreference::Dark => 1,
        ThemePreference::Auto => 2,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(super) enum Section {
    #[default]
    General,
    Appearance,
    Providers,
    AgentCapabilities,
    Subagents,
    Skills,
    Shortcuts,
    Activity,
}

impl Section {
    pub(super) fn key(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Appearance => "appearance",
            Self::Providers => "providers",
            Self::AgentCapabilities => "agent-capabilities",
            Self::Subagents => "subagents",
            Self::Skills => "skills",
            Self::Shortcuts => "shortcuts",
            Self::Activity => "activity",
        }
    }

    fn label(self, language: Language) -> &'static str {
        match (self, language) {
            (Self::General, Language::Chinese) => "常规",
            (Self::Appearance, Language::Chinese) => "外观",
            (Self::Providers, Language::Chinese) => "模型服务",
            (Self::AgentCapabilities, Language::Chinese) => "智能体能力",
            (Self::Subagents, Language::Chinese) => "子代理",
            (Self::Skills, Language::Chinese) => "技能",
            (Self::Shortcuts, Language::Chinese) => "快捷键",
            (Self::Activity, Language::Chinese) => "用量",
            (Self::General, _) => "General",
            (Self::Appearance, _) => "Appearance",
            (Self::Providers, _) => "Model providers",
            (Self::AgentCapabilities, _) => "Agent capabilities",
            (Self::Subagents, _) => "Subagents",
            (Self::Skills, _) => "Skills",
            (Self::Shortcuts, _) => "Shortcuts",
            (Self::Activity, _) => "Usage",
        }
    }

    fn icon(self) -> Glyph {
        match self {
            Self::General => Glyph::Settings2,
            Self::Appearance => Glyph::Palette,
            Self::Providers => Glyph::Package,
            Self::AgentCapabilities => Glyph::Package,
            Self::Subagents => Glyph::Bot,
            Self::Skills => Glyph::WandSparkles,
            Self::Shortcuts => Glyph::Keyboard,
            Self::Activity => Glyph::ChartColumn,
        }
    }
}

/// Local subset of the reference settings navigation.
/// Shortcuts stay in Basics; Usage is the only data section.
/// Agent capabilities remains a group title, not a navigable row.
pub(super) fn settings_nav_groups() -> [(&'static str, &'static [Section]); 3] {
    [
        (
            "basics",
            &[
                Section::General,
                Section::Appearance,
                Section::Providers,
                Section::Shortcuts,
            ],
        ),
        ("agent-capabilities", &[Section::Subagents, Section::Skills]),
        ("data-and-stats", &[Section::Activity]),
    ]
}

pub(super) fn section_from_key(key: Option<&str>) -> Section {
    match key {
        Some("appearance") => Section::Appearance,
        Some("providers") => Section::Providers,
        Some("agent-capabilities") => Section::AgentCapabilities,
        Some("subagents") => Section::Subagents,
        Some("skills") => Section::Skills,
        // Automations is a primary workbench surface in ZCode, not a settings tab.
        // Migrate older persisted selections to the capabilities landing page.
        Some("automations") => Section::AgentCapabilities,
        Some("shortcuts") => Section::Shortcuts,
        Some("activity") => Section::Activity,
        _ => Section::General,
    }
}

/// ZCode `w-28`: the px suffix sits inside the field, and the value is right-aligned.
const INTERFACE_FONT_FIELD_WIDTH: Pixels = px(112.);
const INTERFACE_FONT_FIELD_HEIGHT: Pixels = px(28.);

fn interface_font_frame() -> Div {
    div()
        .relative()
        .w(INTERFACE_FONT_FIELD_WIDTH)
        .h(INTERFACE_FONT_FIELD_HEIGHT)
        .flex_shrink_0()
}

fn interface_font_input<T: Styled>(control: T, font_size: Pixels, c: &ThemeColors) -> T {
    settings_input(control, INTERFACE_FONT_FIELD_HEIGHT, font_size, c)
        .w_full()
        .rounded(px(6.))
        .pl(px(8.))
        .pr(px(32.))
        .text_align(gpui::TextAlign::Right)
}

fn interface_font_unit(font_size: Pixels, c: &ThemeColors) -> Div {
    div()
        .absolute()
        .top_0()
        .right(px(8.))
        .h_full()
        .h_flex()
        .items_center()
        .text_size(font_size + px(2.))
        .text_color(rgba(c.foreground_subtle))
        .child("px")
}

fn settings_title(label: &'static str, compact: bool, font_size: Pixels) -> Div {
    // ZCode title tokens add a fixed offset; they do not multiply the UI scale.
    div()
        .w_full()
        .min_w_0()
        .flex_shrink_0()
        .text_size(font_size + px(if compact { 10. } else { 16. }))
        .line_height(relative(if compact { 4. / 3. } else { 1.2 }))
        .font_semibold()
        .child(label)
}

fn settings_group_label(label: &'static str, font_size: Pixels, c: &ThemeColors) -> Div {
    div()
        .min_w_0()
        .px(px(10.))
        .pb_1()
        .text_size(font_size - px(2.))
        .line_height(relative(1.5))
        .text_color(rgb(c.remote_foreground_muted))
        .font_medium()
        .child(label)
}

fn settings_caption(label: impl Into<SharedString>, font_size: Pixels, c: &ThemeColors) -> Div {
    div()
        .text_size(font_size - px(2.))
        .text_color(rgba(c.foreground_subtle))
        .child(label.into())
}

fn settings_frame(font_size: Pixels, c: &ThemeColors) -> Div {
    div()
        .h_flex()
        .flex_1()
        .size_full()
        .min_w_0()
        .min_h_0()
        .bg(rgb(c.sidebar_bg))
        .text_size(font_size)
}

fn settings_save_row() -> Div {
    div()
        .w_full()
        .min_w_0()
        .min_h(px(32.))
        .flex_shrink_0()
        .h_flex()
        .flex_wrap()
        .items_center()
        .gap_x(px(12.))
        .gap_y(px(8.))
}

fn settings_save_feedback(label: &'static str, font_size: Pixels, c: &ThemeColors) -> Div {
    div()
        .min_w_0()
        .max_w_full()
        .text_size(font_size)
        .line_height(relative(1.5))
        .text_color(rgb(c.accent_green))
        .child(label)
}

fn card(c: &ThemeColors) -> Div {
    div()
        .v_flex()
        .gap_0()
        .p_0()
        .w_full()
        .overflow_hidden()
        .rounded(px(12.))
        .border_1()
        .border_color(rgba(c.settings_border))
        .bg(rgb(c.card_bg))
}

fn setting_row_frame(narrow: bool) -> Div {
    div()
        .h_flex()
        .w_full()
        .min_w_0()
        .items_center()
        .justify_between()
        .gap_4()
        .px_4()
        .py_3()
        .when(narrow, |row| row.v_flex().items_start())
}

fn setting_control_frame(narrow: bool) -> Div {
    div()
        .min_w_0()
        .flex_shrink_0()
        .h_flex()
        .items_center()
        .justify_end()
        .when(narrow, |control| control.w_full())
        .when(!narrow, |control| control.w(px(192.)))
}

fn setting_row(
    label: impl Into<SharedString>,
    description: impl Into<SharedString>,
    control: AnyElement,
    narrow: bool,
    c: &ThemeColors,
) -> Div {
    setting_row_frame(narrow)
        .child(
            div()
                .v_flex()
                .gap_1()
                .min_w_0()
                .when(narrow, |text| text.w_full())
                .when(!narrow, |text| text.flex_1())
                .child(div().font_medium().child(label.into()))
                .child(
                    div()
                        .line_height(px(24.))
                        .text_color(rgba(c.muted_foreground))
                        .child(description.into()),
                ),
        )
        .child(setting_control_frame(narrow).child(control))
}

fn settings_input<T: Styled>(control: T, height: Pixels, font_size: Pixels, c: &ThemeColors) -> T {
    // Use Styled::h: Input's inherent h() only affects multiline editors.
    control
        .h(height)
        .min_w_0()
        .px(px(12.))
        .py_0()
        .rounded(px(8.))
        .border_1()
        .border_color(rgba(c.settings_border))
        .bg(rgb(c.dock_bg))
        .text_size(font_size)
        .line_height(px(20.))
}

fn settings_select<T: Styled>(control: T, font_size: Pixels, c: &ThemeColors) -> T {
    settings_input(control, px(32.), font_size, c).pr(px(8.))
}

#[derive(Clone, Copy)]
enum Shortcut {
    Primary(&'static str),
    PrimaryShift(&'static str),
    Escape,
}

impl Shortcut {
    fn parts(self, apple: bool) -> Vec<&'static str> {
        match (self, apple) {
            (Self::Primary(key), true) => vec!["⌘", key],
            (Self::Primary(key), false) => vec!["Ctrl", key],
            (Self::PrimaryShift(key), true) => vec!["⇧", "⌘", key],
            (Self::PrimaryShift(key), false) => vec!["Ctrl", "Shift", key],
            (Self::Escape, _) => vec!["Esc"],
        }
    }
}

fn shortcut_keycap(
    key: &'static str,
    font_size: Pixels,
    apple: bool,
    mono: SharedString,
    c: &ThemeColors,
) -> Div {
    div()
        .h_flex()
        .items_center()
        .justify_center()
        .flex_shrink_0()
        .h(px(16.))
        .min_w(px(16.))
        .px(px(6.))
        .rounded(px(6.))
        .bg(rgb(c.shortcut_bg))
        .text_color(rgb(c.shortcut_foreground))
        .text_size(font_size - px(4.))
        .font_medium()
        .font_family(if apple { ".SystemUIFont".into() } else { mono })
        .whitespace_nowrap()
        .child(key)
}

fn shortcut_row_frame(narrow: bool) -> Div {
    div()
        .h_flex()
        .items_center()
        .justify_between()
        .min_w_0()
        .gap_3()
        .px_4()
        .py_3()
        .when(narrow, |row| row.v_flex().items_start().gap_2())
}

fn shortcut_key_group() -> Div {
    div()
        .h_flex()
        .items_center()
        .flex_wrap()
        .min_w_0()
        .max_w_full()
        .gap(px(4.))
}

fn provider_field(
    label: &'static str,
    label_size: Pixels,
    control: impl IntoElement,
    c: &ThemeColors,
) -> Div {
    div()
        .v_flex()
        .gap(px(4.))
        .child(
            div()
                .text_size(label_size)
                .line_height(px(20.))
                .text_color(rgba(c.foreground_subtle))
                .child(label),
        )
        .child(control)
}

impl PiDesktopApp {
    pub(super) fn settings_view(
        &self,
        window: &Window,
        c: &ThemeColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let compact = window.viewport_size().width < px(1024.);
        let narrow = window.viewport_size().width < px(640.);
        let mut navigation = div()
            .v_flex()
            .w(px(if compact { 68. } else { 268. }))
            .h_full()
            .flex_shrink_0()
            .bg(rgb(c.sidebar_bg))
            .child(div().h(px(48.)))
            .child(
                div().px_2().pt_3().pb_3().child(SettingsCommand::new(
                    "settings-back",
                    settings_back_button(
                        self.tr("返回", "Back"),
                        compact,
                        self.ui_text(14.),
                        c,
                        cx,
                    )
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.show_settings = false;
                        this.focus_composer(window, cx);
                        cx.notify();
                    })),
                    CommandKind::Back,
                    c,
                    self.preferences.reduced_motion,
                )),
            );
        let mut nav_items = div()
            .id("settings-nav")
            .v_flex()
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .px_2()
            .pb_3()
            .gap_4();
        let groups = settings_nav_groups();
        let mut index = 0usize;
        for (group_index, (group_id, sections)) in groups.into_iter().enumerate() {
            let group_label = match group_id {
                "basics" => self.tr("基础设置", "Basics"),
                "agent-capabilities" => self.tr("智能体能力", "Agent capabilities"),
                _ => self.tr("数据与统计", "Data & stats"),
            };
            let mut group_view = div()
                .v_flex()
                .gap_1()
                .when(compact && group_index > 0, |group| {
                    group
                        .border_t(px(1.))
                        .border_color(rgba(c.settings_border))
                        .pt_3()
                });
            if !compact {
                group_view =
                    group_view.child(settings_group_label(group_label, self.ui_text(14.), c));
            }
            for section in sections {
                let section = *section;
                let item_index = index;
                index += 1;
                let label = section.label(self.language);
                let selected = self.settings_section == section;
                let id = format!("settings-section-{item_index}");
                let button = settings_section_button(
                    id.clone(),
                    label,
                    compact,
                    settings_nav_content(label, section.icon(), compact, self.ui_text(14.)),
                    selected,
                    c,
                    cx,
                )
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.settings_section = section;
                    this.preferences.settings_section = Some(section.key().into());
                    this.settings_error.clear();
                    this.settings_saved = false;
                    this.persist_active_session();
                    cx.notify();
                }));
                group_view = group_view.child(SettingsCommand::new(
                    id,
                    button,
                    CommandKind::Navigation { selected },
                    c,
                    self.preferences.reduced_motion,
                ));
            }
            nav_items = nav_items.child(group_view);
        }
        navigation = navigation.child(nav_items);
        if !compact {
            navigation = navigation.child(self.settings_footer(c));
        }
        let content_inset = if cfg!(target_os = "windows") { 6. } else { 4. };
        let content = match self.settings_section {
            Section::General => self.general_settings(narrow, c, cx),
            Section::Appearance => self.appearance_settings(narrow, c, cx),
            Section::Providers => self.provider_settings(c, cx),
            Section::Shortcuts => self.shortcut_settings(window, c, cx),
            Section::AgentCapabilities => self.agent_capabilities_settings(narrow, c),
            Section::Subagents => self.subagents_settings(narrow, c),
            Section::Skills => self.skills_settings(c, cx),
            Section::Activity => self.activity_settings(c),
        };
        settings_frame(self.ui_text(14.), c)
            .child(navigation)
            .child(
                div()
                    .v_flex()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .pr(px(content_inset))
                    .pb(px(content_inset))
                    .pt(px(content_inset))
                    .when(compact && cfg!(target_os = "macos"), |pane| {
                        pane.pt(px(64.))
                    })
                    .child(
                        div()
                            .v_flex()
                            .size_full()
                            .min_h_0()
                            .rounded(px(12.))
                            .border_1()
                            .border_color(rgba(c.settings_border))
                            .bg(rgb(c.root_bg))
                            .child(div().h(px(48.)).flex_shrink_0())
                            .child(
                                div()
                                    .id("settings-scroll")
                                    .overflow_y_scroll()
                                    .flex_1()
                                    .min_h_0()
                                    .w_full()
                                    .child(
                                        div()
                                            .v_flex()
                                            .mx_auto()
                                            .w_full()
                                            .max_w(px(896.))
                                            .px(px(if compact { 16. } else { 32. }))
                                            .pb(px(if compact { 32. } else { 40. }))
                                            .gap_8()
                                            .child(settings_title(
                                                self.settings_section.label(self.language),
                                                compact,
                                                self.ui_text(14.),
                                            ))
                                            .child(content)
                                            .when(!self.settings_error.is_empty(), |view| {
                                                view.child(
                                                    div()
                                                        .text_color(rgb(c.workflow_error))
                                                        .child(self.settings_error.clone()),
                                                )
                                            })
                                            .when(
                                                matches!(
                                                    self.settings_section,
                                                    Section::General
                                                        | Section::Appearance
                                                        | Section::Providers
                                                ),
                                                |view| {
                                                    view.child(
                                                        settings_save_row()
                                                            .child(SettingsCommand::new(
                                                                "save-settings",
                                                                settings_save_button(
                                                                    self.t().save_btn,
                                                                    self.ui_text(14.),
                                                                )
                                                                .on_click(cx.listener(
                                                                    |this, _, window, cx| {
                                                                        this.save_preferences(
                                                                            window, cx,
                                                                        )
                                                                    },
                                                                )),
                                                                CommandKind::Primary,
                                                                c,
                                                                self.preferences.reduced_motion,
                                                            ))
                                                            .when(self.settings_saved, |row| {
                                                                row.child(settings_save_feedback(
                                                                    self.t().save_success,
                                                                    self.ui_text(14.),
                                                                    c,
                                                                ))
                                                            }),
                                                    )
                                                },
                                            ),
                                    ),
                            ),
                    ),
            )
            .into_any_element()
    }

    fn general_settings(
        &self,
        narrow: bool,
        c: &ThemeColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        card(c)
            .child(setting_row(
                self.tr("界面语言", "Interface language"),
                self.tr(
                    "选择应用 UI 的显示语言。",
                    "Choose the display language for the app UI.",
                ),
                SettingsSelect::new(
                    "settings-general-language",
                    settings_select(
                        Select::new(&self.settings_general_language),
                        self.ui_text(14.),
                        c,
                    )
                    .w_full()
                    .focus_ring(false)
                    .accessibility_label(self.tr("界面语言", "Interface language")),
                    c,
                    self.preferences.reduced_motion,
                )
                .w_full()
                .into_any_element(),
                narrow,
                c,
            ))
            .when(cfg!(target_os = "windows"), |view| {
                view.child(SettingsCommand::new(
                    "install",
                    settings_secondary_button(
                        "install",
                        self.tr("安装或修复快捷方式", "Install or repair shortcuts"),
                        Glyph::Download,
                        self.ui_text(14.),
                    )
                    .self_start()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.settings_error = match installer::install(true) {
                            Ok(()) => String::new(),
                            Err(error) => error.to_string(),
                        };
                        cx.notify();
                    })),
                    CommandKind::Neutral,
                    c,
                    self.preferences.reduced_motion,
                ))
            })
            .into_any_element()
    }

    fn settings_footer(&self, c: &ThemeColors) -> AnyElement {
        div()
            .flex_shrink_0()
            .border_t_1()
            .border_color(rgba(c.settings_border))
            .px_3()
            .pt_3()
            .pb_4()
            .v_flex()
            .gap_2()
            .child(
                settings_caption(self.tr("界面", "Interface"), self.ui_text(14.), c).font_medium(),
            )
            .child(
                div()
                    .h_flex()
                    .gap_2()
                    .child(
                        SettingsSelect::new(
                            "settings-footer-language",
                            settings_select(
                                Select::new(&self.settings_footer_language),
                                self.ui_text(14.),
                                c,
                            )
                            .w_full()
                            .focus_ring(false)
                            .accessibility_label(self.tr("界面语言", "Interface language")),
                            c,
                            self.preferences.reduced_motion,
                        )
                        .flex_1(),
                    )
                    .child(
                        SettingsSelect::new(
                            "settings-footer-theme",
                            settings_select(
                                Select::new(&self.settings_footer_theme),
                                self.ui_text(14.),
                                c,
                            )
                            .w_full()
                            .focus_ring(false)
                            .accessibility_label(self.tr("主题", "Theme")),
                            c,
                            self.preferences.reduced_motion,
                        )
                        .flex_1(),
                    ),
            )
            .into_any_element()
    }

    fn provider_settings(&self, c: &ThemeColors, cx: &mut Context<Self>) -> AnyElement {
        let label_size = self.ui_text(14.);
        let base_label = self.t().api_base_label;
        let key_label = self.t().api_key_label;
        let model_label = self.t().model_name_label;
        let masked = self.settings_key_input.read(cx).presentation().is_masked();
        div()
            .v_flex()
            .gap_4()
            .child(
                div()
                    .text_size(label_size)
                    .line_height(px(24.))
                    .text_color(rgba(c.foreground_subtle))
                    .child(self.tr(
                        "配置当前会话使用的自定义模型服务。保存后才会生效。",
                        "Configure the custom model service for this session. Changes apply after saving.",
                    )),
            )
            .child(
                card(c)
                    .p_4()
                    .gap_3()
                    .child(provider_field(
                        base_label,
                        label_size,
                        SettingsField::new(
                            &self.settings_base_input,
                            settings_input(
                                Input::new(&self.settings_base_input),
                                px(32.),
                                label_size,
                                c,
                            )
                            .w_full()
                            .aria_label(base_label),
                            c,
                            self.preferences.reduced_motion,
                        ),
                        c,
                    ))
                    .child(provider_field(
                        key_label,
                        label_size,
                        div()
                            .relative()
                            .w_full()
                            .h(px(36.))
                            .child(
                                SettingsField::new(
                                    &self.settings_key_input,
                                    settings_input(
                                        Input::new(&self.settings_key_input),
                                        px(36.),
                                        label_size,
                                        c,
                                    )
                                    .w_full()
                                    .pr(px(40.))
                                    .aria_label(key_label),
                                    c,
                                    self.preferences.reduced_motion,
                                ),
                            )
                            .child(
                                SettingsCommand::new(
                                    "settings-key-visibility",
                                    api_key_visibility_button(masked, self.language)
                                        .absolute()
                                        .right(px(6.))
                                        .top(px(6.))
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.settings_key_input.update(cx, |state, cx| {
                                                state.toggle_masked(window, cx)
                                            });
                                            cx.notify();
                                        })),
                                    CommandKind::Neutral,
                                    c,
                                    self.preferences.reduced_motion,
                                ),
                            ),
                        c,
                    ))
                    .child(provider_field(
                        model_label,
                        label_size,
                        SettingsField::new(
                            &self.settings_model_input,
                            settings_input(
                                Input::new(&self.settings_model_input),
                                px(32.),
                                label_size,
                                c,
                            )
                            .w_full()
                            .aria_label(model_label),
                            c,
                            self.preferences.reduced_motion,
                        ),
                        c,
                    ))
                    .child(
                        settings_caption(
                            format!(
                                "{} · {}",
                                mask_api_key(&self.api_key, self.language),
                                self.tr(
                                    "密钥仅用于本次运行；留空保留当前密钥。",
                                    "Key is used for this run only; leave blank to keep the current key."
                                )
                            ),
                            self.ui_text(14.),
                            c,
                        )
                        .line_height(px(20.)),
                    ),
            )
            .into_any_element()
    }

    fn appearance_settings(
        &self,
        narrow: bool,
        c: &ThemeColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .v_flex()
            .gap_3()
            .child(
                div()
                    .v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_size(self.ui_text(14.) + px(2.))
                            .font_semibold()
                            .child(self.tr("界面", "Interface")),
                    )
                    .child(
                        div()
                            .text_size(self.ui_text(14.))
                            .line_height(px(24.))
                            .text_color(rgba(c.foreground_subtle))
                            .child(self.tr(
                                "调整主题和界面文字大小。图标、间距和固定的工作流尺寸保持不变。",
                                "Adjust the theme and interface text size. Icons, spacing, and fixed workflow geometry stay unchanged.",
                            )),
                    ),
            )
            .child(
                card(c)
                    .child(setting_row(
                        self.tr("主题", "Theme"),
                        self.tr(
                            "选择明亮、暗黑或跟随系统主题",
                            "Choose light, dark, or system theme",
                        ),
                        SettingsSelect::new(
                            "settings-appearance-theme",
                            settings_select(
                                Select::new(&self.settings_appearance_theme),
                                self.ui_text(14.),
                                c,
                            )
                            .w_full()
                            .focus_ring(false)
                            .accessibility_label(self.tr("主题", "Theme")),
                            c,
                            self.preferences.reduced_motion,
                        )
                        .w_full()
                        .into_any_element(),
                        narrow,
                        c,
                    ))
                    .child(div().h(px(1.)).bg(rgba(c.settings_border)))
                    .child(setting_row(
                        self.tr("界面字号", "Interface text size"),
                        self.tr(
                            "12–20 px，默认 14。按 Enter 或离开输入框后生效。",
                            "12–20 px. The default is 14. Apply with Enter or by leaving the field.",
                        ),
                        interface_font_frame()
                            .child(
                                SettingsField::new(
                                    &self.settings_font_input,
                                    interface_font_input(
                                        Input::new(&self.settings_font_input),
                                        self.ui_text(14.),
                                        c,
                                    )
                                    .aria_label(self.tr("界面字号", "Interface text size")),
                                    c,
                                    self.preferences.reduced_motion,
                                ),
                            )
                            .child(interface_font_unit(self.ui_text(14.), c))
                            .into_any_element(),
                        narrow,
                        c,
                    ))
                    .child(div().h(px(1.)).bg(rgba(c.settings_border)))
                    .child(setting_row(
                        self.tr("减少动态效果", "Reduce motion"),
                        self.tr(
                            "减少界面过渡和流式输出动画",
                            "Reduce transitions and streaming motion",
                        ),
                        AppearanceSwitch {
                            checked: self.preferences.reduced_motion,
                            reduced_motion: self.preferences.reduced_motion,
                            label: self.tr("减少动态效果", "Reduce motion").into(),
                            on_change: Box::new(cx.listener(|this, checked: &bool, _, cx| {
                                this.preferences.reduced_motion = *checked;
                                this.settings_saved = false;
                                cx.notify();
                            })),
                        }
                        .into_any_element(),
                        narrow,
                        c,
                    )),
            )
            .into_any_element()
    }

    fn agent_capabilities_settings(&self, narrow: bool, c: &ThemeColors) -> AnyElement {
        card(c)
            .child(setting_row(
                self.tr("工作流", "Workflows"),
                self.tr(
                    "多 agent 阶段、交付物和停止操作都留在当前会话里。",
                    "Multi-agent stages, deliverables, and stop controls stay in the current conversation.",
                ),
                div()
                    .text_size(self.ui_text(14.))
                    .text_color(rgba(c.foreground_subtle))
                    .child(self.tr("本地运行", "Local"))
                    .into_any_element(),
                narrow,
                c,
            ))
            .child(div().h(px(1.)).bg(rgba(c.settings_border)))
            .child(setting_row(
                self.tr("工具调用", "Tool calls"),
                self.tr(
                    "脚本和命令输出使用会话内的原生结果块，而不是独立终端。",
                    "Script and command output uses native result blocks in the conversation, not a separate terminal.",
                ),
                div()
                    .text_size(self.ui_text(14.))
                    .text_color(rgba(c.foreground_subtle))
                    .child(self.tr("已启用", "Enabled"))
                    .into_any_element(),
                narrow,
                c,
            ))
            .into_any_element()
    }

    fn subagents_settings(&self, narrow: bool, c: &ThemeColors) -> AnyElement {
        card(c)
            .child(setting_row(
                self.tr("并行执行", "Parallel execution"),
                self.tr(
                    "子代理在当前工作流中并行检查工作区，并把阶段状态回传到同一会话。",
                    "Subagents inspect the workspace in parallel and return stage status to the same conversation.",
                ),
                div()
                    .text_size(self.ui_text(14.))
                    .text_color(rgba(c.foreground_subtle))
                    .child(self.tr("只读", "Read-only"))
                    .into_any_element(),
                narrow,
                c,
            ))
            .child(div().h(px(1.)).bg(rgba(c.settings_border)))
            .child(setting_row(
                self.tr("时限", "Deadline"),
                self.tr(
                    "每个子代理的请求和持续输出共用 90 秒上限。",
                    "Each specialist shares a 90 second limit across the request and continued output.",
                ),
                div()
                    .text_size(self.ui_text(14.))
                    .text_color(rgba(c.foreground_subtle))
                    .child("90s")
                    .into_any_element(),
                narrow,
                c,
            ))
            .child(div().h(px(1.)).bg(rgba(c.settings_border)))
            .child(setting_row(
                self.tr("停止", "Stop"),
                self.tr(
                    "工作流卡片上的停止只结束该工作流中仍在运行的子代理。",
                    "Stop on a workflow card ends only the specialists still running for that workflow.",
                ),
                div()
                    .text_size(self.ui_text(14.))
                    .text_color(rgba(c.foreground_subtle))
                    .child(self.tr("按工作流", "Per workflow"))
                    .into_any_element(),
                narrow,
                c,
            ))
            .into_any_element()
    }

    fn skills_settings(&self, c: &ThemeColors, cx: &mut Context<Self>) -> AnyElement {
        let skills: Vec<_> = self.skill_registry.all().to_vec();
        let count = skills.len();
        div()
            .v_flex()
            .gap_4()
            .child(
                div()
                    .h(px(28.))
                    .h_flex()
                    .items_center()
                    .gap(px(6.))
                    .child(
                        div()
                            .text_size(self.ui_text(14.))
                            .font_medium()
                            .child(self.tr("内置技能", "Built-in skills")),
                    )
                    .child(
                        settings_caption(count.to_string(), self.ui_text(14.), c),
                    ),
            )
            .child(
                div()
                    .text_size(self.ui_text(14.))
                    .line_height(px(24.))
                    .text_color(rgba(c.foreground_subtle))
                    .child(self.tr(
                        "这些技能随应用内置。打开详情查看说明，或把技能指令插入当前任务。",
                        "These skills ship with the app. Open a skill for its instructions, or insert them into the current task.",
                    )),
            )
            .child(
                div()
                    .v_flex()
                    .rounded(px(12.))
                    .overflow_hidden()
                    .bg(rgba(c.card_user_bg))
                    .children(skills.into_iter().enumerate().map(|(index, skill)| {
                        let name = if self.language == Language::Chinese {
                            skill.name.clone()
                        } else {
                            skill.name_en.clone()
                        };
                        let description = if self.language == Language::Chinese {
                            skill.description.clone()
                        } else {
                            skill.description_en.clone()
                        };
                        let prompt = format!("{}\n\n任务 / Task: ", skill.system_prompt_snippet);
                        let detail = skill.clone();
                        div()
                            .h_flex()
                            .items_center()
                            .min_w_0()
                            .gap_3()
                            .px_4()
                            .py_3()
                            .when(index > 0, |row| {
                                row.border_t_1()
                                    .border_color(rgba(c.settings_border).alpha(0.05))
                            })
                            .hover(|style| style.bg(rgba(c.hover)))
                            .child(
                                div()
                                    .size(px(36.))
                                    .flex_shrink_0()
                                    .rounded(px(12.))
                                    .bg(rgb(c.root_bg))
                                    .h_flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        Icon::new(Glyph::WandSparkles)
                                            .size(px(16.))
                                            .text_color(rgba(c.foreground_subtle)),
                                    ),
                            )
                            .child(
                                Button::new(format!("settings-skill-{}", skill.id))
                                    .ghost()
                                    .flex_1()
                                    .min_w_0()
                                    .h_auto()
                                    .p_0()
                                    .justify_start()
                                    .accessibility_label(format!(
                                        "{} · {}",
                                        name,
                                        self.tr("查看详情", "View details")
                                    ))
                                    .child(
                                        div()
                                            .w_full()
                                            .min_w_0()
                                            .v_flex()
                                            .gap(px(2.))
                                            .child(
                                                div()
                                                    .truncate()
                                                    .text_size(self.ui_text(14.))
                                                    .font_medium()
                                                    .child(name),
                                            )
                                            .child(
                                                settings_caption(
                                                    description,
                                                    self.ui_text(14.),
                                                    c,
                                                )
                                                .truncate(),
                                            ),
                                    )
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.open_skill_detail(&detail, window, cx);
                                    })),
                            )
                            .child(
                                SettingsCommand::new(
                                    format!("settings-use-{}", skill.id),
                                    settings_secondary_button(
                                        format!("settings-use-{}", skill.id),
                                        self.tr("用于任务", "Use in task"),
                                        Glyph::Plus,
                                        self.ui_text(14.),
                                    )
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        let prompt = prompt.clone();
                                        this.show_settings = false;
                                        this.selected_nav = "chat";
                                        this.input_state.update(cx, |input, cx| {
                                            input.set_value(prompt.clone(), window, cx)
                                        });
                                        this.drafts.insert(this.session.id.clone(), prompt);
                                        this.persist_active_session();
                                        this.focus_composer(window, cx);
                                    })),
                                    CommandKind::Neutral,
                                    c,
                                    self.preferences.reduced_motion,
                                )
                                .disabled(self.is_executing),
                            )
                    })),
            )
            .into_any_element()
    }

    fn activity_settings(&self, c: &ThemeColors) -> AnyElement {
        let usage = local_usage(
            &self.session_history,
            &self.session,
            self.task_states
                .iter()
                .map(|(id, state)| (id.as_str(), state.tokens)),
            self.tokens_used,
        );
        let items = [
            (
                self.tr("累计 Token 数", "Total tokens"),
                usage.tokens.to_string(),
            ),
            (
                self.tr("工具调用", "Tool calls"),
                usage.tool_calls.to_string(),
            ),
            (self.tr("会话", "Sessions"), usage.sessions.to_string()),
        ];
        div()
            .v_flex()
            .gap_5()
            .child(
                div()
                    .text_size(self.ui_text(14.))
                    .line_height(px(24.))
                    .text_color(rgba(c.foreground_subtle))
                    .child(self.tr(
                        "来自本机保存的会话。不连接远端账户或云端用量接口。",
                        "Built from sessions stored on this machine. No remote account or cloud usage API is connected.",
                    )),
            )
            .child(usage_strip(
                &items,
                self.ui_text(14.) + px(2.),
                self.ui_text(14.),
                c,
            ))
            .into_any_element()
    }

    fn shortcut_settings(
        &self,
        window: &Window,
        c: &ThemeColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mono = cx.theme().mono_font_family.clone();
        let apple = cfg!(target_os = "macos");
        let narrow = window.viewport_size().width < px(640.);
        let rows = [
            (self.tr("新任务", "New task"), Shortcut::Primary("N")),
            (
                self.tr("显示或隐藏侧栏", "Toggle sidebar"),
                Shortcut::Primary("B"),
            ),
            (self.tr("搜索任务", "Search tasks"), Shortcut::Primary("K")),
            (self.tr("设置", "Settings"), Shortcut::Primary(",")),
            (
                self.tr("聚焦输入框", "Focus composer"),
                Shortcut::Primary("L"),
            ),
            (
                self.tr("命令输出", "Command output"),
                Shortcut::Primary("J"),
            ),
            (
                self.tr("审查更改", "Review changes"),
                Shortcut::PrimaryShift("D"),
            ),
            (self.tr("返回", "Back"), Shortcut::Escape),
        ];
        card(c)
            .children(rows.into_iter().enumerate().map(|(index, (label, key))| {
                shortcut_row_frame(narrow)
                    .when(index > 0, |row| {
                        row.border_t_1().border_color(rgba(c.settings_border))
                    })
                    .child(
                        div()
                            .min_w_0()
                            .when(!narrow, |label| label.flex_1())
                            .text_size(self.ui_text(14.))
                            .child(label),
                    )
                    .child(
                        shortcut_key_group().children(key.parts(apple).into_iter().map(|part| {
                            shortcut_keycap(part, self.ui_text(14.), apple, mono.clone(), c)
                        })),
                    )
            }))
            .into_any_element()
    }
}

struct LocalUsage {
    tokens: usize,
    tool_calls: usize,
    sessions: usize,
}

fn session_tool_calls(messages: &[Message]) -> usize {
    let mut finished = HashSet::new();
    let mut count = 0usize;
    for message in messages {
        if let Some(result) = &message.tool_result {
            finished.insert(result.tool_call_id.clone());
            count += 1;
        }
    }
    for message in messages {
        if let Some(calls) = &message.tool_calls {
            count += calls
                .iter()
                .filter(|call| !finished.contains(&call.id))
                .count();
        }
    }
    count
}

fn local_usage<'a>(
    sessions: &HashMap<String, Session>,
    active: &Session,
    stored_tokens: impl IntoIterator<Item = (&'a str, usize)>,
    active_tokens: usize,
) -> LocalUsage {
    let mut tokens = active_tokens;
    for (id, count) in stored_tokens {
        if id != active.id {
            tokens = tokens.saturating_add(count);
        }
    }
    let mut tool_calls = session_tool_calls(&active.messages);
    let mut session_count = 1usize;
    for (id, session) in sessions {
        if id == &active.id {
            continue;
        }
        session_count += 1;
        tool_calls += session_tool_calls(&session.messages);
    }
    LocalUsage {
        tokens,
        tool_calls,
        sessions: session_count,
    }
}

fn usage_strip(
    items: &[(&'static str, String)],
    value_size: Pixels,
    label_size: Pixels,
    c: &ThemeColors,
) -> Div {
    div()
        .h_flex()
        .items_center()
        .overflow_hidden()
        .rounded(px(12.))
        .bg(rgba(c.card_user_bg))
        .children(items.iter().enumerate().map(|(index, (label, value))| {
            div()
                .flex_1()
                .min_w_0()
                .h_flex()
                .items_center()
                .when(index > 0, |cell| {
                    cell.child(
                        div()
                            .w(px(1.))
                            .h(px(28.))
                            .flex_shrink_0()
                            .bg(rgba(c.settings_border)),
                    )
                })
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .px_4()
                        .py_3()
                        .v_flex()
                        .items_center()
                        .child(
                            div()
                                .w_full()
                                .truncate()
                                .text_center()
                                .text_size(value_size)
                                .font_medium()
                                .child(value.clone()),
                        )
                        .child(
                            div()
                                .w_full()
                                .mt_1()
                                .truncate()
                                .text_center()
                                .text_size(label_size)
                                .text_color(rgba(c.foreground_subtle))
                                .child(*label),
                        ),
                )
        }))
}

#[cfg(test)]
mod tests {
    use super::{
        card, interface_font_frame, interface_font_input, interface_font_unit, local_usage,
        section_from_key, session_tool_calls, setting_control_frame, setting_row_frame,
        settings_caption, settings_frame, settings_group_label, settings_input,
        settings_nav_button, settings_nav_content, settings_nav_groups, settings_save_button,
        settings_save_feedback, settings_save_row, settings_select, settings_title,
        shortcut_key_group, shortcut_keycap, shortcut_row_frame, theme_index, theme_labels, Glyph,
        Language, Message, Section, Session, SharedString, Shortcut, ThemeColors, ThemePreference,
        INTERFACE_FONT_FIELD_HEIGHT, INTERFACE_FONT_FIELD_WIDTH,
    };
    use gpui_kit::{
        px, relative, rems, rgb, rgba, AlignItems, AlignSelf, FlexDirection, FlexWrap, FontWeight,
        JustifyContent, Overflow, Position, Styled, TextAlign, WhiteSpace,
    };
    use std::collections::HashMap;

    #[test]
    fn save_feedback_row_wraps_without_stretching_or_framing_the_action() {
        let mut row = settings_save_row();
        let style = row.style();
        assert_eq!(style.flex_direction, Some(FlexDirection::Row));
        assert_eq!(style.flex_wrap, Some(FlexWrap::Wrap));
        assert_eq!(style.align_items, Some(AlignItems::Center));
        assert_eq!(style.size.width, Some(relative(1.).into()));
        assert_eq!(style.min_size.width, Some(px(0.).into()));
        assert_eq!(style.min_size.height, Some(px(32.).into()));
        assert!(style.size.height.is_none());
        assert_eq!(style.gap.width, Some(px(12.).into()));
        assert_eq!(style.gap.height, Some(px(8.).into()));
        assert!(style.background.is_none());
        assert!(style.padding.left.is_none());
        assert!(style.padding.right.is_none());
        assert!(style.overflow.x.is_none());
        assert!(style.overflow.y.is_none());
    }

    #[test]
    fn save_feedback_keeps_localized_text_readable_at_each_interface_size() {
        for colors in [ThemeColors::light(), ThemeColors::dark()] {
            for language in [Language::Chinese, Language::English] {
                for font_size in [12., 14., 20.] {
                    let mut feedback = settings_save_feedback(
                        language.text().save_success,
                        px(font_size),
                        &colors,
                    );
                    let style = feedback.style();
                    assert_eq!(style.min_size.width, Some(px(0.).into()));
                    assert_eq!(style.max_size.width, Some(relative(1.).into()));
                    assert!(style.size.height.is_none());
                    assert_eq!(style.text.font_size, Some(px(font_size).into()));
                    assert_eq!(style.text.line_height, Some(relative(1.5).into()));
                    assert_eq!(style.text.color, Some(rgb(colors.accent_green).into()));
                    assert!(style.text.white_space.is_none());
                    assert!(style.background.is_none());
                }
            }
        }
    }

    #[test]
    fn settings_rows_stack_in_narrow_windows_without_a_fixed_height() {
        for narrow in [false, true] {
            let mut row = setting_row_frame(narrow);
            let style = row.style();
            assert_eq!(
                style.flex_direction,
                Some(if narrow {
                    FlexDirection::Column
                } else {
                    FlexDirection::Row
                })
            );
            assert_eq!(
                style.align_items,
                Some(if narrow {
                    AlignItems::FlexStart
                } else {
                    AlignItems::Center
                })
            );
            assert_eq!(style.size.width, Some(relative(1.).into()));
            assert_eq!(style.min_size.width, Some(px(0.).into()));
            assert!(style.size.height.is_none());
            assert_eq!(style.gap.width, Some(rems(1.).into()));
            assert_eq!(style.gap.height, Some(rems(1.).into()));
            assert_eq!(style.padding.left, Some(rems(1.).into()));
            assert_eq!(style.padding.right, Some(rems(1.).into()));
            assert_eq!(style.padding.top, Some(rems(0.75).into()));
            assert_eq!(style.padding.bottom, Some(rems(0.75).into()));
        }
    }

    #[test]
    fn settings_controls_fill_narrow_rows_and_keep_the_wide_reference_column() {
        for narrow in [false, true] {
            let mut control = setting_control_frame(narrow);
            let style = control.style();
            assert_eq!(
                style.size.width,
                Some(if narrow {
                    relative(1.).into()
                } else {
                    px(192.).into()
                })
            );
            assert_eq!(style.min_size.width, Some(px(0.).into()));
            assert_eq!(style.flex_shrink, Some(0.));
            assert_eq!(style.align_items, Some(AlignItems::Center));
            assert_eq!(style.justify_content, Some(JustifyContent::End));
            assert!(style.size.height.is_none());
        }
    }

    #[test]
    fn settings_cards_constrain_overflow_and_keep_the_reference_frame() {
        for colors in [ThemeColors::light(), ThemeColors::dark()] {
            let mut card = card(&colors);
            let style = card.style();
            assert_eq!(style.overflow.x, Some(Overflow::Hidden));
            assert_eq!(style.overflow.y, Some(Overflow::Hidden));
            assert_eq!(style.size.width, Some(relative(1.).into()));
            assert!(style.size.height.is_none());
            assert_eq!(style.padding.left, Some(px(0.).into()));
            assert_eq!(style.padding.right, Some(px(0.).into()));
            assert_eq!(style.padding.top, Some(px(0.).into()));
            assert_eq!(style.padding.bottom, Some(px(0.).into()));
            assert_eq!(style.corner_radii.top_left, Some(px(12.).into()));
            assert_eq!(style.corner_radii.top_right, Some(px(12.).into()));
            assert_eq!(style.corner_radii.bottom_left, Some(px(12.).into()));
            assert_eq!(style.corner_radii.bottom_right, Some(px(12.).into()));
            assert_eq!(style.background, Some(rgb(colors.card_bg).into()));
            assert_eq!(
                style.border_color,
                Some(rgba(colors.settings_border).into())
            );
        }
    }

    #[test]
    fn shortcut_parts_use_platform_modifier_names_and_order() {
        for key in ["N", "B", "K", ",", "L", "J"] {
            assert_eq!(Shortcut::Primary(key).parts(true), vec!["⌘", key]);
            assert_eq!(Shortcut::Primary(key).parts(false), vec!["Ctrl", key]);
        }
        assert_eq!(Shortcut::PrimaryShift("D").parts(true), vec!["⇧", "⌘", "D"]);
        assert_eq!(
            Shortcut::PrimaryShift("D").parts(false),
            vec!["Ctrl", "Shift", "D"]
        );
        for apple in [false, true] {
            assert_eq!(Shortcut::Escape.parts(apple), vec!["Esc"]);
        }
    }

    #[test]
    fn shortcut_keycaps_keep_reference_geometry_palette_and_platform_font() {
        for colors in [ThemeColors::light(), ThemeColors::dark()] {
            for font_size in [12., 14., 20.] {
                for apple in [false, true] {
                    let mono = SharedString::from("Test Mono");
                    let mut keycap =
                        shortcut_keycap("K", px(font_size), apple, mono.clone(), &colors);
                    let style = keycap.style();
                    assert_eq!(style.size.height, Some(px(16.).into()));
                    assert!(style.size.width.is_none());
                    assert_eq!(style.min_size.width, Some(px(16.).into()));
                    assert_eq!(style.flex_shrink, Some(0.));
                    assert_eq!(style.padding.left, Some(px(6.).into()));
                    assert_eq!(style.padding.right, Some(px(6.).into()));
                    assert_eq!(style.corner_radii.top_left, Some(px(6.).into()));
                    assert_eq!(style.corner_radii.top_right, Some(px(6.).into()));
                    assert_eq!(style.corner_radii.bottom_left, Some(px(6.).into()));
                    assert_eq!(style.corner_radii.bottom_right, Some(px(6.).into()));
                    assert_eq!(style.align_items, Some(AlignItems::Center));
                    assert_eq!(style.justify_content, Some(JustifyContent::Center));
                    assert_eq!(style.background, Some(rgb(colors.shortcut_bg).into()));
                    assert_eq!(
                        style.text.color,
                        Some(rgb(colors.shortcut_foreground).into())
                    );
                    assert_eq!(style.text.font_size, Some(px(font_size - 4.).into()));
                    assert_eq!(style.text.font_weight, Some(FontWeight::MEDIUM));
                    assert_eq!(
                        style.text.font_family,
                        Some(if apple { ".SystemUIFont".into() } else { mono })
                    );
                    assert_eq!(style.text.white_space, Some(WhiteSpace::Nowrap));
                }
            }
        }
    }

    #[test]
    fn shortcut_rows_stack_on_narrow_layouts_without_fixing_the_height() {
        for narrow in [false, true] {
            let mut row = shortcut_row_frame(narrow);
            let style = row.style();
            assert_eq!(
                style.flex_direction,
                Some(if narrow {
                    FlexDirection::Column
                } else {
                    FlexDirection::Row
                })
            );
            assert_eq!(
                style.align_items,
                Some(if narrow {
                    AlignItems::FlexStart
                } else {
                    AlignItems::Center
                })
            );
            assert_eq!(style.justify_content, Some(JustifyContent::SpaceBetween));
            assert_eq!(style.min_size.width, Some(px(0.).into()));
            assert!(style.size.height.is_none());
            assert_eq!(style.padding.left, Some(rems(1.).into()));
            assert_eq!(style.padding.right, Some(rems(1.).into()));
            assert_eq!(style.padding.top, Some(rems(0.75).into()));
            assert_eq!(style.padding.bottom, Some(rems(0.75).into()));
            let gap = Some(rems(if narrow { 0.5 } else { 0.75 }).into());
            assert_eq!(style.gap.width, gap);
            assert_eq!(style.gap.height, gap);
        }
    }

    #[test]
    fn shortcut_groups_wrap_between_keys_within_the_available_width() {
        let mut group = shortcut_key_group();
        let style = group.style();
        assert_eq!(style.flex_direction, Some(FlexDirection::Row));
        assert_eq!(style.flex_wrap, Some(FlexWrap::Wrap));
        assert_eq!(style.align_items, Some(AlignItems::Center));
        assert_eq!(style.min_size.width, Some(px(0.).into()));
        assert_eq!(style.max_size.width, Some(relative(1.).into()));
        assert_eq!(style.gap.width, Some(px(4.).into()));
        assert_eq!(style.gap.height, Some(px(4.).into()));
        assert!(style.size.height.is_none());
    }

    #[test]
    fn settings_selects_keep_reference_insets_without_squeezing_the_text_row() {
        for colors in [ThemeColors::light(), ThemeColors::dark()] {
            for font_size in [12., 14., 20.] {
                let mut control = settings_select(gpui_kit::div(), px(font_size), &colors);
                let style = control.style();
                assert_eq!(style.size.height, Some(px(32.).into()));
                assert!(style.size.width.is_none());
                assert_eq!(style.min_size.width, Some(px(0.).into()));
                assert_eq!(style.padding.left, Some(px(12.).into()));
                assert_eq!(style.padding.right, Some(px(8.).into()));
                assert_eq!(style.padding.top, Some(px(0.).into()));
                assert_eq!(style.padding.bottom, Some(px(0.).into()));
                assert_eq!(style.text.font_size, Some(px(font_size).into()));
                assert_eq!(style.text.line_height, Some(px(20.).into()));
                assert_eq!(style.background, Some(rgb(colors.dock_bg).into()));
                assert_eq!(
                    style.border_color,
                    Some(rgba(colors.settings_border).into())
                );
            }
        }
    }

    #[test]
    fn single_line_settings_inputs_store_height_in_the_rendered_style() {
        for colors in [ThemeColors::light(), ThemeColors::dark()] {
            for height in [32., 36.] {
                for font_size in [12., 14., 20.] {
                    let mut control =
                        settings_input(gpui_kit::div(), px(height), px(font_size), &colors);
                    let style = control.style();
                    assert_eq!(style.size.height, Some(px(height).into()));
                    assert_eq!(style.min_size.width, Some(px(0.).into()));
                    assert_eq!(style.padding.left, Some(px(12.).into()));
                    assert_eq!(style.padding.right, Some(px(12.).into()));
                    assert_eq!(style.padding.top, Some(px(0.).into()));
                    assert_eq!(style.padding.bottom, Some(px(0.).into()));
                    assert_eq!(style.text.font_size, Some(px(font_size).into()));
                    assert_eq!(style.text.line_height, Some(px(20.).into()));
                    assert_eq!(style.background, Some(rgb(colors.dock_bg).into()));
                    assert_eq!(
                        style.border_color,
                        Some(rgba(colors.settings_border).into())
                    );
                }
            }
        }
    }

    #[test]
    fn settings_frame_scopes_the_inherited_interface_font_to_its_own_surface() {
        for colors in [ThemeColors::light(), ThemeColors::dark()] {
            for font_size in [12., 14., 20.] {
                let mut frame = settings_frame(px(font_size), &colors);
                let style = frame.style();
                assert_eq!(style.text.font_size, Some(px(font_size).into()));
                assert_eq!(style.background, Some(rgb(colors.sidebar_bg).into()));
                assert_eq!(style.size.width, Some(relative(1.).into()));
                assert_eq!(style.size.height, Some(relative(1.).into()));
                assert_eq!(style.min_size.width, Some(px(0.).into()));
                assert_eq!(style.min_size.height, Some(px(0.).into()));
            }
        }
    }

    #[test]
    fn settings_captions_use_additive_small_text_without_fixing_their_height() {
        for colors in [ThemeColors::light(), ThemeColors::dark()] {
            for font_size in [12., 14., 20.] {
                let mut caption = settings_caption("Caption", px(font_size), &colors);
                let style = caption.style();
                assert_eq!(style.text.font_size, Some(px(font_size - 2.).into()));
                assert_eq!(
                    style.text.color,
                    Some(rgba(colors.foreground_subtle).into())
                );
                assert!(style.size.height.is_none());
            }
        }
    }

    #[test]
    fn settings_titles_follow_additive_reference_font_tokens_and_allow_wrapping() {
        for language in [Language::Chinese, Language::English] {
            for font_size in [12., 14., 20.] {
                for compact in [false, true] {
                    for section in [
                        Section::General,
                        Section::Appearance,
                        Section::Providers,
                        Section::AgentCapabilities,
                        Section::Subagents,
                        Section::Skills,
                        Section::Shortcuts,
                        Section::Activity,
                    ] {
                        let mut title =
                            settings_title(section.label(language), compact, px(font_size));
                        let style = title.style();
                        assert_eq!(
                            style.text.font_size,
                            Some(px(font_size + if compact { 10. } else { 16. }).into())
                        );
                        assert_eq!(
                            style.text.line_height,
                            Some(relative(if compact { 4. / 3. } else { 1.2 }).into())
                        );
                        assert_eq!(style.text.font_weight, Some(FontWeight::SEMIBOLD));
                        assert_eq!(style.size.width, Some(relative(1.).into()));
                        assert_eq!(style.min_size.width, Some(px(0.).into()));
                        assert!(style.size.height.is_none());
                        assert!(style.text.white_space.is_none());
                    }
                }
            }
        }
    }

    #[test]
    fn settings_group_labels_follow_small_font_and_opaque_muted_color() {
        for colors in [ThemeColors::light(), ThemeColors::dark()] {
            for label in [
                "基础设置",
                "Basics",
                "智能体能力",
                "Agent capabilities",
                "Data & stats",
            ] {
                for font_size in [12., 14., 20.] {
                    let mut label = settings_group_label(label, px(font_size), &colors);
                    let style = label.style();
                    assert_eq!(style.text.font_size, Some(px(font_size - 2.).into()));
                    assert_eq!(style.text.line_height, Some(relative(1.5).into()));
                    assert_eq!(
                        style.text.color,
                        Some(rgb(colors.remote_foreground_muted).into())
                    );
                    assert_eq!(style.text.font_weight, Some(FontWeight::MEDIUM));
                    assert_eq!(style.padding.left, Some(px(10.).into()));
                    assert_eq!(style.padding.right, Some(px(10.).into()));
                    assert_eq!(style.padding.bottom, Some(rems(0.25).into()));
                    assert!(style.size.height.is_none());
                }
            }
        }
    }

    #[test]
    fn interface_font_field_is_112px() {
        assert_eq!(INTERFACE_FONT_FIELD_WIDTH, px(112.));
        assert_eq!(INTERFACE_FONT_FIELD_HEIGHT, px(28.));
        let mut frame = interface_font_frame();
        let style = frame.style();
        assert_eq!(style.position, Some(Position::Relative));
        assert_eq!(style.size.width, Some(px(112.).into()));
        assert_eq!(style.size.height, Some(px(28.).into()));
        assert_eq!(style.flex_shrink, Some(0.));
    }

    #[test]
    fn interface_font_input_reserves_a_fixed_unit_slot_in_both_themes() {
        for colors in [ThemeColors::light(), ThemeColors::dark()] {
            for font_size in [12., 14., 20.] {
                let mut input = interface_font_input(gpui_kit::div(), px(font_size), &colors);
                let style = input.style();
                assert_eq!(style.size.height, Some(px(28.).into()));
                assert_eq!(style.size.width, Some(relative(1.).into()));
                assert_eq!(style.padding.left, Some(px(8.).into()));
                assert_eq!(style.padding.right, Some(px(32.).into()));
                assert_eq!(style.corner_radii.top_left, Some(px(6.).into()));
                assert_eq!(style.text.text_align, Some(TextAlign::Right));
                assert_eq!(style.text.font_size, Some(px(font_size).into()));
                assert_eq!(style.background, Some(rgb(colors.dock_bg).into()));

                let mut unit = interface_font_unit(px(font_size), &colors);
                let style = unit.style();
                assert_eq!(style.position, Some(Position::Absolute));
                assert_eq!(style.inset.right, Some(px(8.).into()));
                assert_eq!(style.inset.top, Some(px(0.).into()));
                assert_eq!(style.size.height, Some(relative(1.).into()));
                assert_eq!(style.align_items, Some(AlignItems::Center));
                assert_eq!(style.text.font_size, Some(px(font_size + 2.).into()));
                assert_eq!(
                    style.text.color,
                    Some(rgba(colors.foreground_subtle).into())
                );
            }
        }
    }

    #[test]
    fn save_action_keeps_compact_height_without_fixing_localized_content_width() {
        for language in [Language::Chinese, Language::English] {
            for font_size in [12., 14., 20.] {
                let mut button = settings_save_button(language.text().save_btn, px(font_size));
                let style = button.style();
                assert_eq!(style.size.height, Some(px(32.).into()));
                assert!(style.size.width.is_none());
                assert_eq!(style.align_self, Some(AlignSelf::Start));
                assert_eq!(style.padding.left, Some(px(10.).into()));
                assert_eq!(style.padding.right, Some(px(10.).into()));
            }
        }
    }

    #[test]
    fn settings_navigation_keeps_wide_rows_and_centered_compact_targets() {
        for language in [Language::Chinese, Language::English] {
            for font_size in [12., 14., 20.] {
                for compact in [false, true] {
                    let label = Section::Providers.label(language);
                    let mut content =
                        settings_nav_content(label, Glyph::Package, compact, px(font_size));
                    let style = content.style();
                    assert_eq!(style.text.font_size, Some(px(font_size).into()));
                    assert_eq!(style.text.line_height, Some(px(20.).into()));
                    assert_eq!(
                        style.justify_content,
                        if compact {
                            Some(JustifyContent::Center)
                        } else {
                            None
                        }
                    );
                    assert_eq!(
                        style.gap.width,
                        if compact { None } else { Some(px(8.).into()) }
                    );
                    let mut button = settings_nav_button("nav-test", label, compact, content);
                    let style = button.style();
                    assert_eq!(
                        style.size.width,
                        Some(if compact {
                            px(40.).into()
                        } else {
                            relative(1.).into()
                        })
                    );
                    assert_eq!(
                        style.size.height,
                        Some(px(if compact { 40. } else { 32. }).into())
                    );
                    let padding = Some(px(if compact { 0. } else { 10. }).into());
                    assert_eq!(style.padding.left, padding);
                    assert_eq!(style.padding.right, padding);
                }
            }
        }
    }

    #[test]
    fn theme_options_keep_light_dark_auto_order() {
        assert_eq!(
            theme_labels(Language::English),
            vec![
                SharedString::from("Light"),
                SharedString::from("Dark"),
                SharedString::from("System"),
            ]
        );
        assert_eq!(theme_index(ThemePreference::Light), 0);
        assert_eq!(theme_index(ThemePreference::Dark), 1);
        assert_eq!(theme_index(ThemePreference::Auto), 2);
    }

    #[test]
    fn settings_navigation_follows_the_reference_group_order() {
        let groups = settings_nav_groups();
        assert_eq!(
            groups[0],
            (
                "basics",
                &[
                    Section::General,
                    Section::Appearance,
                    Section::Providers,
                    Section::Shortcuts,
                ][..]
            )
        );
        assert_eq!(
            groups[1],
            (
                "agent-capabilities",
                &[Section::Subagents, Section::Skills][..]
            )
        );
        assert_eq!(groups[2], ("data-and-stats", &[Section::Activity][..]));
        assert!(groups
            .iter()
            .all(|(_, sections)| !sections.contains(&Section::AgentCapabilities)));
    }

    #[test]
    fn settings_sections_round_trip_and_unknown_values_fall_back() {
        let sections = [
            Section::General,
            Section::Appearance,
            Section::Providers,
            Section::AgentCapabilities,
            Section::Subagents,
            Section::Skills,
            Section::Shortcuts,
            Section::Activity,
        ];
        for section in sections {
            assert_eq!(section_from_key(Some(section.key())), section);
        }
        assert_eq!(section_from_key(None), Section::General);
        assert_eq!(section_from_key(Some("removed-section")), Section::General);
        assert_eq!(
            section_from_key(Some("automations")),
            Section::AgentCapabilities
        );
    }

    #[test]
    fn local_usage_uses_live_tokens_and_does_not_double_count_finished_tools() {
        let mut active = Session::new("active", "Active");
        let mut finished = Message::assistant("run");
        finished.tool_calls = Some(vec![pi_agent_core::ToolCall {
            id: "call-1".into(),
            name: "bash".into(),
            arguments: serde_json::Value::Null,
        }]);
        active.messages.push(finished);
        active
            .messages
            .push(Message::tool_result("call-1", "bash", "ok", false));
        let mut pending = Message::assistant("pending");
        pending.tool_calls = Some(vec![pi_agent_core::ToolCall {
            id: "call-2".into(),
            name: "read".into(),
            arguments: serde_json::Value::Null,
        }]);
        active.messages.push(pending);

        let mut other = Session::new("other", "Other");
        other
            .messages
            .push(Message::tool_result("call-9", "bash", "done", false));

        let mut sessions = HashMap::new();
        sessions.insert(other.id.clone(), other);
        let mut stored = HashMap::new();
        stored.insert("active".to_string(), 10usize);
        stored.insert("other".to_string(), 5usize);

        let usage = local_usage(
            &sessions,
            &active,
            stored.iter().map(|(id, tokens)| (id.as_str(), *tokens)),
            40,
        );
        assert_eq!(usage.tokens, 45);
        assert_eq!(usage.sessions, 2);
        assert_eq!(usage.tool_calls, 3);
        assert_eq!(session_tool_calls(&active.messages), 2);

        let only_active = local_usage(&HashMap::new(), &active, std::iter::empty(), 0);
        assert_eq!(only_active.sessions, 1);
        assert_eq!(only_active.tokens, 0);
        assert_eq!(only_active.tool_calls, 2);
    }
}
