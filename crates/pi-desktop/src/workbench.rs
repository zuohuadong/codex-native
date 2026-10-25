use super::*;
use gpui_kit::assets::IconName as Glyph;
use gpui_kit::component::menu::{DropdownMenu, PopupMenuItem};
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    text::TextView,
};
use gpui_kit::prelude::FluentBuilder;
use interactions::*;

/// Shell geometry from the pinned ZCode sidebar and header contract.
const SIDEBAR_WIDTH: Pixels = px(268.);
const SIDEBAR_HIDE_BREAKPOINT: Pixels = px(820.);
const WORKBENCH_HEADER_HEIGHT: Pixels = px(40.);
const SIDEBAR_NAV_ROW_HEIGHT: Pixels = px(32.);
const SIDEBAR_HISTORY_ROW_HEIGHT: Pixels = px(31.);
const SIDEBAR_NAV_RADIUS: Pixels = px(6.);
const HEADER_ACTION_SIZE: Pixels = px(28.);
const HEADER_COMPACT_ACTION_COUNT: f32 = 5.;
const HEADER_COMPACT_GAP: Pixels = px(8.);
const HEADER_COMPACT_LEFT_CLEARANCE: Pixels = px(76.);

fn sidebar_is_visible(preference: bool, viewport_width: Pixels) -> bool {
    preference && viewport_width >= SIDEBAR_HIDE_BREAKPOINT
}

fn header_uses_compact_navigation(viewport_width: Pixels) -> bool {
    viewport_width < SIDEBAR_HIDE_BREAKPOINT
}

/// Fixed-width cluster for New task / History / Skills / Automations / Settings.
fn compact_header_actions_width() -> Pixels {
    HEADER_ACTION_SIZE * HEADER_COMPACT_ACTION_COUNT
        + HEADER_COMPACT_GAP * (HEADER_COMPACT_ACTION_COUNT - 1.)
}

/// At the supported 480px minimum the compact header still keeps every
/// navigation target plus the trailing Review/Command actions on-screen.
fn compact_header_controls_fit(viewport_width: Pixels) -> bool {
    let trailing = HEADER_ACTION_SIZE * 2. + HEADER_COMPACT_GAP;
    let padding = px(24.);
    let title_floor = px(48.);
    viewport_width
        >= HEADER_COMPACT_LEFT_CLEARANCE
            + compact_header_actions_width()
            + trailing
            + padding
            + title_floor
}

fn chat_content_width(
    viewport_width: Pixels,
    sidebar_preference: bool,
    review_visible: bool,
    narrow_review: bool,
) -> Pixels {
    let mut width = viewport_width;
    if sidebar_is_visible(sidebar_preference, viewport_width) {
        width -= SIDEBAR_WIDTH;
    }
    if review_visible && !narrow_review {
        width -= crate::panels::REVIEW_SIDE_PANE_WIDTH;
    }
    width
}

fn sidebar_action(id: impl Into<ElementId>, label: impl Into<SharedString>, icon: Glyph) -> Button {
    action(id, label, icon)
        .h(SIDEBAR_NAV_ROW_HEIGHT)
        .rounded(SIDEBAR_NAV_RADIUS)
}

pub(super) fn action(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    icon: Glyph,
) -> Button {
    let label = label.into();
    Button::new(id)
        .ghost()
        .small()
        .h(px(28.))
        .rounded(px(8.))
        .justify_start()
        .child(
            div()
                .w_full()
                .h_flex()
                .items_center()
                .gap_2()
                .child(Icon::new(icon).size(px(16.)))
                .child(div().min_w_0().truncate().child(label.clone())),
        )
        .accessibility_label(label)
}
pub(super) fn icon_button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    icon: Glyph,
) -> Button {
    let label = label.into();
    Button::new(id)
        .ghost()
        .small()
        .h(px(28.))
        .w(px(28.))
        .rounded(px(8.))
        .icon(icon)
        .tooltip(label.clone())
        .accessibility_label(label)
}
pub(super) fn heading(title: impl Into<SharedString>) -> Div {
    div().text_size(px(18.)).font_semibold().child(title.into())
}
pub(super) fn field(
    label: impl Into<SharedString>,
    input: &Entity<InputState>,
    c: &ThemeColors,
) -> Div {
    let label = label.into();
    div()
        .v_flex()
        .gap_2()
        .child(
            div()
                .text_size(px(14.))
                .font_semibold()
                .child(label.clone()),
        )
        .child(
            Input::new(input)
                .h(px(32.))
                .rounded(px(8.))
                .border_1()
                .border_color(rgba(c.settings_border))
                .bg(rgb(c.dock_bg))
                .aria_label(label),
        )
}
/// Automation forms use the reference label rhythm: 14px/20px subtle labels,
/// 6px label-to-control spacing and a 36px single-line control.
pub(super) fn automation_field(
    label: impl Into<SharedString>,
    input: &Entity<InputState>,
    c: &ThemeColors,
) -> Div {
    let label = label.into();
    div()
        .v_flex()
        .gap(px(6.))
        .child(
            div()
                .text_size(px(14.))
                .line_height(px(20.))
                .text_color(rgba(c.foreground_subtle))
                .child(label.clone()),
        )
        .child(
            Input::new(input)
                .h(px(36.))
                .rounded(px(12.))
                .border_1()
                .border_color(rgba(c.settings_border))
                .bg(rgb(c.dock_bg))
                .aria_label(label),
        )
}
const AUTOMATION_GRID_BREAKPOINT: Pixels = px(1024.);
/// ZCode uses Tailwind `lg:grid-cols-2` for the automation card list.
pub(super) fn automation_grid_columns(width: Pixels) -> u16 {
    if width >= AUTOMATION_GRID_BREAKPOINT {
        2
    } else {
        1
    }
}
/// Compact card schedule metadata; intentionally avoids duplicating a full timestamp.
pub(super) fn automation_schedule_label(minutes: u32, chinese: bool) -> String {
    if chinese {
        format!("每 {minutes} 分钟")
    } else {
        format!("Every {minutes} min")
    }
}
/// ZCode conversation content column: `calc(100% - 6rem)`, max 4xl at the
/// 864px breakpoint, then `calc(100% - 24rem)`, max 6xl at 1280px.
fn conversation_composer_width(width: Pixels) -> Pixels {
    if width >= px(1280.) {
        (width - px(384.)).clamp(px(0.), px(1152.))
    } else if width >= px(864.) {
        (width - px(96.)).clamp(px(0.), px(896.))
    } else {
        width
    }
}

/// Keep large usage counts from widening the narrow composer toolbar.
fn compact_token_count(tokens: usize) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}m tok", tokens as f64 / 1_000_000.)
    } else if tokens >= 1_000 {
        format!("{:.1}k tok", tokens as f64 / 1_000.)
    } else {
        format!("{tokens} tok")
    }
}

