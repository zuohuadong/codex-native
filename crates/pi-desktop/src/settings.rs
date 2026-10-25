use super::*;
use gpui_kit::assets::IconName as Glyph;
use gpui_kit::base::{Switch as BaseSwitch, SwitchThumb, SwitchTrack};
use gpui_kit::component::button::ButtonVariants;
use gpui_kit::component::select::Select;
use gpui_kit::prelude::FluentBuilder;
use workbench::{action, field};

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
            (Self::Activity, Language::Chinese) => "数据与统计",
            (Self::General, _) => "General",
            (Self::Appearance, _) => "Appearance",
            (Self::Providers, _) => "Model providers",
            (Self::AgentCapabilities, _) => "Agent capabilities",
            (Self::Subagents, _) => "Subagents",
            (Self::Skills, _) => "Skills",
            (Self::Shortcuts, _) => "Shortcuts",
            (Self::Activity, _) => "Local activity",
        }
    }

    fn icon(self) -> Glyph {
        match self {
            Self::General => Glyph::Settings,
            Self::Appearance => Glyph::Sun,
            Self::Providers => Glyph::Server,
            Self::AgentCapabilities => Glyph::Package,
            Self::Subagents => Glyph::MessagesSquare,
            Self::Skills => Glyph::WandSparkles,
            Self::Shortcuts => Glyph::Command,
            Self::Activity => Glyph::Terminal,
        }
    }
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

fn card(c: &ThemeColors) -> Div {
    div()
        .v_flex()
        .gap_0()
        .p_0()
        .w_full()
        .rounded(px(12.))
        .border_1()
        .border_color(rgba(c.settings_border))
        .bg(rgb(c.card_bg))
}

fn setting_row(
    label: impl Into<SharedString>,
    description: impl Into<SharedString>,
    control: AnyElement,
    c: &ThemeColors,
) -> Div {
    div()
        .h_flex()
        .items_center()
        .justify_between()
        .gap_4()
        .px_4()
        .py_3()
        .child(
            div()
                .v_flex()
                .gap_1()
                .flex_1()
                .min_w_0()
                .child(div().font_medium().child(label.into()))
                .child(
                    div()
                        .line_height(px(24.))
                        .text_color(rgba(c.muted_foreground))
                        .child(description.into()),
                ),
        )
        .child(
            div()
                .w(px(192.))
                .flex_shrink_0()
                .h_flex()
                .justify_end()
                .child(control),
        )
}

