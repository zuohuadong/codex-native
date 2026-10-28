use super::*;
use gpui_kit::assets::IconName as Glyph;
use gpui_kit::base::{Switch as BaseSwitch, SwitchThumb, SwitchTrack};
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::Input;
use gpui_kit::component::select::Select;
use gpui_kit::prelude::FluentBuilder;
use workbench::action;

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

fn provider_field(label: &'static str, label_size: Pixels, control: Input, c: &ThemeColors) -> Div {
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
        .child(
            control
                .w_full()
                .rounded(px(8.))
                .border_1()
                .border_color(rgba(c.settings_border))
                .bg(rgb(c.dock_bg))
                .text_size(label_size)
                .aria_label(label),
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
                group_view = group_view.child(
                    div()
                        .px(px(10.))
                        .pb_1()
                        .text_size(px(12.))
                        .text_color(rgba(c.foreground_subtle))
                        .font_medium()
                        .child(group_label),
                );
            }
            for section in sections {
                let section = *section;
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
                .when(!compact, |button| button.px(px(10.)))
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
            Section::Shortcuts => self.shortcut_settings(c, cx),
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
        let label_size = self.ui_text(14.);
        let base_label = self.t().api_base_label;
        let key_label = self.t().api_key_label;
        let model_label = self.t().model_name_label;
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
                        Input::new(&self.settings_base_input).h(px(32.)),
                        c,
                    ))
                    .child(provider_field(
                        key_label,
                        label_size,
                        Input::new(&self.settings_key_input).h(px(36.)).mask_toggle(),
                        c,
                    ))
                    .child(provider_field(
                        model_label,
                        label_size,
                        Input::new(&self.settings_model_input).h(px(32.)),
                        c,
                    ))
                    .child(
                        div()
                            .text_size(px(12.))
                            .line_height(px(20.))
                            .text_color(rgba(c.foreground_subtle))
                            .child(format!(
                                "{} · {}",
                                mask_api_key(&self.api_key, self.language),
                                self.tr(
                                    "密钥仅用于本次运行；留空保留当前密钥。",
                                    "Key is used for this run only; leave blank to keep the current key."
                                )
                            )),
                    ),
            )
            .into_any_element()
    }

    fn appearance_settings(&self, c: &ThemeColors, cx: &mut Context<Self>) -> AnyElement {
        div()
            .v_flex()
            .gap_3()
            .child(
                div()
                    .v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_size(self.ui_text(16.))
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
                        self.tr("界面字号", "Interface text size"),
                        self.tr(
                            "12–20 px，默认 14。按 Enter 或离开输入框后生效。",
                            "12–20 px. The default is 14. Apply with Enter or by leaving the field.",
                        ),
                        Input::new(&self.settings_font_input)
                            .w(INTERFACE_FONT_FIELD_WIDTH)
                            .h(px(32.))
                            .rounded(px(8.))
                            .border_1()
                            .border_color(rgba(c.settings_border))
                            .bg(rgb(c.dock_bg))
                            .text_size(self.ui_text(14.))
                            .text_align(gpui::TextAlign::Right)
                            .suffix(
                                div()
                                    .text_size(self.ui_text(16.))
                                    .text_color(rgba(c.foreground_subtle))
                                    .child("px"),
                            )
                            .aria_label(self.tr("界面字号", "Interface text size"))
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
                    )),
            )
            .into_any_element()
    }

    fn agent_capabilities_settings(&self, c: &ThemeColors) -> AnyElement {
        card(c)
            .child(setting_row(
                self.tr("工作流", "Workflows"),
                self.tr(
                    "多 agent 阶段、交付物和停止操作都留在当前会话里。",
                    "Multi-agent stages, deliverables, and stop controls stay in the current conversation.",
                ),
                div()
                    .text_size(px(14.))
                    .text_color(rgba(c.foreground_subtle))
                    .child(self.tr("本地运行", "Local"))
                    .into_any_element(),
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
                    .text_size(px(14.))
                    .text_color(rgba(c.foreground_subtle))
                    .child(self.tr("已启用", "Enabled"))
                    .into_any_element(),
                c,
            ))
            .into_any_element()
    }

    fn subagents_settings(&self, c: &ThemeColors) -> AnyElement {
        card(c)
            .child(setting_row(
                self.tr("并行执行", "Parallel execution"),
                self.tr(
                    "子代理在当前工作流中并行检查工作区，并把阶段状态回传到同一会话。",
                    "Subagents inspect the workspace in parallel and return stage status to the same conversation.",
                ),
                div()
                    .text_size(px(14.))
                    .text_color(rgba(c.foreground_subtle))
                    .child(self.tr("只读", "Read-only"))
                    .into_any_element(),
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
                    .text_size(px(14.))
                    .text_color(rgba(c.foreground_subtle))
                    .child("90s")
                    .into_any_element(),
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
                    .text_size(px(14.))
                    .text_color(rgba(c.foreground_subtle))
                    .child(self.tr("按工作流", "Per workflow"))
                    .into_any_element(),
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
                            .text_size(px(14.))
                            .font_medium()
                            .child(self.tr("内置技能", "Built-in skills")),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(rgba(c.foreground_subtle))
                            .child(count.to_string()),
                    ),
            )
            .child(
                div()
                    .text_size(px(14.))
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
                                                    .text_size(px(14.))
                                                    .font_medium()
                                                    .child(name),
                                            )
                                            .child(
                                                div()
                                                    .truncate()
                                                    .text_size(px(12.))
                                                    .text_color(rgba(c.foreground_subtle))
                                                    .child(description),
                                            ),
                                    )
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.open_skill_detail(&detail, window, cx);
                                    })),
                            )
                            .child(
                                action(
                                    format!("settings-use-{}", skill.id),
                                    self.tr("用于任务", "Use in task"),
                                    Glyph::Plus,
                                )
                                .disabled(self.is_executing)
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
                self.ui_text(16.),
                self.ui_text(14.),
                c,
            ))
            .into_any_element()
    }

    fn shortcut_settings(&self, c: &ThemeColors, cx: &mut Context<Self>) -> AnyElement {
        let mono = cx.theme().mono_font_family.clone();
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
            .children(rows.into_iter().enumerate().map(|(index, (label, key))| {
                div()
                    .h_flex()
                    .items_center()
                    .justify_between()
                    .min_w_0()
                    .gap_3()
                    .px_4()
                    .py_3()
                    .when(index > 0, |row| {
                        row.border_t_1().border_color(rgba(c.settings_border))
                    })
                    .child(div().min_w_0().flex_1().text_size(px(14.)).child(label))
                    .child(
                        div()
                            .h_flex()
                            .items_center()
                            .flex_shrink_0()
                            .gap(px(6.))
                            .child(
                                Icon::new(Glyph::Keyboard)
                                    .size(px(16.))
                                    .text_color(rgba(c.foreground_subtle)),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .rounded(px(6.))
                                    .bg(rgba(c.card_user_bg))
                                    .text_size(px(12.))
                                    .font_family(mono.clone())
                                    .child(key),
                            ),
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
        local_usage, section_from_key, session_tool_calls, settings_nav_groups, theme_index,
        theme_labels, Language, Message, Section, Session, SharedString, ThemePreference,
        INTERFACE_FONT_FIELD_WIDTH,
    };
    use gpui_kit::px;
    use std::collections::HashMap;

    #[test]
    fn interface_font_field_is_112px() {
        assert_eq!(INTERFACE_FONT_FIELD_WIDTH, px(112.));
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