impl Render for PiDesktopApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let dark = match self.theme_preference {
            ThemePreference::Dark => true,
            ThemePreference::Light => false,
            ThemePreference::Auto => matches!(
                window.appearance(),
                WindowAppearance::Dark | WindowAppearance::VibrantDark
            ),
        };
        if Theme::global(cx).is_dark() != dark {
            Theme::change(
                if dark {
                    ThemeMode::Dark
                } else {
                    ThemeMode::Light
                },
                Some(window),
                cx,
            );
        }
        let colors = if dark {
            ThemeColors::dark()
        } else {
            ThemeColors::light()
        };
        sync_component_theme(&colors, cx);
        let viewport_width = window.viewport_size().width;
        let compact = header_uses_compact_navigation(viewport_width);
        let sidebar =
            sidebar_is_visible(self.sidebar_visible, viewport_width) && !self.show_settings;
        let narrow_review = crate::panels::review_uses_single_column(window.viewport_size().width);
        let body = if self.show_settings {
            self.settings_view(window, &colors, cx)
        } else {
            if self.review_visible && narrow_review {
                self.review_view(&colors, cx)
            } else {
                match self.selected_nav {
                    "skills" => self.skills(&colors, cx),
                    "automations" => self.automations_view(window, &colors, cx),
                    "review" => self.review_view(&colors, cx),
                    _ => self.chat(window, &colors, cx),
                }
            }
        };
        div()
            .id("volt-workbench")
            .key_context("Volt")
            .track_focus(&self.root_focus)
            .on_action(
                cx.listener(|this, _: &NewTask, window, cx| this.new_conversation(window, cx)),
            )
            .on_action(cx.listener(|this, _: &ToggleSidebar, _, cx| {
                this.sidebar_visible = !this.sidebar_visible;
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &Settings, _, cx| {
                this.show_settings = !this.show_settings;
                cx.notify();
            }))
            .on_action(
                cx.listener(|this, _: &FocusComposer, window, cx| this.focus_composer(window, cx)),
            )
            .on_action(cx.listener(|this, _: &ToggleReview, _, cx| this.toggle_review(cx)))
            .on_action(cx.listener(|this, _: &ToggleTerminal, _, cx| this.toggle_terminal(cx)))
            .on_action(cx.listener(|this, _: &Dismiss, window, cx| {
                this.review_visible = false;
                this.terminal_visible = false;
                this.automation_form = false;
                this.show_settings = false;
                this.focus_composer(window, cx);
            }))
            .size_full()
            .h_flex()
            .bg(rgb(colors.root_bg))
            .text_color(rgb(colors.text_primary))
            .text_size(px(14.))
            .when(sidebar, |root| root.child(self.sidebar(&colors, cx)))
            .child(
                div()
                    .v_flex()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .when(!self.show_settings, |view| {
                        view.child(self.header(compact, viewport_width, &colors, cx))
                    })
                    .when(!self.storage_error.is_empty(), |root| {
                        root.child(
                            div()
                                .p_2()
                                .text_size(px(12.))
                                .text_color(rgb(0xb42318))
                                .child(self.storage_error.clone()),
                        )
                    })
                    .child(
                        div()
                            .h_flex()
                            .items_stretch()
                            .flex_1()
                            .min_h_0()
                            .min_w_0()
                            .child(body)
                            .when(
                                self.review_visible && !narrow_review && !self.show_settings,
                                |root| {
                                    root.child(
                                        div()
                                            .w(crate::panels::REVIEW_SIDE_PANE_WIDTH)
                                            .flex_shrink_0()
                                            .h_full()
                                            .border_l_1()
                                            .border_color(rgb(colors.card_border))
                                            .child(self.review_view(&colors, cx)),
                                    )
                                },
                            ),
                    )
                    .when(self.terminal_visible && !self.show_settings, |root| {
                        root.child(self.terminal_drawer(
                            &colors,
                            crate::panels::command_drawer_height(window.viewport_size().height),
                            cx,
                        ))
                    }),
            )
    }
}