impl PiDesktopApp {
    pub(super) fn settings_view(
        &self,
        window: &Window,
        c: &ThemeColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let compact = window.viewport_size().width < px(1024.);
        let mut navigation = div()
            .v_flex()
            .w(px(if compact { 68. } else { 268. }))
            .h_full()
            .flex_shrink_0()
            .bg(rgb(c.sidebar_bg))
            .child(div().h(px(48.)))
            .child(
                div().px_2().pt_3().pb_3().child(
                    action(
                        "settings-back",
                        if compact {
                            ""
                        } else {
                            self.tr("返回", "Back")
                        },
                        Glyph::ChevronLeft,
                    )
                    .tooltip(self.tr("返回", "Back"))
                    .accessibility_label(self.tr("返回", "Back"))
                    .w(px(if compact { 40. } else { 244. }))
                    .h(px(if compact { 40. } else { 32. }))
                    .m(px(4.))
                    .rounded(px(12.))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.show_settings = false;
                        this.focus_composer(window, cx);
                        cx.notify();
                    })),
                ),
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
        let groups = [
            (
                self.tr("基础设置", "Basics"),
                vec![Section::General, Section::Appearance, Section::Providers],
            ),
            (
                self.tr("智能体能力", "Agent capabilities"),
                vec![Section::Subagents, Section::Skills],
            ),
            (
                self.tr("数据与统计", "Data & stats"),
                vec![Section::Activity, Section::Shortcuts],
            ),
        ];
        let mut index = 0usize;
        for (group_index, (group_label, sections)) in groups.into_iter().enumerate() {
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
                group_view = group_view.child(
                    div()
                        .px(px(10.))
                        .pb_1()
                        .text_size(px(12.))
                        .text_color(rgba(c.foreground_subtle))
                        .font_semibold()
                        .child(group_label),
                );
            }
            for section in sections {
                let item_index = index;
                index += 1;
                let label = section.label(self.language);
                let button = action(
                    format!("settings-section-{item_index}"),
                    if compact { "" } else { label },
                    section.icon(),
                )
                .w_full()
                .h(px(32.))
                .when(compact, |button| button.w(px(40.)).h(px(40.)).mx_auto())
                .rounded(px(12.))
                .tooltip(label)
                .accessibility_label(label)
                .selected(self.settings_section == section)
                .toggled(self.settings_section == section)
                .when(self.settings_section == section, |button| {
                    button.bg(rgba(c.selected))
                })
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.settings_section = section;
                    this.preferences.settings_section = Some(section.key().into());
                    this.settings_error.clear();
                    this.settings_saved = false;
                    this.persist_active_session();
                    cx.notify();
                }));
                group_view = group_view.child(button);
            }
            nav_items = nav_items.child(group_view);
        }
        navigation = navigation.child(nav_items);
        if !compact {
            navigation = navigation.child(self.settings_footer(c));
        }
        let content_inset = if cfg!(target_os = "windows") { 6. } else { 4. };
        let content = match self.settings_section {
            Section::General => self.general_settings(c, cx),
            Section::Appearance => self.appearance_settings(c, cx),
            Section::Providers => self.provider_settings(c),
            Section::Shortcuts => self.shortcut_settings(c),
            Section::AgentCapabilities => self.agent_capabilities_settings(c),
            Section::Subagents => self.subagents_settings(c),
            Section::Skills => self.skills_settings(c, cx),
            Section::Activity => self.activity_settings(c),
        };
        div()
            .h_flex()
            .flex_1()
            .size_full()
            .min_w_0()
            .min_h_0()
            .bg(rgb(c.sidebar_bg))
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
                                            .child(
                                                div()
                                                    .text_size(px(if compact { 24. } else { 30. }))
                                                    .line_height(px(if compact {
                                                        32.
                                                    } else {
                                                        36.
                                                    }))
                                                    .font_semibold()
                                                    .child(
                                                        self.settings_section.label(self.language),
                                                    ),
                                            )
                                            .child(content)
                                            .when(!self.settings_error.is_empty(), |view| {
                                                view.child(
                                                    div()
                                                        .text_color(rgb(0xb42318))
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
                                                        action(
                                                            "save-settings",
                                                            self.t().save_btn,
                                                            Glyph::Check,
                                                        )
                                                        .primary()
                                                        .w(px(132.))
                                                        .h(px(32.))
                                                        .on_click(cx.listener(
                                                            |this, _, window, cx| {
                                                                this.save_preferences(window, cx)
                                                            },
                                                        )),
                                                    )
                                                },
                                            )
                                            .when(self.settings_saved, |view| {
                                                view.child(
                                                    div()
                                                        .text_color(rgb(c.accent_green))
                                                        .child(self.t().save_success),
                                                )
                                            }),
                                    ),
                            ),
                    ),
            )
            .into_any_element()
    }

    fn general_settings(&self, c: &ThemeColors, cx: &mut Context<Self>) -> AnyElement {
        card(c)
            .child(setting_row(
                self.tr("界面语言", "Interface language"),
                self.tr(
                    "选择应用 UI 的显示语言。",
                    "Choose the display language for the app UI.",
                ),
                Select::new(&self.settings_language)
                    .w_full()
                    .h(px(32.))
                    .rounded(px(8.))
                    .border_1()
                    .border_color(rgba(c.settings_border))
                    .bg(rgb(c.dock_bg))
                    .focus_ring(false)
                    .accessibility_label(self.tr("界面语言", "Interface language"))
                    .into_any_element(),
                c,
            ))
            .when(cfg!(target_os = "windows"), |view| {
                view.child(
                    action(
                        "install",
                        self.tr("安装或修复快捷方式", "Install or repair shortcuts"),
                        Glyph::Download,
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.settings_error = match installer::install(true) {
                            Ok(()) => String::new(),
                            Err(error) => error.to_string(),
                        };
                        cx.notify();
                    })),
                )
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
                div()
                    .text_size(px(12.))
                    .font_medium()
                    .text_color(rgba(c.foreground_subtle))
                    .child(self.tr("界面", "Interface")),
            )
            .child(
                div()
                    .h_flex()
                    .gap_2()
                    .child(
                        Select::new(&self.settings_language)
                            .h(px(32.))
                            .flex_1()
                            .rounded(px(8.))
                            .border_1()
                            .border_color(rgba(c.settings_border))
                            .bg(rgb(c.dock_bg))
                            .focus_ring(false)
                            .accessibility_label(self.tr("界面语言", "Interface language")),
                    )
                    .child(
                        Select::new(&self.settings_theme)
                            .h(px(32.))
                            .flex_1()
                            .rounded(px(8.))
                            .border_1()
                            .border_color(rgba(c.settings_border))
                            .bg(rgb(c.dock_bg))
                            .focus_ring(false)
                            .accessibility_label(self.tr("主题", "Theme")),
                    ),
            )
            .into_any_element()
    }

    fn provider_settings(&self, c: &ThemeColors) -> AnyElement {
        card(c)
            .p_4()
            .gap_4()
            .child(
                div()
                    .font_semibold()
                    .child(self.tr("自定义服务", "Custom provider")),
            )
            .child(field(self.t().api_base_label, &self.settings_base_input, c))
            .child(field(
                self.t().model_name_label,
                &self.settings_model_input,
                c,
            ))
            .child(field(self.t().api_key_label, &self.settings_key_input, c))
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(rgba(c.foreground_subtle))
                    .child(format!(
                        "{} · {}",
                        mask_api_key(&self.api_key, self.language),
                        self.tr(
                            "密钥仅用于本次运行；留空保留当前密钥。",
                            "Key is used for this run only; leave blank to keep the current key."
                        )
                    )),
            )
            .into_any_element()
    }

    fn appearance_settings(&self, c: &ThemeColors, cx: &mut Context<Self>) -> AnyElement {
        card(c)
            .child(setting_row(
                self.tr("主题", "Theme"),
                self.tr(
                    "选择明亮、暗黑或跟随系统主题",
                    "Choose light, dark, or system theme",
                ),
                Select::new(&self.settings_theme)
                    .w_full()
                    .h(px(32.))
                    .rounded(px(8.))
                    .border_1()
                    .border_color(rgba(c.settings_border))
                    .bg(rgb(c.dock_bg))
                    .focus_ring(false)
                    .accessibility_label(self.tr("主题", "Theme"))
                    .into_any_element(),
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
                c,
            ))
            .into_any_element()
    }

    fn agent_capabilities_settings(&self, c: &ThemeColors) -> AnyElement {
        card(c)
            .p_4()
            .gap_4()
            .child(
                div()
                    .font_semibold()
                    .child(self.tr("原生智能体运行", "Native agent runtime")),
            )
            .child(
                div()
                    .text_color(rgba(c.muted_foreground))
                    .line_height(px(24.))
                    .child(self.tr(
                        "工作流、多 agent 协作、工具调用和脚本输出均由本地 Rust 运行时驱动。",
                        "Workflows, multi-agent collaboration, tool calls, and script output are driven by the local Rust runtime.",
                    )),
            )
            .child(
                div()
                    .h_flex()
                    .gap_2()
                    .flex_wrap()
                    .children([
                        self.capability_badge("多 agent 工作流", c),
                        self.capability_badge("工具调用状态", c),
                        self.capability_badge("流式输出", c),
                    ]),
            )
            .into_any_element()
    }

    fn capability_badge(&self, label: &str, c: &ThemeColors) -> AnyElement {
        div()
            .px_2()
            .py_1()
            .rounded(px(6.))
            .bg(rgba(c.selected))
            .text_size(px(12.))
            .child(label.to_string())
            .into_any_element()
    }

    fn subagents_settings(&self, c: &ThemeColors) -> AnyElement {
        card(c)
            .p_4()
            .gap_3()
            .child(
                div()
                    .font_semibold()
                    .child(self.tr("子代理", "Subagents")),
            )
            .child(
                div()
                    .text_color(rgba(c.muted_foreground))
                    .line_height(px(24.))
                    .child(self.tr(
                        "子代理在工作流对话中并行执行，并将阶段状态和交付结果回传到当前会话。",
                        "Subagents run in parallel inside workflow conversations and return stage status and deliverables to the current session.",
                    )),
            )
            .child(
                div()
                    .h_flex()
                    .gap_2()
                    .flex_wrap()
                    .children([
                        self.capability_badge("并行阶段", c),
                        self.capability_badge("流式状态", c),
                        self.capability_badge("可中止", c),
                    ]),
            )
            .into_any_element()
    }

    fn skills_settings(&self, c: &ThemeColors, cx: &mut Context<Self>) -> AnyElement {
        card(c)
            .p_4()
            .gap_3()
            .child(
                div()
                    .font_semibold()
                    .child(self.tr("技能与工具", "Skills & tools")),
            )
            .child(
                div()
                    .text_color(rgba(c.muted_foreground))
                    .line_height(px(24.))
                    .child(self.tr(
                        "浏览本地内置技能，并将技能说明插入当前任务。",
                        "Browse built-in local skills and insert their instructions into the current task.",
                    )),
            )
            .child(
                action(
                    "settings-open-skills",
                    self.tr("打开技能", "Open skills"),
                    Glyph::Sparkles,
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    this.show_settings = false;
                    this.selected_nav = "skills";
                    cx.notify();
                })),
            )
            .into_any_element()
    }

    fn activity_settings(&self, c: &ThemeColors) -> AnyElement {
        card(c)
            .p_4()
            .gap_0()
            .child(
                div()
                    .font_semibold()
                    .child(self.tr("本地活动", "Local activity")),
            )
            .child(
                div()
                    .h_flex()
                    .justify_between()
                    .py_3()
                    .child(self.tr("当前会话已用 token", "Tokens used in current session"))
                    .child(
                        div()
                            .text_color(rgba(c.foreground_subtle))
                            .child(self.tokens_used.to_string()),
                    ),
            )
            .child(div().h(px(1.)).bg(rgba(c.settings_border)))
            .child(
                div()
                    .py_3()
                    .text_color(rgba(c.muted_foreground))
                    .line_height(px(24.))
                    .child(self.tr(
                        "统计仅来自本地运行状态；未连接远端账户或云端用量接口。",
                        "These stats come from local runtime state; no remote account or cloud usage API is connected.",
                    )),
            )
            .into_any_element()
    }

    fn shortcut_settings(&self, c: &ThemeColors) -> AnyElement {
        let rows = [
            (self.tr("新任务", "New task"), "⌘/Ctrl + N"),
            (self.tr("显示或隐藏侧栏", "Toggle sidebar"), "⌘/Ctrl + B"),
            (self.tr("设置", "Settings"), "⌘/Ctrl + ,"),
            (self.tr("聚焦输入框", "Focus composer"), "⌘/Ctrl + L"),
            (self.tr("命令输出", "Command output"), "⌘/Ctrl + J"),
            (self.tr("审查更改", "Review changes"), "⌘/Ctrl + Shift + D"),
            (self.tr("返回", "Back"), "Esc"),
        ];
        card(c)
            .p_4()
            .children(rows.into_iter().map(|(label, key)| {
                div()
                    .h_flex()
                    .justify_between()
                    .flex_wrap()
                    .gap_3()
                    .py_2()
                    .child(label)
                    .child(
                        div()
                            .px_2()
                            .py_1()
                            .rounded(px(6.))
                            .bg(rgb(c.sidebar_bg))
                            .text_size(px(12.))
                            .child(key),
                    )
            }))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        section_from_key, theme_index, theme_labels, Language, Section, SharedString,
        ThemePreference,
    };

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
}