/// Keep gpui-kit controls (Input, Select, Switch and menus) on the same Zai
/// palette as the application-owned GPUI surfaces. Without this projection,
/// those controls silently retain gpui-kit defaults even when the surrounding
/// page has switched theme.
fn sync_component_theme(colors: &ThemeColors, cx: &mut Context<PiDesktopApp>) {
    let mut palette = Theme::global(cx).colors;
    let dark = Theme::global(cx).is_dark();
    let primary = rgb(if dark { 0xffffff } else { 0x000000 });
    let inverse = rgb(if dark { 0x000000 } else { 0xffffff });
    palette.background = rgb(colors.root_bg).into();
    palette.foreground = rgb(colors.text_primary).into();
    palette.border = rgba(colors.settings_border).into();
    palette.input = rgba(colors.settings_border).into();
    palette.sidebar = rgb(colors.sidebar_bg).into();
    palette.sidebar_foreground = palette.foreground;
    palette.primary = primary.into();
    palette.primary_foreground = inverse.into();
    palette.primary_hover = primary.alpha(0.8).into();
    palette.primary_active = palette.primary_hover;
    palette.button_primary = palette.primary;
    palette.button_primary_foreground = palette.primary_foreground;
    palette.button_primary_hover = palette.primary_hover;
    palette.button_primary_active = palette.primary_active;
    palette.button = rgb(colors.card_bg).into();
    palette.button_foreground = palette.foreground;
    palette.button_hover = rgba(colors.hover).into();
    palette.button_active = rgba(colors.selected).into();
    // Ghost buttons use the secondary interaction tokens; keep hover one step
    // lighter than the persistent selected surface, matching the Zai theme.
    palette.secondary_hover = rgba(colors.hover).into();
    palette.secondary_active = rgba(colors.selected).into();
    palette.button_secondary_hover = palette.secondary_hover;
    palette.button_secondary_active = palette.secondary_active;
    palette.switch = primary.alpha(0.3).into();
    palette.switch_thumb = inverse.into();
    palette.muted_foreground = rgba(colors.foreground_subtle).into();
    palette.popover = rgb(colors.card_bg).into();
    palette.popover_foreground = palette.foreground;
    palette.ring = rgba(colors.dock_border_active).into();
    palette.list = palette.popover;
    // Select/Combobox popup rows use the reference menu-hover surface, not the
    // semi-transparent page selection token.
    palette.list_hover = rgb(colors.menu_hover).into();
    palette.list_active = palette.list_hover;
    if Theme::global(cx).colors == palette && Theme::global(cx).font_size == px(14.) {
        return;
    }
    // update reconciles component tokens and the Base theme, unlike global_mut.
    Theme::update(cx, |theme| {
        theme.colors = palette;
        theme.font_size = px(14.);
    });
}
impl PiDesktopApp {
    fn sidebar(&self, c: &ThemeColors, cx: &mut Context<Self>) -> AnyElement {
        let query = self.search_input.read(cx).value().to_lowercase();
        let skills_active = self.selected_nav == "skills" && !self.show_settings;
        let automations_active = self.selected_nav == "automations" && !self.show_settings;
        div()
            .id("workbench-sidebar")
            .v_flex()
            .w(SIDEBAR_WIDTH)
            .flex_shrink_0()
            .h_full()
            .min_h_0()
            .overflow_hidden()
            .bg(rgb(c.sidebar_bg))
            // The 48px top strip matches the desktop drag/title clearance.
            .child(
                div()
                    .h(px(48.))
                    .pl(px(68.))
                    .pr_2()
                    .flex_shrink_0()
                    .h_flex()
                    .items_center()
                    .justify_between()
                    .child(div().font_semibold().child("Volt"))
                    .child(
                        icon_button(
                            "sidebar-hide",
                            self.tr("收起侧栏", "Hide sidebar"),
                            Glyph::PanelLeft,
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.sidebar_visible = false;
                            cx.notify();
                        })),
                    ),
            )
            // Primary navigation: reference px-2 / py-3, 4px rhythm, 32px rows.
            .child(
                div()
                    .v_flex()
                    .flex_shrink_0()
                    .gap_1()
                    .px_2()
                    .py_3()
                    .child(
                        sidebar_action("new-task", self.t().new_conversation, Glyph::SquarePen)
                            .w_full()
                            .disabled(self.is_executing)
                            .on_click(
                                cx.listener(|this, _, window, cx| {
                                    this.new_conversation(window, cx)
                                }),
                            ),
                    )
                    .child(
                        sidebar_action("skills-nav", self.t().skills_tools, Glyph::Sparkles)
                            .w_full()
                            .selected(skills_active)
                            .toggled(skills_active)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.selected_nav = "skills";
                                this.show_settings = false;
                                cx.notify();
                            })),
                    )
                    .child(
                        sidebar_action("automation-nav", self.t().automations, Glyph::Clock)
                            .w_full()
                            .selected(automations_active)
                            .toggled(automations_active)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.selected_nav = "automations";
                                this.show_settings = false;
                                cx.notify();
                            })),
                    ),
            )
            // The list region is separated from navigation and owns the only
            // vertical scroll, matching the reference sidebar structure.
            .child(
                div()
                    .v_flex()
                    .flex_1()
                    .min_h_0()
                    .border_t_1()
                    .border_color(rgba(c.settings_border))
                    .pt_3()
                    .child(
                        div()
                            .h_flex()
                            .flex_shrink_0()
                            .items_center()
                            .gap_2()
                            .px_2()
                            .pb_2()
                            .child(
                                div()
                                    .min_w_0()
                                    .flex_1()
                                    .truncate()
                                    .px(px(10.))
                                    .text_size(px(14.))
                                    .font_medium()
                                    .text_color(rgba(c.foreground_subtle))
                                    .child(self.t().projects_header),
                            ),
                    )
                    .child(
                        div()
                            .h(px(32.))
                            .flex_shrink_0()
                            .h_flex()
                            .items_center()
                            .gap_2()
                            .px(px(12.))
                            .text_size(px(14.))
                            .child(Icon::new(Glyph::Folder).size(px(16.)))
                            .child(div().min_w_0().truncate().child(self.workspace_name())),
                    )
                    .child(
                        div()
                            .px_2()
                            .pb_2()
                            .flex_shrink_0()
                            .child(Input::new(&self.search_input).small()),
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .px(px(12.))
                            .pt_1()
                            .pb_1()
                            .text_size(px(12.))
                            .text_color(rgba(c.foreground_subtle))
                            .child(self.t().conversation_history),
                    )
                    .child(
                        div()
                            .id("history-list")
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .v_flex()
                            .gap_1()
                            .px_2()
                            .pb_2()
                            .children(
                                self.conversations
                                    .iter()
                                    .filter(|entry| entry.title.to_lowercase().contains(&query))
                                    .map(|entry| {
                                        let id = entry.id.clone();
                                        action(
                                            format!("history-{}", entry.id),
                                            entry.title.clone(),
                                            Glyph::MessageSquare,
                                        )
                                        .w_full()
                                        .h(SIDEBAR_HISTORY_ROW_HEIGHT)
                                        .rounded(SIDEBAR_NAV_RADIUS)
                                        .selected(entry.is_active)
                                        .toggled(entry.is_active)
                                        .disabled(self.is_executing)
                                        .on_click(
                                            cx.listener(move |this, _, window, cx| {
                                                this.activate_conversation(&id, window, cx)
                                            }),
                                        )
                                    }),
                            ),
                    ),
            )
            // Keep only the existing Settings entry in the footer; no remote,
            // upgrade, account or avatar surface is introduced here.
            .child(
                div()
                    .flex_shrink_0()
                    .border_t_1()
                    .border_color(rgba(c.settings_border))
                    .px_4()
                    .pt_2()
                    .pb_4()
                    .child(
                        sidebar_action("settings-nav", self.t().settings, Glyph::Settings)
                            .w_full()
                            .selected(self.show_settings)
                            .toggled(self.show_settings)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.show_settings = !this.show_settings;
                                cx.notify();
                            })),
                    ),
            )
            .into_any_element()
    }
    fn workspace_name(&self) -> String {
        self.workspace_dir
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.workspace_dir.display().to_string())
    }
    fn header(
        &self,
        compact: bool,
        viewport_width: Pixels,
        c: &ThemeColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let title = if self.show_settings {
            self.t().settings.into()
        } else {
            match self.selected_nav {
                "skills" => self.t().skills_tools.into(),
                "automations" => self.t().automations.into(),
                _ => self.session.title.clone(),
            }
        };
        div()
            .h(WORKBENCH_HEADER_HEIGHT)
            .flex_shrink_0()
            .h_flex()
            .items_center()
            .gap_2()
            .px_3()
            .when(compact || !self.sidebar_visible, |row| row.pl(px(76.)))
            .when(!compact && !self.sidebar_visible, |row| {
                row.child(
                    icon_button(
                        "show-sidebar",
                        self.tr("展开侧栏", "Show sidebar"),
                        Glyph::PanelLeft,
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.sidebar_visible = true;
                        cx.notify();
                    })),
                )
            })
            .when(compact, |row| {
                row.children([
                    icon_button("compact-new", self.t().new_conversation, Glyph::SquarePen)
                        .disabled(self.is_executing)
                        .on_click(
                            cx.listener(|this, _, window, cx| this.new_conversation(window, cx)),
                        ),
                    icon_button(
                        "compact-history",
                        self.t().conversation_history,
                        Glyph::MessagesSquare,
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.selected_nav = "history";
                        this.show_settings = false;
                        cx.notify();
                    })),
                    icon_button("compact-skills", self.t().skills_tools, Glyph::Sparkles).on_click(
                        cx.listener(|this, _, _, cx| {
                            this.selected_nav = "skills";
                            this.show_settings = false;
                            cx.notify();
                        }),
                    ),
                    icon_button("compact-auto", self.t().automations, Glyph::Clock).on_click(
                        cx.listener(|this, _, _, cx| {
                            this.selected_nav = "automations";
                            this.show_settings = false;
                            cx.notify();
                        }),
                    ),
                    icon_button("compact-settings", self.t().settings, Glyph::Settings).on_click(
                        cx.listener(|this, _, _, cx| {
                            this.show_settings = !this.show_settings;
                            cx.notify();
                        }),
                    ),
                ])
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_size(px(12.))
                    .child(title),
            )
            .child(
                icon_button(
                    "toggle-review",
                    self.tr("审查变更", "Review changes"),
                    Glyph::GitPullRequest,
                )
                .selected(self.review_visible)
                .on_click(cx.listener(|this, _, _, cx| this.toggle_review(cx))),
            )
            .child(
                icon_button(
                    "toggle-terminal",
                    self.tr("命令输出", "Command output"),
                    Glyph::Terminal,
                )
                .selected(self.terminal_visible)
                .on_click(cx.listener(|this, _, _, cx| this.toggle_terminal(cx))),
            )
            .when(
                self.is_executing && compact_header_controls_fit(viewport_width),
                |row| {
                    row.child(
                        div()
                            .flex_shrink_0()
                            .text_size(px(12.))
                            .text_color(rgba(c.foreground_subtle))
                            .child(self.tr("运行中", "Running")),
                    )
                },
            )
            .into_any_element()
    }
    fn chat(&self, window: &Window, c: &ThemeColors, cx: &mut Context<Self>) -> AnyElement {
        let content_width = chat_content_width(
            window.viewport_size().width,
            self.sidebar_visible,
            self.review_visible,
            crate::panels::review_uses_single_column(window.viewport_size().width),
        );

        if self.selected_nav == "history" {
            return div()
                .id("compact-history-list")
                .v_flex()
                .flex_1()
                .overflow_y_scroll()
                .p_4()
                .gap_2()
                .child(heading(self.t().conversation_history))
                .children(self.conversations.iter().map(|entry| {
                    let id = entry.id.clone();
                    action(
                        format!("compact-task-{}", id),
                        entry.title.clone(),
                        Glyph::MessageSquare,
                    )
                    .disabled(self.is_executing)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.activate_conversation(&id, window, cx)
                    }))
                }))
                .into_any_element();
        }
        let empty = !self
            .session
            .messages
            .iter()
            .any(|message| message.role != pi_agent_core::Role::System);
        if empty && self.run_error.is_none() && !self.is_executing {
            let greeting: SharedString = self
                .tr(
                    "今天有什么工作，交给我吧",
                    "What's on your plate today? Leave it to me.",
                )
                .into();
            let mut font = window.text_style().font();
            font.weight = gpui::FontWeight::MEDIUM;
            let run = gpui::TextRun {
                len: greeting.len(),
                font,
                color: rgb(c.text_primary).into(),
                background_color: None,
                underline: None,
                strikethrough: None,
            };
            let natural_width = window
                .text_system()
                .shape_line(greeting.clone(), px(30.), &[run], None)
                .width;
            let available_width = (content_width.min(px(672.)) - px(32.)).max(px(0.));
            let greeting_size = if natural_width > px(0.) && available_width > px(0.) {
                (30. * (available_width / natural_width))
                    .floor()
                    .clamp(20., 30.)
            } else {
                30.
            };
            return div()
                .id("empty-conversation")
                .flex_1()
                .min_w_0()
                .min_h_0()
                .overflow_y_scroll()
                .child(
                    div()
                        .v_flex()
                        .min_h_full()
                        .items_center()
                        // ZCode's draft timeline uses a viewport-based leading spacer,
                        // then lets the trailing space absorb the remaining height.
                        .child(
                            div()
                                .w_full()
                                .min_h(px(52.))
                                .flex_basis(window.viewport_size().height * 0.29),
                        )
                        .child(
                            div()
                                .w_full()
                                .max_w(px(672.))
                                .px_4()
                                .flex_shrink_0()
                                .mb(px(if window.viewport_size().width < px(640.) {
                                    40.
                                } else {
                                    32.
                                }))
                                .text_center()
                                .text_size(px(greeting_size))
                                .line_height(px(greeting_size * 1.2))
                                .font_medium()
                                .text_color(rgb(c.text_primary))
                                .child(greeting),
                        )
                        .child(self.queue_view(c, cx))
                        .child(div().w_full().flex_shrink_0().child(self.composer(
                            content_width,
                            c,
                            cx,
                        )))
                        .child(div().w_full().min_h(px(16.)).flex_1()),
                )
                .into_any_element();
        }
        if self.scroll_pinned {
            self.scroll.scroll_to_bottom();
        }
        div()
            .v_flex()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .child(
                div()
                    .id("transcript")
                    .track_focus(&self.transcript_focus)
                    .w_full()
                    .min_w_0()
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                        let modifiers = event.keystroke.modifiers;
                        if !this.transcript_focus.is_focused(window)
                            || modifiers.control
                            || modifiers.platform
                            || modifiers.alt
                        {
                            return;
                        }
                        let offset = this.scroll.offset();
                        if let Some((top, pinned)) = crate::transcript_scroll::keyboard_target(
                            &event.keystroke.key,
                            modifiers.shift,
                            -offset.y / px(1.),
                            this.scroll.max_offset().y / px(1.),
                            this.scroll.bounds().size.height / px(1.),
                        ) {
                            this.scroll_pinned = pinned;
                            this.scroll.set_offset(point(offset.x, px(-top)));
                            if pinned {
                                this.scroll.scroll_to_bottom();
                            }
                            cx.stop_propagation();
                            cx.notify();
                        }
                    }))
                    .on_scroll_wheel(cx.listener(
                        |this, event: &gpui::ScrollWheelEvent, window, cx| {
                            let delta_y = event.delta.pixel_delta(window.line_height()).y;
                            let current = this.scroll.offset();
                            let max = this.scroll.max_offset();
                            let distance_to_bottom = (max.y + current.y).max(px(0.));
                            this.scroll_pinned = crate::transcript_scroll::after_wheel(
                                this.scroll_pinned,
                                delta_y / px(1.),
                                distance_to_bottom / px(1.),
                                max.y > px(0.),
                            );
                            cx.notify();
                        },
                    ))
                    .v_flex()
                    .items_center()
                    .px_4()
                    .py_6()
                    .gap_6()
                    .children(
                        self.session
                            .messages
                            .iter()
                            .filter(|m| m.role != pi_agent_core::Role::System)
                            .map(|message| {
                                let mut row = div()
                                    .w_full()
                                    .max_w(px(1104.))
                                    .min_w_0()
                                    .v_flex()
                                    .items_center()
                                    .gap_3()
                                    .child(self.message(message, content_width, c, window, cx));
                                if let Some(state) = self.task_states.get(&self.session.id) {
                                    for workflow in state
                                        .workflows
                                        .iter()
                                        .filter(|w| w.parent_message_id == message.id)
                                    {
                                        row = row.child(self.workflow_card(workflow, c, cx));
                                    }
                                }
                                row
                            }),
                    ),
            )
            .when(self.run_error.is_some(), |root| {
                let stopped = self.run_stopped;
                root.child(
                    div()
                        .px_4()
                        .py_2()
                        .v_flex()
                        .gap_2()
                        .child(
                            div()
                                .text_color(if stopped {
                                    rgba(c.foreground_subtle)
                                } else {
                                    rgb(0xb42318)
                                })
                                .child(self.run_error.clone().unwrap_or_default()),
                        )
                        .child(
                            action("retry-run", self.tr("重试", "Retry"), Glyph::RefreshCw)
                                .disabled(self.is_executing)
                                .on_click(cx.listener(|this, _, _, cx| this.retry_run(cx))),
                        ),
                )
            })
            .child(self.queue_view(c, cx))
            .child(
                div()
                    .relative()
                    .w_full()
                    .child(self.composer(content_width, c, cx))
                    .when(
                        !self.scroll_pinned && self.scroll.max_offset().y > px(0.),
                        |dock| {
                            dock.child(
                                Button::new("transcript-bottom")
                                    .outline()
                                    .absolute()
                                    .top(px(-40.))
                                    .left(relative(0.5))
                                    .ml(px(-16.))
                                    .w(px(32.))
                                    .h(px(32.))
                                    .rounded_full()
                                    .shadow_sm()
                                    .bg(rgb(c.card_bg))
                                    .accessibility_label(self.tr("回到底部", "Scroll to bottom"))
                                    .tooltip(self.tr("回到底部", "Scroll to bottom"))
                                    .child(Icon::new(Glyph::ArrowDown).size(px(16.)))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.scroll_pinned = true;
                                        this.scroll.scroll_to_bottom();
                                        cx.notify();
                                    })),
                            )
                        },
                    ),
            )
            .into_any_element()
    }
    fn message(
        &self,
        message: &Message,
        content_width: Pixels,
        c: &ThemeColors,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let user = message.role == pi_agent_core::Role::User;
        let mut view = div().w_full().max_w(px(760.)).min_w_0().v_flex().gap_2();
        if user {
            let bubble_width = user_message_bubble_width(content_width);
            // Measure wrapped lines with the same font, size and available width as
            // the selectable text; character counts miss newlines and CJK wrapping.
            let text: SharedString = message.content.clone().into();
            let run = gpui::TextRun {
                len: text.len(),
                font: window.text_style().font(),
                color: rgb(c.text_primary).into(),
                background_color: None,
                underline: None,
                strikethrough: None,
            };
            let text_height = window
                .text_system()
                .shape_text(
                    text.clone(),
                    px(14.),
                    &[run],
                    Some((bubble_width - px(34.)).max(px(1.))),
                    None,
                )
                .ok()
                .map(|lines| {
                    lines
                        .iter()
                        .map(|line| line.size(px(24.)).height)
                        .sum::<Pixels>()
                });
            let expandable = text_height.is_some_and(user_message_content_is_expandable);
            let bubble_surface = rgb(c.root_bg).blend(rgba(c.card_user_bg));
            let expand_key =
                user_message_disclosure_key(&self.session.id, &message.id, &message.content);
            let expanded = self.expanded_tools.contains(&expand_key);
            let toggle_label = if expanded {
                self.tr("收起消息", "Collapse message")
            } else {
                self.tr("展开消息", "Expand message")
            };
            return view
                .items_end()
                .child(
                    div()
                        .min_w_0()
                        .max_w(bubble_width)
                        .px_4()
                        .py_3()
                        .rounded(px(12.))
                        .rounded_tr(px(2.))
                        .border_1()
                        .border_color(rgba(c.settings_border))
                        .bg(rgba(c.card_user_bg))
                        .child(
                            div()
                                .relative()
                                .min_w_0()
                                .v_flex()
                                .child(
                                    div()
                                        .min_w_0()
                                        .text_size(px(14.))
                                        .line_height(px(24.))
                                        .when(expandable && !expanded, |body| {
                                            body.max_h(px(120.)).overflow_hidden()
                                        })
                                        .child(gpui_kit::base::SelectableText::new(
                                            format!("user-{}", message.id),
                                            text,
                                        )),
                                )
                                .when(expandable, |body| {
                                    body.child(
                                        div()
                                            .w_full()
                                            .h_flex()
                                            .justify_center()
                                            .when(!expanded, |toggle| {
                                                toggle.absolute().bottom_0().left_0().child(
                                                    div()
                                                        .absolute()
                                                        .bottom_0()
                                                        .left_0()
                                                        .right_0()
                                                        .h(px(36.))
                                                        .bg(gpui::linear_gradient(
                                                            180.,
                                                            gpui::linear_color_stop(
                                                                bubble_surface.alpha(0.),
                                                                0.,
                                                            ),
                                                            gpui::linear_color_stop(
                                                                bubble_surface,
                                                                1.,
                                                            ),
                                                        )),
                                                )
                                            })
                                            .when(expanded, |toggle| toggle.pt_2())
                                            .child(
                                                Button::new(expand_key.clone())
                                                    .outline()
                                                    .relative()
                                                    .shadow_sm()
                                                    .small()
                                                    .rounded_full()
                                                    .w(px(32.))
                                                    .h(px(24.))
                                                    .bg(rgb(c.root_bg))
                                                    .tooltip(toggle_label)
                                                    .accessibility_label(toggle_label)
                                                    .child(
                                                        Icon::new(if expanded {
                                                            Glyph::ChevronUp
                                                        } else {
                                                            Glyph::ChevronDown
                                                        })
                                                        .size(px(16.)),
                                                    )
                                                    .on_click(cx.listener(
                                                        move |this, _, _, cx| {
                                                            this.toggle_tool_expansion(&expand_key);
                                                            cx.notify();
                                                        },
                                                    )),
                                            ),
                                    )
                                }),
                        ),
                )
                .into_any_element();
        }
        if let Some(tool) = &message.tool_result {
            let id = tool.tool_call_id.clone();
            let expanded = self.expanded_tools.contains(&id);
            let label = format!(
                "{} · {}",
                tool.name,
                if tool.is_error {
                    self.tr("失败", "Failed")
                } else {
                    self.tr("完成", "Completed")
                }
            );
            return view
                .child(
                    action(
                        format!("tool-{}", id),
                        label,
                        if tool.is_error {
                            Glyph::CircleAlert
                        } else {
                            Glyph::Check
                        },
                    )
                    .selected(expanded)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.toggle_tool_expansion(&id);
                        cx.notify();
                    })),
                )
                .when(expanded, |view| {
                    let (output_font_size, output_max_height) =
                        raw_tool_output_presentation(&tool.name);
                    let scroll_id = format!("tool-output-scroll-{}", message.id);
                    let scroll = self
                        .tool_output_scroll
                        .borrow_mut()
                        .entry(scroll_id.clone())
                        .or_default()
                        .clone();
                    let wheel_scroll = scroll.clone();
                    view.child(
                        div()
                            .min_w_0()
                            .p_3()
                            .rounded(px(12.))
                            .bg(rgba(c.card_user_bg))
                            .child(
                                div()
                                    .id(scroll_id)
                                    .min_w_0()
                                    .max_h(output_max_height)
                                    .overflow_y_scroll()
                                    .track_scroll(&scroll)
                                    .on_scroll_wheel(
                                        move |event: &gpui::ScrollWheelEvent, window, cx| {
                                            let delta_y =
                                                event.delta.pixel_delta(window.line_height()).y
                                                    / px(1.);
                                            let offset = -wheel_scroll.offset().y / px(1.);
                                            let maximum = wheel_scroll.max_offset().y / px(1.);
                                            if crate::transcript_scroll::nested_wheel_captures(
                                                delta_y, offset, maximum,
                                            ) {
                                                cx.stop_propagation();
                                            }
                                        },
                                    )
                                    .text_size(output_font_size)
                                    .line_height(px(20.))
                                    .font_family(cx.theme().mono_font_family.clone())
                                    .text_color(rgba(c.foreground_subtle))
                                    .child(gpui_kit::base::SelectableText::new(
                                        format!("tool-output-{}", message.id),
                                        tool.output.clone(),
                                    )),
                            ),
                    )
                })
                .into_any_element();
        }
        if let Some(reasoning) = &message.reasoning {
            let id = format!("reasoning-{}", message.id);
            let expanded = self.expanded_tools.contains(&id);
            view = view
                .child(
                    action(
                        id.clone(),
                        self.tr("思考过程", "Reasoning"),
                        Glyph::ChevronRight,
                    )
                    .selected(expanded)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.toggle_tool_expansion(&id);
                        cx.notify();
                    })),
                )
                .when(expanded, |view| {
                    view.child(
                        TextView::markdown(
                            format!("reasoning-text-{}", message.id),
                            reasoning.clone(),
                        )
                        .selectable(true),
                    )
                });
        }
        if let Some(calls) = &message.tool_calls {
            for call in calls {
                let finished = self.session.messages.iter().any(|m| {
                    m.tool_result
                        .as_ref()
                        .is_some_and(|r| r.tool_call_id == call.id)
                });
                if !finished {
                    view = view.child(
                        div()
                            .text_size(px(12.))
                            .text_color(rgba(c.foreground_subtle))
                            .child(format!(
                                "{} · {}",
                                call.name,
                                if self.is_executing {
                                    self.tr("运行中", "Running")
                                } else {
                                    self.tr("未完成", "Incomplete")
                                }
                            )),
                    );
                }
            }
        }
        let last_message_id = self
            .session
            .messages
            .iter()
            .rev()
            .find(|message| message.role != pi_agent_core::Role::System)
            .map(|message| message.id.as_str());
        let streaming = assistant_message_is_streaming(
            self.is_executing,
            last_message_id == Some(message.id.as_str()),
            &message.role,
        );
        view.when(!message.content.is_empty(), |view| {
            view.child(
                TextView::markdown(format!("assistant-{}", message.id), message.content.clone())
                    .selectable(true)
                    .stream_fade(!self.preferences.reduced_motion && streaming),
            )
        })
        .into_any_element()
    }
    fn composer(
        &self,
        content_width: Pixels,
        c: &ThemeColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let empty = !self
            .session
            .messages
            .iter()
            .any(|message| message.role != pi_agent_core::Role::System);
        // ZCode: draft composer is max-w-2xl; an existing conversation uses the
        // same responsive content column as the timeline (896px wide, then 1152px
        // once the 1280px status-panel breakpoint applies).
        let target_width = if empty {
            content_width.min(px(672.))
        } else {
            conversation_composer_width(content_width)
        };
        let compact = content_width < px(680.);
        let very_compact = content_width < px(540.);
        let model_max_width = if very_compact {
            px(116.)
        } else if compact {
            px(160.)
        } else {
            px(240.)
        };
        let token_label = if compact {
            compact_token_count(self.tokens_used)
        } else {
            format!("{} tokens", self.tokens_used)
        };
        div()
            .w_full()
            .px_4()
            .when(!empty, |dock| dock.pb_4())
            .v_flex()
            .items_center()
            .child(
                div()
                    .w_full()
                    .max_w(target_width)
                    .v_flex()
                    .rounded(px(16.))
                    .when(empty, |card| {
                        card.bg(rgba(c.selected)).shadow_xs().child(
                            div().p(px(6.)).h_flex().items_center().child(
                                action("composer-workspace", self.workspace_name(), Glyph::Folder)
                                    .tooltip(self.workspace_dir.display().to_string())
                                    .on_click(cx.listener(|this, _, _, cx| this.toggle_review(cx))),
                            ),
                        )
                    })
                    .child(
                        div()
                            .id("composer-shell")
                            .track_focus(&self.input_state.read(cx).focus_handle(cx))
                            .w_full()
                            .min_w_0()
                            .v_flex()
                            .gap_3()
                            .p_3()
                            .rounded(px(16.))
                            .bg(rgb(c.dock_bg))
                            .border_1()
                            .border_color(rgba(c.dock_border))
                            .hover(|style| style.border_color(rgba(c.dock_border_active)))
                            .focus(|style| {
                                style
                                    .border_color(rgba(c.dock_border_active))
                                    .bg(rgb(c.dock_bg))
                            })
                            .child(
                                Textarea::new(&self.input_state)
                                    .appearance(false)
                                    .p_0()
                                    .min_h(px(40.))
                                    .max_h(px(160.))
                                    .text_size(px(14.))
                                    .line_height(px(20.)),
                            )
                            .child(
                                div()
                                    .h_flex()
                                    .items_center()
                                    .gap_3()
                                    .child(
                                        div()
                                            .h_flex()
                                            .flex_1()
                                            .min_w_0()
                                            .items_center()
                                            .child(
                                                icon_button(
                                                    "composer-actions",
                                                    self.tr("更多操作", "More actions"),
                                                    Glyph::Plus,
                                                )
                                                .dropdown_menu_with_anchor(Anchor::BottomLeft, {
                                                    let app = cx.entity().downgrade();
                                                    let skills_label = self.tr("技能", "Skills");
                                                    let files_label =
                                                        self.tr("变更文件", "Changed files");
                                                    move |menu, _, _| {
                                                        let skills_app = app.clone();
                                                        let files_app = app.clone();
                                                        menu.item(
                                                            PopupMenuItem::new(skills_label).on_click(
                                                                move |_, _, cx| {
                                                                    let _ = skills_app.update(
                                                                        cx,
                                                                        |this, cx| {
                                                                            this.selected_nav =
                                                                                "skills";
                                                                            cx.notify();
                                                                        },
                                                                    );
                                                                },
                                                            ),
                                                        )
                                                        .item(
                                                            PopupMenuItem::new(files_label).on_click(
                                                                move |_, _, cx| {
                                                                    let _ = files_app.update(
                                                                        cx,
                                                                        |this, cx| {
                                                                            this.toggle_review(cx)
                                                                        },
                                                                    );
                                                                },
                                                            ),
                                                        )
                                                    }
                                                }),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .h_flex()
                                            .min_w_0()
                                            .items_center()
                                            .justify_end()
                                            .gap(px(6.))
                                            .child(
                                                Button::new("model-picker")
                                                    .ghost()
                                                    .small()
                                                    .h(px(28.))
                                                    .rounded(px(8.))
                                                    .pl(px(8.))
                                                    .pr(px(6.))
                                                    .accessibility_label(
                                                        self.tr("选择模型", "Select model"),
                                                    )
                                                    .child(
                                                        div()
                                                            .h_flex()
                                                            .min_w_0()
                                                            .gap_1()
                                                            .items_center()
                                                            .child(
                                                                Icon::new(Glyph::Package)
                                                                    .size(px(16.))
                                                                    .flex_shrink_0(),
                                                            )
                                                            .child(
                                                                div()
                                                                    .min_w_0()
                                                                    .truncate()
                                                                    .child(self.current_model()),
                                                            )
                                                            .child(
                                                                Icon::new(Glyph::ChevronDown)
                                                                    .size(px(14.))
                                                                    .flex_shrink_0()
                                                                    .text_color(rgba(
                                                                        c.foreground_subtle,
                                                                    )),
                                                            ),
                                                    )
                                                    .max_w(model_max_width)
                                                    .min_w_0()
                                                    .disabled(self.is_executing)
                                                    .dropdown_menu_with_anchor(
                                                        Anchor::BottomCenter,
                                                        {
                                                            let app = cx.entity().downgrade();
                                                            move |mut menu, _, cx| {
                                                                let selected = app
                                                                    .upgrade()
                                                                    .map(|app| {
                                                                        app.read(cx)
                                                                            .model_identifier
                                                                            .clone()
                                                                    });
                                                                for (name, id) in MODELS {
                                                                    let app = app.clone();
                                                                    let model = id.to_string();
                                                                    menu = menu.item(
                                                                        PopupMenuItem::new(*name)
                                                                            .checked(
                                                                                selected.as_deref()
                                                                                    == Some(*id),
                                                                            )
                                                                            .on_click(
                                                                                move |_,
                                                                                      window,
                                                                                      cx| {
                                                                                    let _ = app.update(
                                                                                        cx,
                                                                                        |this, cx| {
                                                                                            if this.is_executing { return; }
                                                                                            this.model_identifier = model.clone();
                                                                                            this.preferences.model = model.clone();
                                                                                            this.persist_active_session();
                                                                                            this.settings_saved = false;
                                                                                            this.settings_model_input.update(cx, |input, cx| {
                                                                                                input.set_value(model.clone(), window, cx);
                                                                                            });
                                                                                            cx.notify();
                                                                                        },
                                                                                    );
                                                                                },
                                                                            ),
                                                                    );
                                                                }
                                                                menu
                                                            }
                                                        },
                                                    ),
                                            )
                                            .when(self.tokens_used > 0, |row| {
                                                row.child(
                                                    div()
                                                        .flex_shrink_0()
                                                        .text_size(px(12.))
                                                        .text_color(rgba(c.foreground_subtle))
                                                        .child(token_label),
                                                )
                                            })
                                            .when(self.is_executing, |row| {
                                                row.child(
                                                    icon_button(
                                                        "enqueue",
                                                        self.tr("加入队列", "Queue message"),
                                                        Glyph::Plus,
                                                    )
                                                    .disabled(
                                                        self.input_state
                                                            .read(cx)
                                                            .value()
                                                            .trim()
                                                            .is_empty(),
                                                    )
                                                    .on_click(cx.listener(
                                                        |this, _, window, cx| {
                                                            this.trigger_send(window, cx)
                                                        },
                                                    )),
                                                )
                                            })
                                            .child(if self.is_executing {
                                                icon_button(
                                                    "stop",
                                                    self.tr("停止", "Stop"),
                                                    Glyph::Square,
                                                )
                                                .secondary()
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.stop_run(cx)
                                                }))
                                            } else {
                                                icon_button(
                                                    "send",
                                                    self.t().send_btn,
                                                    Glyph::ArrowUp,
                                                )
                                                .primary()
                                                .disabled(
                                                    self.input_state
                                                        .read(cx)
                                                        .value()
                                                        .trim()
                                                        .is_empty(),
                                                )
                                                .on_click(cx.listener(
                                                    |this, _, window, cx| {
                                                        this.trigger_send(window, cx)
                                                    },
                                                ))
                                            }),
                                    ),
                            ),
                    ),
            )
            .into_any_element()
    }
    fn skills(&self, c: &ThemeColors, cx: &mut Context<Self>) -> AnyElement {
        let query = self
            .skill_search_input
            .read(cx)
            .value()
            .trim()
            .to_lowercase();
        let skills: Vec<_> = self
            .skill_registry
            .all()
            .iter()
            .filter(|skill| {
                self.skill_category
                    .is_none_or(|category| skill.category == category)
            })
            .filter(|skill| {
                format!(
                    "{} {} {} {}",
                    skill.name, skill.name_en, skill.description, skill.description_en
                )
                .to_lowercase()
                .contains(&query)
            })
            .collect();
        div()
            .id("skills-scroll")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .v_flex()
            .p_6()
            .gap_4()
            .child(heading(self.t().skills_tools))
            .child(Input::new(&self.skill_search_input))
            .child(
                div().h_flex().flex_wrap().gap_2().children(
                    [
                        (None, self.t().skills_cat_all),
                        (Some(SkillCategory::Office), self.t().skills_cat_office),
                        (
                            Some(SkillCategory::Scientific),
                            self.t().skills_cat_scientific,
                        ),
                        (
                            Some(SkillCategory::Financial),
                            self.t().skills_cat_financial,
                        ),
                        (Some(SkillCategory::General), self.t().skills_cat_core),
                    ]
                    .into_iter()
                    .enumerate()
                    .map(|(index, (category, label))| {
                        action(format!("category-{index}"), label, Glyph::File)
                            .selected(self.skill_category == category)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.skill_category = category;
                                cx.notify();
                            }))
                    }),
                ),
            )
            .when(skills.is_empty(), |view| {
                view.child(div().child(self.tr("没有匹配的技能", "No matching skills")))
            })
            .child(
                div()
                    .h_flex()
                    .items_center()
                    .h(px(28.))
                    .gap(px(6.))
                    .child(
                        div()
                            .font_medium()
                            .child(self.tr("内置技能", "Built-in skills")),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(rgba(c.foreground_subtle))
                            .child(skills.len().to_string()),
                    ),
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
                            .id(format!("skill-row-{}", skill.id))
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
                            .hover(|style| style.bg(rgba(c.selected)))
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
                                Button::new(format!("skill-details-{}", skill.id))
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
                                    format!("use-{}", skill.id),
                                    self.tr("用于任务", "Use in task"),
                                    Glyph::Plus,
                                )
                                .disabled(self.is_executing)
                                .on_click(cx.listener(
                                    move |this, _, window, cx| {
                                        this.input_state.update(cx, |input, cx| {
                                            input.set_value(prompt.clone(), window, cx)
                                        });
                                        this.drafts.insert(this.session.id.clone(), prompt.clone());
                                        this.persist_active_session();
                                        this.focus_composer(window, cx);
                                    },
                                )),
                            )
                    })),
            )
            .into_any_element()
    }
    fn open_skill_detail(
        &self,
        skill: &pi_agent_core::skills::Skill,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let chinese = self.language == Language::Chinese;
        let name = if chinese { &skill.name } else { &skill.name_en }.clone();
        let description = if chinese {
            &skill.description
        } else {
            &skill.description_en
        }
        .clone();
        let instructions = skill.system_prompt_snippet.clone();
        let examples = skill.cli_examples.clone();
        let id = skill.id.clone();
        let description_label = self.tr("描述", "Description");
        let instructions_label = self.tr("技能指令", "Instructions");
        let examples_label = self.tr("命令示例", "Command examples");
        // The native dialog has no usable caller-side height cap: its own
        // `max_h(viewport-based max_height)` is applied after the caller's
        // style. Bound the complete header+body frame from inside instead.
        // The 2px subtraction accounts for the native surface's 1px top and
        // bottom borders, so the full outer frame stays within the cap.
        let max_content_height = (window.viewport_size().height * 0.8).min(px(640.)) - px(2.);
        let theme = Theme::global(cx);
        let border = theme.colors.border;
        let subtle = theme.colors.muted_foreground;
        window.open_dialog(cx, move |dialog, window, _| {
            dialog
                .width((window.viewport_size().width - px(32.)).min(px(576.)))
                .p_0()
                .close_button(true)
                .keyboard(true)
                .content({
                    let name = name.clone();
                    let description = description.clone();
                    let instructions = instructions.clone();
                    let examples = examples.clone();
                    let id = id.clone();
                    move |content, _, _| {
                        content
                            .max_h(max_content_height)
                            .min_h_0()
                            .gap_0()
                            .p_0()
                            .overflow_hidden()
                            .child(
                                gpui_kit::component::dialog::DialogTitle::new()
                                    .flex_shrink_0()
                                    .w_full()
                                    .min_w_0()
                                    .px(px(16.))
                                    .pt(px(16.))
                                    .pb(px(12.))
                                    .border_b_1()
                                    .border_color(border)
                                    .text_size(px(16.))
                                    .line_height(px(24.))
                                    .font_semibold()
                                    .child(
                                        div()
                                            .w_full()
                                            .min_w_0()
                                            .pr(px(32.))
                                            .truncate()
                                            .child(name.clone()),
                                    ),
                            )
                            .child(
                                div()
                                    .id(format!("skill-detail-{id}"))
                                    .w_full()
                                    .flex_1()
                                    .min_h_0()
                                    .v_flex()
                                    .px(px(16.))
                                    .py(px(20.))
                                    .gap_4()
                                    .overflow_y_scroll()
                                    .child(
                                        div()
                                            .v_flex()
                                            .gap(px(6.))
                                            .child(
                                                div()
                                                    .text_size(px(14.))
                                                    .line_height(px(20.))
                                                    .font_medium()
                                                    .child(description_label),
                                            )
                                            .child(
                                                div()
                                                    .id(format!("skill-description-scroll-{id}"))
                                                    .max_h(px(160.))
                                                    .overflow_y_scroll()
                                                    .text_size(px(14.))
                                                    .line_height(px(22.75))
                                                    .text_color(subtle)
                                                    .child(gpui_kit::base::SelectableText::new(
                                                        format!("skill-description-{id}"),
                                                        description.clone(),
                                                    )),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .v_flex()
                                            .gap(px(6.))
                                            .child(
                                                div()
                                                    .text_size(px(14.))
                                                    .line_height(px(20.))
                                                    .font_medium()
                                                    .child(instructions_label),
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(14.))
                                                    .line_height(px(20.))
                                                    .child(gpui_kit::base::SelectableText::new(
                                                        format!("skill-instructions-{id}"),
                                                        instructions.clone(),
                                                    )),
                                            ),
                                    )
                                    .when(!examples.is_empty(), |body| {
                                        body.child(
                                            div()
                                                .v_flex()
                                                .gap_3()
                                                .child(
                                                    div()
                                                        .text_size(px(14.))
                                                        .line_height(px(20.))
                                                        .font_medium()
                                                        .child(examples_label),
                                                )
                                                .children(examples.iter().enumerate().map(
                                                    |(index, example)| {
                                                        div()
                                                    .v_flex()
                                                    .gap_1()
                                                    .child(
                                                        div()
                                                            .text_size(px(14.))
                                                            .line_height(px(20.))
                                                            .font_medium()
                                                            .child(example.title.clone()),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_size(px(14.))
                                                            .line_height(px(20.))
                                                            .child(
                                                                gpui_kit::base::SelectableText::new(
                                                                    format!(
                                                                        "skill-example-{id}-{index}"
                                                                    ),
                                                                    example.command.clone(),
                                                                ),
                                                            ),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_size(px(14.))
                                                            .line_height(px(20.))
                                                            .text_color(subtle)
                                                            .child(example.description.clone()),
                                                    )
                                                    },
                                                )),
                                        )
                                    }),
                            )
                    }
                })
        });
    }
    fn automations_view(
        &self,
        window: &Window,
        c: &ThemeColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let create = action(
            "create-automation",
            self.tr("新建自动化", "New automation"),
            Glyph::Plus,
        )
        .on_click(cx.listener(|this, _, window, cx| this.edit_automation(None, window, cx)));
        div()
            .id("automations-scroll")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .v_flex()
            .p_6()
            .gap_4()
            .child(
                div()
                    .h_flex()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .child(heading(self.t().automations))
                    .child(create),
            )
            .child(
                div()
                    .text_size(px(13.))
                    .text_color(rgba(c.foreground_subtle))
                    .child(self.tr(
                        "应用打开时按间隔运行；关闭期间跳过。保存后默认暂停，启用后执行。",
                        "Runs at intervals while the app is open. Missed runs are skipped. New schedules start paused.",
                    )),
            )
            .when(self.automation_form, |view| {
                view.child(
                    div()
                        .v_flex()
                        .gap_3()
                        .p_4()
                        .rounded(px(12.))
                        .border_1()
                        .border_color(rgb(c.card_border))
                        .bg(rgb(c.card_bg))
                        .child(automation_field(self.tr("名称", "Name"), &self.automation_name, c))
                        .child(
                            div().v_flex().gap(px(6.))
                                .child(div().text_size(px(14.)).line_height(px(20.))
                                    .text_color(rgba(c.foreground_subtle))
                                    .child(self.tr("任务或 $ 命令", "Task or $ command")))
                                .child(Textarea::new(&self.automation_prompt)
                                    .min_h(px(116.)).max_h(px(156.))
                                    .text_size(px(14.)).line_height(px(20.))
                                    .rounded(px(12.)).bg(rgb(c.dock_bg))
                                    .aria_label(self.tr("任务或 $ 命令", "Task or $ command"))),
                        )
                        .child(automation_field(
                            self.tr("间隔（分钟）", "Interval (minutes)"),
                            &self.automation_minutes,
                            c,
                        ))
                        .when(!self.automation_error.is_empty(), |view| {
                            view.child(
                                div()
                                    .text_color(rgb(0xb42318))
                                    .child(self.automation_error.clone()),
                            )
                        })
                        .child(
                            div()
                                .h_flex()
                                .gap_2()
                                .child(action(
                                    "save-automation",
                                    self.tr("保存", "Save"),
                                    Glyph::Check,
                                )
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.save_automation(cx)
                                })))
                                .child(action(
                                    "cancel-automation",
                                    self.tr("取消", "Cancel"),
                                    Glyph::X,
                                )
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.automation_form = false;
                                    cx.notify();
                                }))),
                        ),
                )
            })
            .when(self.automations.is_empty(), |view| {
                view.child(
                    div()
                        .h(px(226.))
                        .h_flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(16.))
                        .border_1()
                        .border_color(rgb(c.card_border))
                        .bg(rgb(c.root_bg))
                        .text_color(rgba(c.foreground_subtle))
                        .child(self.tr("还没有自动化任务", "No automations yet")),
                )
            })
            .when(!self.automations.is_empty(), |view| {
                view.child(div().grid().grid_cols(automation_grid_columns(window.viewport_size().width)).gap_4()
                    .children(self.automations.iter().map(|job| {
                        let edit = job.id.clone();
                        let job_id = job.id.clone();
                        let app = cx.entity().downgrade();
                        let enabled = job.enabled;
                        let busy = self.is_executing;
                        let toggle_label = if enabled { self.tr("暂停", "Pause") } else { self.tr("启用", "Enable") };
                        let edit_label = self.tr("编辑", "Edit");
                        let run_label = self.tr("立即运行", "Run now");
                        let delete_label = self.tr("删除", "Delete");
                        div().relative().h(px(132.)).min_w_0()
                            .child(Button::new(format!("automation-{}", job.id)).custom(
                                gpui_kit::component::button::ButtonCustomVariant::new(cx)
                                    .color(rgb(c.root_bg).into())
                                    .hover(rgba(c.selected).into())
                                    .active(rgba(c.selected).into())
                            )
                                .w_full().h_full().min_w_0().p_3().rounded(px(12.))
                                .border_1().border_color(rgb(c.card_border))
                                .accessibility_label(format!("{} · {}", job.name, edit_label))
                                .child(div().w_full().h_full().min_w_0().v_flex().gap_3()
                                    .child(div().pr(px(48.)).truncate().text_size(px(14.)).line_height(px(20.)).font_medium().child(job.name.clone()))
                                    .child(div().h(px(40.)).min_h(px(40.)).overflow_hidden().whitespace_normal().text_size(px(14.)).line_height(px(20.)).text_color(rgba(c.foreground_subtle)).child(job.prompt.clone()))
                                    .child(div().h(px(24.)).h_flex().gap_2().min_w_0().text_size(px(14.)).text_color(rgba(c.foreground_subtle))
                                        .child(Icon::new(Glyph::Clock).size(px(16.)))
                                        .child(div().flex_1().min_w_0().truncate().child(automation_schedule_label(job.minutes, self.language == Language::Chinese)))
                                        .child(div().rounded(px(6.)).px(px(6.)).py(px(2.)).bg(rgba(c.card_user_bg)).child(if enabled {self.tr("已启用", "Enabled")} else {self.tr("已暂停", "Paused")}))))
                                .on_click(cx.listener(move |this, _, window, cx| this.edit_automation(Some(edit.clone()), window, cx))))
                            .child(icon_button(format!("automation-menu-{}", job.id), self.tr("任务操作", "Task actions"), Glyph::Ellipsis)
                                .w(px(24.)).h(px(24.)).rounded(px(6.))
                                .absolute().right(px(12.)).top(px(12.))
                                .dropdown_menu(move |menu, _, _| {
                                    let toggle_app=app.clone();let toggle_id=job_id.clone();
                                    let run_app=app.clone();let run_id=job_id.clone();
                                    let edit_app=app.clone();let edit_id=job_id.clone();
                                    let delete_app=app.clone();let delete_id=job_id.clone();
                                    menu.item(PopupMenuItem::new(toggle_label).on_click(move |_, _, cx| {
                                        let _=toggle_app.update(cx, |this, cx| {
                                            if let Some(job)=this.automations.iter_mut().find(|job|job.id==toggle_id) {
                                                job.enabled=!job.enabled;
                                                job.next_run=desktop_state::now()+u64::from(job.minutes)*60;
                                            }
                                            this.persist_active_session();cx.notify();
                                        });
                                    }))
                                    .item(PopupMenuItem::new(run_label).disabled(busy).on_click(move |_, window, cx| {
                                        let _=run_app.update(cx, |this, cx| {
                                            if this.is_executing {return;}
                                            if let Some(prompt)=this.automations.iter().find(|job|job.id==run_id).map(|job|job.prompt.clone()) {
                                                this.new_conversation(window,cx);this.submit_prompt(prompt,cx);
                                            }
                                        });
                                    }))
                                    .item(PopupMenuItem::new(edit_label).on_click(move |_, window, cx| {
                                        let _=edit_app.update(cx, |this,cx| this.edit_automation(Some(edit_id.clone()),window,cx));
                                    }))
                                    .item(PopupMenuItem::new(delete_label).on_click(move |_, _, cx| {
                                        let _=delete_app.update(cx, |this,cx| {
                                            this.automations.retain(|job|job.id!=delete_id);
                                            this.persist_active_session();cx.notify();
                                        });
                                    }))
                                }))
                    })))
            })
            .into_any_element()
    }
}

const USER_MESSAGE_BREAKPOINT: Pixels = px(624.);
const USER_MESSAGE_MAX_WIDTH: Pixels = px(576.);
const USER_MESSAGE_HORIZONTAL_PADDING: Pixels = px(32.);
const USER_MESSAGE_COLLAPSED_MAX_HEIGHT: f32 = 120.;
const USER_MESSAGE_OVERFLOW_TOLERANCE: f32 = 1.;

fn user_message_bubble_width(content_width: Pixels) -> Pixels {
    if content_width >= USER_MESSAGE_BREAKPOINT {
        USER_MESSAGE_MAX_WIDTH
    } else {
        (content_width - USER_MESSAGE_HORIZONTAL_PADDING).max(px(0.))
    }
}

fn user_message_content_is_expandable(measured_height: Pixels) -> bool {
    measured_height / px(1.) > USER_MESSAGE_COLLAPSED_MAX_HEIGHT + USER_MESSAGE_OVERFLOW_TOLERANCE
}

fn raw_tool_output_presentation(name: &str) -> (Pixels, Pixels) {
    // ZCode's execute renderer owns terminal output; keep the 14px/100px treatment
    // for tool aliases that produce the same raw stream, and use the fallback size
    // for all other raw tool payloads.
    let terminal = matches!(name, "bash" | "execute" | "shell");
    if terminal {
        (px(14.), px(100.))
    } else {
        (px(12.), px(200.))
    }
}

fn assistant_message_is_streaming(
    is_executing: bool,
    is_last_message: bool,
    role: &pi_agent_core::Role,
) -> bool {
    is_executing && is_last_message && *role == pi_agent_core::Role::Assistant
}

// A content edit resets disclosure, and identical message IDs in separate chats
// cannot change each other's expansion state. Hashing avoids storing message text in keys.
fn user_message_disclosure_key(session: &str, message: &str, content: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    (session, message, content).hash(&mut hash);
    format!("user-message-expand-{:016x}", hash.finish())
}

#[cfg(test)]
mod tests {
    use super::{
        assistant_message_is_streaming, automation_grid_columns, automation_schedule_label,
        chat_content_width, compact_header_actions_width, compact_header_controls_fit,
        compact_token_count, conversation_composer_width, header_uses_compact_navigation,
        raw_tool_output_presentation, sidebar_is_visible, user_message_bubble_width,
        user_message_content_is_expandable,
    };
    use gpui_kit::px;
    use pi_agent_core::Role;

    #[test]
    fn sidebar_visibility_and_chat_width_share_the_820px_breakpoint() {
        assert!(!sidebar_is_visible(false, px(1180.)));
        assert!(!sidebar_is_visible(true, px(819.)));
        assert!(sidebar_is_visible(true, px(820.)));
        assert_eq!(chat_content_width(px(1180.), true, false, false), px(912.));
        assert_eq!(chat_content_width(px(1180.), true, true, false), px(532.));
        assert_eq!(chat_content_width(px(1179.), true, true, true), px(911.));
        assert_eq!(chat_content_width(px(819.), true, false, true), px(819.));
    }

    #[test]
    fn compact_header_keeps_all_navigation_targets_at_minimum_width() {
        assert!(header_uses_compact_navigation(px(819.)));
        assert!(!header_uses_compact_navigation(px(820.)));
        // Five 28px targets and four 8px gaps.
        assert_eq!(compact_header_actions_width(), px(172.));
        assert!(compact_header_controls_fit(px(480.)));
        assert!(!compact_header_controls_fit(px(360.)));
    }

    #[test]
    fn conversation_composer_follows_reference_content_column() {
        assert_eq!(conversation_composer_width(px(480.)), px(480.));
        assert_eq!(conversation_composer_width(px(864.)), px(768.));
        assert_eq!(conversation_composer_width(px(1180.)), px(896.));
        assert_eq!(conversation_composer_width(px(1440.)), px(1056.));
    }

    #[test]
    fn user_bubbles_use_the_reference_width_and_overflow_tolerance() {
        assert_eq!(user_message_bubble_width(px(480.)), px(448.));
        assert_eq!(user_message_bubble_width(px(623.)), px(591.));
        assert_eq!(user_message_bubble_width(px(624.)), px(576.));
        assert_eq!(user_message_bubble_width(px(1180.)), px(576.));
        assert!(!user_message_content_is_expandable(px(121.)));
        assert!(user_message_content_is_expandable(px(121.01)));
    }

    #[test]
    fn raw_tool_output_uses_terminal_or_fallback_metrics() {
        assert_eq!(raw_tool_output_presentation("bash"), (px(14.), px(100.)));
        assert_eq!(raw_tool_output_presentation("execute"), (px(14.), px(100.)));
        assert_eq!(raw_tool_output_presentation("shell"), (px(14.), px(100.)));
        assert_eq!(
            raw_tool_output_presentation("read_file"),
            (px(12.), px(200.))
        );
    }

    #[test]
    fn only_the_current_assistant_text_row_is_streaming() {
        assert!(assistant_message_is_streaming(true, true, &Role::Assistant));
        assert!(!assistant_message_is_streaming(
            false,
            true,
            &Role::Assistant
        ));
        assert!(!assistant_message_is_streaming(
            true,
            false,
            &Role::Assistant
        ));
        assert!(!assistant_message_is_streaming(true, true, &Role::User));
    }

    #[test]
    fn large_token_counts_use_compact_labels() {
        assert_eq!(compact_token_count(999), "999 tok");
        assert_eq!(compact_token_count(12_345), "12.3k tok");
        assert_eq!(compact_token_count(1_234_567), "1.2m tok");
    }

    #[test]
    fn automation_cards_switch_to_two_columns_at_1024px() {
        assert_eq!(automation_grid_columns(px(1023.)), 1);
        assert_eq!(automation_grid_columns(px(1024.)), 2);
        assert_eq!(automation_grid_columns(px(1440.)), 2);
    }

    #[test]
    fn automation_schedule_metadata_is_compact_and_localized() {
        assert_eq!(automation_schedule_label(1, true), "每 1 分钟");
        assert_eq!(automation_schedule_label(60, false), "Every 60 min");
    }
}
