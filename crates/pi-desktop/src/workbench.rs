use super::*;
use gpui::StatefulInteractiveElement;
use gpui_kit::assets::IconName as Glyph;
use gpui_kit::component::button::{Button, ButtonCustomVariant, ButtonVariants};
use gpui_kit::component::menu::{DropdownMenu, PopupMenuItem};
use gpui_kit::prelude::FluentBuilder;
use interactions::*;

/// Shell geometry from the pinned ZCode sidebar and header contract.
const SIDEBAR_WIDTH: Pixels = px(268.);
const SIDEBAR_HIDE_BREAKPOINT: Pixels = px(820.);
const WORKBENCH_HEADER_HEIGHT: Pixels = px(48.);
const SIDEBAR_NAV_ROW_HEIGHT: Pixels = px(32.);
const SIDEBAR_HISTORY_ROW_HEIGHT: Pixels = px(32.);
const SIDEBAR_NAV_RADIUS: Pixels = px(8.);
const SIDEBAR_NAV_TO_TASKS_GAP: Pixels = px(21.);
const HEADER_ACTION_SIZE: Pixels = px(28.);
const HEADER_COMPACT_ACTION_COUNT: f32 = 5.;
const HEADER_COMPACT_GAP: Pixels = px(8.);
const HEADER_COMPACT_LEFT_CLEARANCE: Pixels = px(96.);

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
    side_pane_visible: bool,
    narrow_pane: bool,
) -> Pixels {
    let mut width = viewport_width;
    if sidebar_is_visible(sidebar_preference, viewport_width) {
        width -= SIDEBAR_WIDTH;
    }
    if side_pane_visible && !narrow_pane {
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
pub(crate) fn conversation_composer_width(width: Pixels) -> Pixels {
    if width >= px(1280.) {
        (width - px(384.)).clamp(px(0.), px(1152.))
    } else if width >= px(864.) {
        (width - px(96.)).clamp(px(0.), px(896.))
    } else {
        width
    }
}

/// `@md/conversation` uses Tailwind's default 28rem container breakpoint
/// (448px), rather than the 768px viewport breakpoint. A turn is `px-4`
/// below it and `px-6` from it. The composer dock stays `px-4`.
fn transcript_turn_padding(content_width: Pixels) -> Pixels {
    if content_width >= px(448.) {
        px(24.)
    } else {
        px(16.)
    }
}

/// Width available to a transcript row after its own horizontal padding.
fn transcript_content_width(content_width: Pixels) -> Pixels {
    let column = conversation_composer_width(content_width).min(content_width);
    let padding = transcript_turn_padding(content_width);
    (column - padding - padding).max(px(0.))
}

/// Space above a visible transcript row. A user bubble opens a turn with 56px,
/// and a later turn also keeps the previous turn's 20px bottom padding.
/// Consecutive reasoning and tool rows use the work-group 16px gap.
fn transcript_row_top(index: usize, user: bool, continues_work: bool) -> Pixels {
    if user {
        px(if index == 0 { 56. } else { 76. })
    } else if index == 0 {
        px(16.)
    } else if continues_work {
        px(16.)
    } else {
        px(20.)
    }
}

/// Reasoning and tool rows share one work group. Assistant prose does not.
fn transcript_message_is_work(message: &pi_agent_core::Message) -> bool {
    match message.role {
        pi_agent_core::Role::Tool => true,
        pi_agent_core::Role::Assistant => {
            message.content.trim().is_empty()
                && (message
                    .reasoning
                    .as_ref()
                    .is_some_and(|text| !text.trim().is_empty())
                    || message
                        .tool_calls
                        .as_ref()
                        .is_some_and(|calls| !calls.is_empty()))
        }
        _ => false,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ToolSummaryPhase {
    Running,
    Stopped,
    Done,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ToolFamily {
    Terminal,
    Read,
    Write,
    Edit,
    Search,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ToolKindLabel {
    Localized(&'static str, &'static str),
    Raw(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ToolSummaryCopy {
    icon: Glyph,
    kind: ToolKindLabel,
    detail: String,
    status: Option<(&'static str, &'static str)>,
}

fn tool_summary_family(name: &str) -> ToolFamily {
    match name {
        "bash" | "execute" | "shell" => ToolFamily::Terminal,
        "read" | "read_file" => ToolFamily::Read,
        "write" | "write_file" => ToolFamily::Write,
        "edit" | "edit_file" => ToolFamily::Edit,
        "grep" | "find" | "search" => ToolFamily::Search,
        _ => ToolFamily::Other,
    }
}

fn argument_text<'a>(arguments: Option<&'a serde_json::Value>, key: &str) -> Option<&'a str> {
    arguments
        .and_then(|value| value.get(key))
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn tool_summary_detail(
    name: &str,
    family: ToolFamily,
    arguments: Option<&serde_json::Value>,
) -> String {
    match family {
        ToolFamily::Terminal => argument_text(arguments, "command")
            .map(collapse_whitespace)
            .unwrap_or_default(),
        ToolFamily::Read | ToolFamily::Write | ToolFamily::Edit => argument_text(arguments, "path")
            .unwrap_or_default()
            .to_string(),
        ToolFamily::Search => argument_text(arguments, "pattern")
            .unwrap_or_default()
            .to_string(),
        ToolFamily::Other => match name {
            "calc" => argument_text(arguments, "action")
                .unwrap_or_default()
                .to_string(),
            "office" => argument_text(arguments, "file_path")
                .or_else(|| argument_text(arguments, "action"))
                .unwrap_or_default()
                .to_string(),
            _ => String::new(),
        },
    }
}

fn tool_summary_status(
    family: ToolFamily,
    phase: ToolSummaryPhase,
) -> Option<(&'static str, &'static str)> {
    if phase == ToolSummaryPhase::Failed {
        return Some(("执行失败", "Failed"));
    }
    if family != ToolFamily::Other {
        return None;
    }
    Some(match phase {
        ToolSummaryPhase::Running => ("执行中", "Running"),
        ToolSummaryPhase::Stopped => ("已停止", "Stopped"),
        ToolSummaryPhase::Done => ("已执行", "Completed"),
        ToolSummaryPhase::Failed => unreachable!(),
    })
}

/// Compact tool row from `ToolSummaryRow` / `ToolLayout`. Known families use the
/// reference kind words and argument detail. Success does not add a status word.
fn tool_summary_copy(
    name: &str,
    arguments: Option<&serde_json::Value>,
    phase: ToolSummaryPhase,
) -> ToolSummaryCopy {
    let family = tool_summary_family(name);
    let detail = tool_summary_detail(name, family, arguments);
    let (icon, kind) = match family {
        ToolFamily::Terminal => (
            Glyph::SquareTerminal,
            ToolKindLabel::Localized(
                if phase == ToolSummaryPhase::Running {
                    "正在执行"
                } else {
                    "终端"
                },
                if phase == ToolSummaryPhase::Running {
                    "Running"
                } else {
                    "Terminal"
                },
            ),
        ),
        ToolFamily::Read => (
            Glyph::Search,
            ToolKindLabel::Localized(
                if phase == ToolSummaryPhase::Running {
                    "正在读取"
                } else {
                    "读取"
                },
                if phase == ToolSummaryPhase::Running {
                    "Reading"
                } else {
                    "Read"
                },
            ),
        ),
        ToolFamily::Write => (
            Glyph::FilePen,
            ToolKindLabel::Localized(
                if phase == ToolSummaryPhase::Running {
                    "正在写入"
                } else {
                    "写入"
                },
                if phase == ToolSummaryPhase::Running {
                    "Writing"
                } else {
                    "Write"
                },
            ),
        ),
        ToolFamily::Edit => (
            Glyph::FilePen,
            ToolKindLabel::Localized(
                if phase == ToolSummaryPhase::Running {
                    "正在编辑"
                } else {
                    "编辑"
                },
                if phase == ToolSummaryPhase::Running {
                    "Editing"
                } else {
                    "Edit"
                },
            ),
        ),
        ToolFamily::Search => (
            Glyph::Search,
            ToolKindLabel::Localized(
                if phase == ToolSummaryPhase::Running {
                    "正在搜索"
                } else {
                    "搜索"
                },
                if phase == ToolSummaryPhase::Running {
                    "Searching"
                } else {
                    "Search"
                },
            ),
        ),
        ToolFamily::Other => (
            match name {
                "calc" => Glyph::Calculator,
                "office" => Glyph::File,
                _ => Glyph::Wrench,
            },
            ToolKindLabel::Raw(name.to_string()),
        ),
    };
    ToolSummaryCopy {
        icon,
        kind,
        detail,
        status: tool_summary_status(family, phase),
    }
}

fn tool_call_arguments<'a>(
    messages: &'a [Message],
    tool_call_id: &str,
) -> Option<&'a serde_json::Value> {
    messages.iter().find_map(|message| {
        message.tool_calls.as_ref().and_then(|calls| {
            calls
                .iter()
                .find(|call| call.id == tool_call_id)
                .map(|call| &call.arguments)
        })
    })
}

/// Thinking stays on the running label only until this turn produces text or a tool call.
fn reasoning_summary_is_streaming(
    executing: bool,
    is_last: bool,
    content_empty: bool,
    has_tool_call: bool,
) -> bool {
    executing && is_last && content_empty && !has_tool_call
}

struct ToolSummary {
    icon: Glyph,
    kind: SharedString,
    detail: SharedString,
    status: SharedString,
    text_size: Pixels,
    chevron_open: Option<bool>,
    chevron_opacity: f32,
    chevron_radians: f32,
    emphasis: KindEmphasis,
}

enum KindEmphasis {
    Quiet,
    Running,
    Sweep(SharedString),
}

fn tool_summary_content(summary: &ToolSummary, c: &ThemeColors) -> Div {
    let muted = rgba(c.foreground_subtlest);
    div()
        .h_flex()
        .items_center()
        .gap(px(8.))
        .min_w_0()
        .max_w_full()
        .text_size(summary.text_size)
        .child(
            Icon::new(summary.icon)
                .size(px(16.))
                .flex_shrink_0()
                .text_color(muted),
        )
        .child(tool_kind_label(summary, c))
        .when(!summary.detail.is_empty(), |row| {
            row.child(
                div()
                    .min_w_0()
                    .truncate()
                    .text_color(muted)
                    .child(summary.detail.clone()),
            )
        })
        .when(!summary.status.is_empty(), |row| {
            row.child(
                div()
                    .flex_shrink_0()
                    .whitespace_nowrap()
                    .text_color(muted)
                    .child(summary.status.clone()),
            )
        })
        .when(summary.chevron_open.is_some(), |row| {
            row.child(
                Icon::new(Glyph::ChevronRight)
                    .size(px(16.))
                    .flex_shrink_0()
                    .text_color(muted)
                    .opacity(summary.chevron_opacity)
                    .rotate(gpui::radians(summary.chevron_radians)),
            )
        })
}

fn tool_kind_label(summary: &ToolSummary, colors: &ThemeColors) -> AnyElement {
    match &summary.emphasis {
        KindEmphasis::Quiet => medium_label(summary.kind.clone(), rgba(colors.foreground_subtlest)),
        KindEmphasis::Running => medium_label(summary.kind.clone(), gradient_text_strong(colors)),
        KindEmphasis::Sweep(id) => sweeping_medium_label(
            id.clone(),
            summary.kind.clone(),
            gradient_text_strong(colors),
        ),
    }
}

fn medium_label(text: SharedString, color: Rgba) -> AnyElement {
    div()
        .flex_shrink_0()
        .whitespace_nowrap()
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(color)
        .child(text)
        .into_any_element()
}

/// Zai animated-gradient-text: a 300% ink gradient whose soft notch crosses the label.
fn sweeping_medium_label(id: SharedString, text: SharedString, strong: Rgba) -> AnyElement {
    div()
        .flex_shrink_0()
        .whitespace_nowrap()
        .font_weight(gpui::FontWeight::MEDIUM)
        .with_animation(
            id,
            gpui::Animation::new(std::time::Duration::from_secs(4)).repeat(),
            move |row, progress| row.child(gradient_styled_text(&text, progress, strong)),
        )
        .into_any_element()
}

fn gradient_styled_text(text: &str, progress: f32, strong: Rgba) -> gpui::StyledText {
    let count = text.chars().count().max(1) as f32;
    let highlights = text.char_indices().enumerate().map(|(glyph, (start, ch))| {
        let alpha = gradient_text_alpha(progress, (glyph as f32 + 0.5) / count);
        (
            start..start + ch.len_utf8(),
            gpui::HighlightStyle {
                color: Some(strong.opacity(alpha).into()),
                font_weight: Some(gpui::FontWeight::MEDIUM),
                ..gpui::HighlightStyle::default()
            },
        )
    });
    gpui::StyledText::new(text.to_owned()).with_highlights(highlights)
}

/// Light ink is #0d0d0d and dark ink is white. The soft stop is the same ink at 22%.
fn gradient_text_strong(colors: &ThemeColors) -> Rgba {
    if colors.root_bg == ThemeColors::dark().root_bg {
        rgba(0xffffffff)
    } else {
        rgba(0x0d0d0dff)
    }
}

const GRADIENT_TEXT_SOFT: f32 = 0.22;

fn gradient_text_alpha(progress: f32, glyph_center: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    let position = if progress < 0.5 {
        1.0 - progress * 2.0
    } else {
        0.0
    };
    let sample = (2.0 / 3.0) * position + glyph_center.clamp(0.0, 1.0) / 3.0;
    if sample <= 0.34 || sample >= 0.66 {
        1.0
    } else if sample <= 0.5 {
        let traveled = (sample - 0.34) / 0.16;
        1.0 + (GRADIENT_TEXT_SOFT - 1.0) * traveled
    } else {
        let traveled = (sample - 0.5) / 0.16;
        GRADIENT_TEXT_SOFT + (1.0 - GRADIENT_TEXT_SOFT) * traveled
    }
}

#[cfg(test)]
fn gradient_glyph_alphas(text: &str, progress: f32) -> Vec<f32> {
    let count = text.chars().count().max(1) as f32;
    (0..text.chars().count())
        .map(|glyph| gradient_text_alpha(progress, (glyph as f32 + 0.5) / count))
        .collect()
}

fn clip_tooltip(value: &str) -> String {
    let mut chars = value.chars();
    let clipped: String = chars.by_ref().take(280).collect();
    if chars.next().is_some() {
        format!("{clipped}...")
    } else {
        clipped
    }
}

fn summary_label(kind: &str, detail: &str, status: &str) -> String {
    let mut label = kind.to_string();
    if !detail.is_empty() {
        label.push(' ');
        label.push_str(detail);
    }
    if !status.is_empty() {
        label.push(' ');
        label.push_str(status);
    }
    label
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
        sync_component_theme(&colors, self.ui_text(14.), cx);
        self.sync_workflow_pane();
        let viewport_width = window.viewport_size().width;
        let compact = header_uses_compact_navigation(viewport_width);
        let sidebar =
            sidebar_is_visible(self.sidebar_visible, viewport_width) && !self.show_settings;
        let narrow_review = crate::panels::review_uses_single_column(window.viewport_size().width);
        let workflow_pane = self.workflow_pane_active();
        let body = if self.show_settings {
            self.settings_view(window, &colors, cx)
        } else {
            if workflow_pane && narrow_review {
                self.workflow_pane_view(&colors, cx)
            } else if self.review_visible && narrow_review {
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
            .id("zcode-workbench")
            .key_context("ZCode")
            .track_focus(&self.root_focus)
            .on_action(
                cx.listener(|this, _: &NewTask, window, cx| this.new_conversation(window, cx)),
            )
            .on_action(cx.listener(|this, _: &ToggleSidebar, _, cx| {
                this.sidebar_visible = !this.sidebar_visible;
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &ToggleSearch, window, cx| {
                this.search_open = true;
                this.search_input
                    .read(cx)
                    .focus_handle(cx)
                    .focus(window, cx);
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &Settings, _, cx| {
                this.show_settings = !this.show_settings;
                cx.notify();
            }))
            .on_action(
                cx.listener(|this, _: &FocusComposer, window, cx| this.focus_composer(window, cx)),
            )
            .on_action(
                cx.listener(|this, _: &ToggleReview, window, cx| this.toggle_review(window, cx)),
            )
            .on_action(cx.listener(|this, _: &ToggleTerminal, _, cx| this.toggle_terminal(cx)))
            .on_action(cx.listener(|this, _: &Dismiss, window, cx| {
                if this.search_open {
                    this.search_open = false;
                    this.focus_composer(window, cx);
                    cx.notify();
                    return;
                }
                if !this.search_input.read(cx).value().is_empty() {
                    this.search_input.update(cx, |input, cx| {
                        input.set_value(String::new(), window, cx);
                    });
                    cx.notify();
                    return;
                }
                if this
                    .search_input
                    .read(cx)
                    .focus_handle(cx)
                    .is_focused(window)
                {
                    this.focus_composer(window, cx);
                    return;
                }
                if this.workflow_pane_active() {
                    this.close_workflow_pane(window, cx);
                    return;
                }
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
                                .text_color(rgb(colors.workflow_error))
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
                            .when(workflow_pane && !narrow_review, |root| {
                                root.child(
                                    div()
                                        .w(crate::panels::REVIEW_SIDE_PANE_WIDTH)
                                        .flex_shrink_0()
                                        .h_full()
                                        .border_l_1()
                                        .border_color(rgba(colors.card_border))
                                        .child(self.workflow_pane_view(&colors, cx)),
                                )
                            })
                            .when(
                                self.review_visible
                                    && !workflow_pane
                                    && !narrow_review
                                    && !self.show_settings,
                                |root| {
                                    root.child(
                                        div()
                                            .w(crate::panels::REVIEW_SIDE_PANE_WIDTH)
                                            .flex_shrink_0()
                                            .h_full()
                                            .border_l_1()
                                            .border_color(rgba(colors.card_border))
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
            .when(self.search_open, |root| {
                root.child(self.search_modal(&colors, cx))
            })
    }
}

/// Keep gpui-kit controls (Input, Select, Switch and menus) on the same Zai
/// palette as the application-owned GPUI surfaces. Without this projection,
/// those controls silently retain gpui-kit defaults even when the surrounding
/// page has switched theme.
fn sync_component_theme(colors: &ThemeColors, font_size: Pixels, cx: &mut Context<PiDesktopApp>) {
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
    if Theme::global(cx).colors == palette && Theme::global(cx).font_size == font_size {
        return;
    }
    // update reconciles component tokens and the Base theme, unlike global_mut.
    Theme::update(cx, |theme| {
        theme.colors = palette;
        theme.font_size = font_size;
    });
}
impl PiDesktopApp {
    fn sidebar(&self, c: &ThemeColors, cx: &mut Context<Self>) -> AnyElement {
        let query = self.search_input.read(cx).value().to_lowercase();
        let visible_conversations: Vec<_> = self
            .conversations
            .iter()
            .filter(|entry| {
                let has_history = if entry.id == self.session.id {
                    !self.session.messages.is_empty()
                } else {
                    self.session_history
                        .get(&entry.id)
                        .map(|s| !s.messages.is_empty())
                        .unwrap_or(false)
                };
                has_history && (query.is_empty() || entry.title.to_lowercase().contains(&query))
            })
            .collect();
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
            // The 56px top strip matches the desktop drag/title clearance.
            // On macOS: traffic lights take ~96px, followed by sidebar toggle, back, and forward.
            .child(
                div()
                    .pt(px(10.))
                    .h(px(56.))
                    .pl(px(if cfg!(target_os = "macos") { 96. } else { 16. }))
                    .pr_2()
                    .flex_shrink_0()
                    .h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .h_flex()
                            .items_center()
                            .gap(px(4.))
                            .child(
                                icon_button(
                                    "sidebar-hide",
                                    self.tr("收起侧栏", "Hide sidebar"),
                                    Glyph::PanelLeftClose,
                                )
                                .on_click(cx.listener(
                                    |this, _, _, cx| {
                                        this.sidebar_visible = false;
                                        cx.notify();
                                    },
                                )),
                            )
                            .child(
                                Button::new("sidebar-nav-back")
                                    .ghost()
                                    .small()
                                    .h(px(28.))
                                    .w(px(28.))
                                    .p_0()
                                    .icon(Glyph::ArrowLeft)
                                    .tooltip(self.tr("返回", "Back"))
                                    .disabled(true),
                            )
                            .child(
                                Button::new("sidebar-nav-forward")
                                    .ghost()
                                    .small()
                                    .h(px(28.))
                                    .w(px(28.))
                                    .p_0()
                                    .icon(Glyph::ArrowRight)
                                    .tooltip(self.tr("前进", "Forward"))
                                    .disabled(true),
                            ),
                    )
                    .child(
                        div()
                            .h_flex()
                            .items_center()
                            .gap(px(2.))
                            .child(
                                icon_button(
                                    "toggle-search",
                                    self.tr("搜索 (⌘K)", "Search (⌘K)"),
                                    Glyph::Search,
                                )
                                .selected(self.search_open)
                                .on_click(cx.listener(
                                    |this, _, window, cx| {
                                        this.search_open = !this.search_open;
                                        if this.search_open {
                                            this.search_input
                                                .read(cx)
                                                .focus_handle(cx)
                                                .focus(window, cx);
                                        }
                                        cx.notify();
                                    },
                                )),
                            )
                            .child(icon_button(
                                "filter-tasks",
                                self.tr("筛选", "Filter"),
                                Glyph::SlidersHorizontal,
                            )),
                    ),
            )
            // Primary navigation: reference px-2, 4px rhythm, 32px rows.
            .child(
                div()
                    .v_flex()
                    .flex_shrink_0()
                    .gap_1()
                    .px_2()
                    .pt_1()
                    .pb_1()
                    .child(
                        Button::new("new-task")
                            .ghost()
                            .small()
                            .h(SIDEBAR_NAV_ROW_HEIGHT)
                            .rounded(SIDEBAR_NAV_RADIUS)
                            .justify_start()
                            .w_full()
                            .child(
                                div()
                                    .w_full()
                                    .h_flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        Icon::new(Glyph::CirclePlus).size(px(16.)).flex_shrink_0(),
                                    )
                                    .child(
                                        div()
                                            .min_w_0()
                                            .truncate()
                                            .child(self.tr("新建任务", "New Task")),
                                    )
                                    .child(
                                        div()
                                            .ml_auto()
                                            .flex_shrink_0()
                                            .text_size(px(11.))
                                            .font_normal()
                                            .text_color(rgba(c.foreground_subtlest))
                                            .child(if cfg!(target_os = "macos") {
                                                "⌘ N"
                                            } else {
                                                "Ctrl+N"
                                            }),
                                    ),
                            )
                            .accessibility_label(self.tr("新建任务", "New Task"))
                            .disabled(self.is_executing)
                            .on_click(
                                cx.listener(|this, _, window, cx| {
                                    this.new_conversation(window, cx)
                                }),
                            ),
                    )
                    .child(
                        sidebar_action(
                            "automation-nav",
                            self.tr("自动化", "Automations"),
                            Glyph::CalendarClock,
                        )
                        .w_full()
                        .selected(automations_active)
                        .toggled(automations_active)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.selected_nav = "automations";
                            this.show_settings = false;
                            cx.notify();
                        })),
                    )
                    .child(
                        sidebar_action("skills-nav", self.tr("插件市场", "Plugins"), Glyph::Blocks)
                            .w_full()
                            .selected(skills_active)
                            .toggled(skills_active)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.selected_nav = "skills";
                                this.show_settings = false;
                                cx.notify();
                            })),
                    ),
            )
            // Spacing to divider: 21pt (42 Retina px), landing divider at Retina y=352
            .child(div().h(SIDEBAR_NAV_TO_TASKS_GAP))
            // 1px horizontal divider matching ZCode 3.16.2 reference at Retina y=352
            .child(div().w_full().px_3().child(div().w_full().h(px(1.)).bg(
                if Theme::global(cx).is_dark() {
                    rgb(0x3d3d3d)
                } else {
                    rgba(c.card_border)
                },
            )))
            // Projects and tasks list
            .child(
                div().v_flex().flex_1().min_h_0().child(
                    div()
                        .id("history-list")
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scroll()
                        .v_flex()
                        .gap_1()
                        .px_2()
                        .pb_2()
                        .child(
                            div()
                                .pt(px(16.))
                                .px_2()
                                .h(px(28.))
                                .h_flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_size(px(12.))
                                        .font_medium()
                                        .text_color(rgba(c.foreground_subtle))
                                        .child(self.tr("项目", "Projects")),
                                )
                                .child(
                                    icon_button(
                                        "new-project",
                                        self.tr("新建项目", "New project"),
                                        Glyph::Plus,
                                    )
                                    .on_click(cx.listener(
                                        |this, _, window, cx| {
                                            this.toggle_review(window, cx);
                                        },
                                    )),
                                ),
                        )
                        .when(!visible_conversations.is_empty(), |list| {
                            list.child(
                                Button::new("sidebar-workspace")
                                    .ghost()
                                    .small()
                                    .h(SIDEBAR_NAV_ROW_HEIGHT)
                                    .rounded(SIDEBAR_NAV_RADIUS)
                                    .justify_start()
                                    .px(px(8.))
                                    .w_full()
                                    .tooltip(self.workspace_dir.display().to_string())
                                    .accessibility_label(self.workspace_name())
                                    .child(
                                        div()
                                            .w_full()
                                            .h_flex()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                Icon::new(Glyph::Folder)
                                                    .size(px(16.))
                                                    .text_color(rgba(c.foreground_subtle))
                                                    .flex_shrink_0(),
                                            )
                                            .child(
                                                div()
                                                    .min_w_0()
                                                    .truncate()
                                                    .text_size(px(14.))
                                                    .child(self.workspace_name()),
                                            ),
                                    ),
                            )
                        })
                        .child(
                            div()
                                .px_2()
                                .pt(px(16.))
                                .pb_1()
                                .text_size(px(12.))
                                .font_medium()
                                .text_color(rgba(c.foreground_subtlest))
                                .child(self.tr("任务", "Tasks")),
                        )
                        .when(
                            visible_conversations.is_empty() && query.is_empty(),
                            |list| {
                                list.child(
                                    div()
                                        .px_3()
                                        .py(px(8.))
                                        .text_size(px(13.))
                                        .text_color(rgba(c.foreground_subtlest))
                                        .child(self.tr("暂无任务", "No tasks")),
                                )
                            },
                        )
                        .when(
                            !query.is_empty() && visible_conversations.is_empty(),
                            |list| {
                                list.child(
                                    div()
                                        .px_3()
                                        .py_3()
                                        .text_size(px(13.))
                                        .text_color(rgba(c.foreground_subtlest))
                                        .child(self.tr("无匹配会话", "No matching threads")),
                                )
                            },
                        )
                        .children(visible_conversations.into_iter().map(|entry| {
                            let id = entry.id.clone();
                            Button::new(format!("history-{}", entry.id))
                                .ghost()
                                .small()
                                .h(SIDEBAR_HISTORY_ROW_HEIGHT)
                                .rounded(SIDEBAR_NAV_RADIUS)
                                .justify_start()
                                .px(px(8.))
                                .selected(entry.is_active)
                                .toggled(entry.is_active)
                                .disabled(self.is_executing)
                                .w_full()
                                .child(
                                    div()
                                        .w_full()
                                        .h_flex()
                                        .items_center()
                                        .gap_2()
                                        .when(entry.is_active && self.is_executing, |row| {
                                            row.child(
                                                Icon::new(Glyph::Loader)
                                                    .size(px(14.))
                                                    .text_color(rgba(c.foreground_subtle)),
                                            )
                                        })
                                        .child(
                                            div()
                                                .min_w_0()
                                                .flex_1()
                                                .truncate()
                                                .text_size(px(14.))
                                                .child(entry.title.clone()),
                                        ),
                                )
                                .accessibility_label(entry.title.clone())
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.activate_conversation(&id, window, cx)
                                }))
                        })),
                ),
            )
            // Footer: Left User profile "连接使用", Right Smartphone + Settings gear
            .child(
                div()
                    .flex_shrink_0()
                    .px_3()
                    .pt_2()
                    .pb_4()
                    .h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        Button::new("sidebar-profile")
                            .ghost()
                            .small()
                            .h(px(32.))
                            .rounded(px(8.))
                            .px_2()
                            .child(
                                div()
                                    .h_flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .w(px(22.))
                                            .h(px(22.))
                                            .rounded_full()
                                            .bg(rgba(c.card_border))
                                            .h_flex()
                                            .items_center()
                                            .justify_center()
                                            .child(
                                                div()
                                                    .w(px(10.))
                                                    .h(px(10.))
                                                    .rounded_full()
                                                    .bg(rgba(c.foreground_subtle)),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(13.))
                                            .font_normal()
                                            .text_color(rgba(c.foreground_subtle))
                                            .child(self.tr("连接使用", "Connect")),
                                    ),
                            )
                            .tooltip(self.tr("连接使用", "Connect")),
                    )
                    .child(
                        div()
                            .h_flex()
                            .items_center()
                            .gap(px(2.))
                            .child(
                                Button::new("sidebar-mobile")
                                    .ghost()
                                    .small()
                                    .h(px(28.))
                                    .w(px(28.))
                                    .p_0()
                                    .icon(Glyph::Smartphone)
                                    .tooltip(self.tr("移动端互联", "Mobile Connect")),
                            )
                            .child(
                                sidebar_action("settings-nav", self.t().settings, Glyph::Settings)
                                    .selected(self.show_settings)
                                    .toggled(self.show_settings)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.show_settings = !this.show_settings;
                                        cx.notify();
                                    })),
                            ),
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
    fn search_modal(&self, c: &ThemeColors, cx: &mut Context<Self>) -> AnyElement {
        let query = self.search_input.read(cx).value().trim().to_lowercase();
        let has_query = !query.is_empty();

        let matching_conversations: Vec<_> = self
            .conversations
            .iter()
            .filter(|entry| {
                let has_history = if entry.id == self.session.id {
                    !self.session.messages.is_empty()
                } else {
                    self.session_history
                        .get(&entry.id)
                        .map(|s| !s.messages.is_empty())
                        .unwrap_or(false)
                };
                has_history && (!has_query || entry.title.to_lowercase().contains(&query))
            })
            .collect();

        div()
            .id("search-modal-backdrop")
            .absolute()
            .inset_0()
            .bg(rgba(0x00000055))
            .h_flex()
            .justify_center()
            .items_start()
            .pt(px(90.))
            .on_click(cx.listener(|this, _, window, cx| {
                this.search_open = false;
                this.focus_composer(window, cx);
                cx.notify();
            }))
            .child(
                div()
                    .id("search-modal-card")
                    .w(px(640.))
                    .max_w(px(640.))
                    .bg(rgb(c.card_bg))
                    .border_1()
                    .border_color(rgba(c.card_border))
                    .rounded(px(16.))
                    .shadow_xl()
                    .overflow_hidden()
                    .v_flex()
                    .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    .child(
                        div()
                            .h(px(52.))
                            .px_4()
                            .h_flex()
                            .items_center()
                            .gap_3()
                            .border_b_1()
                            .border_color(rgba(c.card_border))
                            .child(
                                Icon::new(Glyph::Search)
                                    .size(px(18.))
                                    .text_color(rgba(c.foreground_subtle))
                                    .flex_shrink_0(),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .child(Input::new(&self.search_input)),
                            )
                            .child(
                                Button::new("close-search-modal-esc")
                                    .ghost()
                                    .small()
                                    .h(px(24.))
                                    .px(px(6.))
                                    .rounded(px(6.))
                                    .border_1()
                                    .border_color(rgba(c.card_border))
                                    .child(
                                        div()
                                            .text_size(px(11.))
                                            .font_medium()
                                            .text_color(rgba(c.foreground_subtle))
                                            .child("Esc"),
                                    )
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.search_open = false;
                                        this.focus_composer(window, cx);
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .px_4()
                            .pt_3()
                            .pb_1()
                            .text_size(px(12.))
                            .font_medium()
                            .text_color(rgba(c.foreground_subtlest))
                            .child(if has_query {
                                self.tr("搜索结果", "Search Results")
                            } else {
                                self.tr("最近任务", "Recent Tasks")
                            }),
                    )
                    .child(
                        div()
                            .id("search-modal-results")
                            .px_2()
                            .pb_2()
                            .max_h(px(360.))
                            .overflow_y_scroll()
                            .v_flex()
                            .gap(px(2.))
                            .when(matching_conversations.is_empty(), |list| {
                                list.child(
                                    div()
                                        .h_flex()
                                        .v_flex()
                                        .items_center()
                                        .justify_center()
                                        .py_8()
                                        .gap_2()
                                        .child(
                                            Icon::new(Glyph::Search)
                                                .size(px(24.))
                                                .text_color(rgba(c.foreground_subtlest)),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(13.))
                                                .text_color(rgba(c.foreground_subtlest))
                                                .child(if has_query {
                                                    self.tr("暂无搜索结果", "No results found")
                                                } else {
                                                    self.tr("暂无任务", "No tasks")
                                                }),
                                        ),
                                )
                            })
                            .children(matching_conversations.into_iter().map(|entry| {
                                let id = entry.id.clone();
                                let is_active = entry.is_active;
                                Button::new(format!("search-modal-item-{}", entry.id))
                                    .ghost()
                                    .small()
                                    .h(px(38.))
                                    .rounded(px(8.))
                                    .justify_start()
                                    .px_3()
                                    .w_full()
                                    .selected(is_active)
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.activate_conversation(&id, window, cx);
                                        this.search_open = false;
                                        this.focus_composer(window, cx);
                                        cx.notify();
                                    }))
                                    .child(
                                        div()
                                            .w_full()
                                            .h_flex()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .min_w_0()
                                                    .truncate()
                                                    .text_size(px(13.))
                                                    .font_medium()
                                                    .text_color(rgb(c.text_primary))
                                                    .child(entry.title.clone()),
                                            )
                                            .when(is_active, |row| {
                                                row.child(
                                                    div()
                                                        .px(px(6.))
                                                        .py(px(1.))
                                                        .rounded(px(4.))
                                                        .bg(rgba(c.selected))
                                                        .text_size(px(11.))
                                                        .font_medium()
                                                        .text_color(rgb(c.link))
                                                        .child(self.tr("当前任务", "Current")),
                                                )
                                            })
                                            .child(
                                                div()
                                                    .text_size(px(12.))
                                                    .text_color(rgba(c.foreground_subtlest))
                                                    .child(self.workspace_name()),
                                            ),
                                    )
                            })),
                    )
                    .child(
                        div()
                            .h_flex()
                            .items_center()
                            .gap_4()
                            .px_4()
                            .py_2()
                            .border_t_1()
                            .border_color(rgba(c.card_border))
                            .text_size(px(11.))
                            .text_color(rgba(c.foreground_subtlest))
                            .child(
                                div()
                                    .h_flex()
                                    .items_center()
                                    .gap_1()
                                    .child(
                                        div()
                                            .px(px(5.))
                                            .py(px(1.))
                                            .rounded(px(4.))
                                            .border_1()
                                            .border_color(rgba(c.card_border))
                                            .child("↑"),
                                    )
                                    .child(
                                        div()
                                            .px(px(5.))
                                            .py(px(1.))
                                            .rounded(px(4.))
                                            .border_1()
                                            .border_color(rgba(c.card_border))
                                            .child("↓"),
                                    )
                                    .child(self.tr("选择", "Select")),
                            )
                            .child(
                                div()
                                    .h_flex()
                                    .items_center()
                                    .gap_1()
                                    .child(
                                        div()
                                            .px(px(5.))
                                            .py(px(1.))
                                            .rounded(px(4.))
                                            .border_1()
                                            .border_color(rgba(c.card_border))
                                            .child("↵"),
                                    )
                                    .child(self.tr("打开", "Open")),
                            ),
                    ),
            )
            .into_any_element()
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
            .when(!self.session.messages.is_empty(), |row| {
                row.border_b_1().border_color(rgba(c.card_border))
            })
            .when(compact || !self.sidebar_visible, |row| {
                row.pl(px(if cfg!(target_os = "macos") { 96. } else { 16. }))
            })
            .when(!compact && !self.sidebar_visible, |row| {
                row.child(
                    icon_button(
                        "show-sidebar",
                        self.tr("展开侧栏", "Show sidebar"),
                        Glyph::PanelLeftOpen,
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.sidebar_visible = true;
                        cx.notify();
                    })),
                )
                .child(
                    icon_button(
                        "header-new-task",
                        self.t().new_conversation,
                        Glyph::SquarePen,
                    )
                    .disabled(self.is_executing)
                    .on_click(cx.listener(|this, _, window, cx| this.new_conversation(window, cx))),
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
                    icon_button("compact-auto", self.t().automations, Glyph::CalendarClock)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.selected_nav = "automations";
                            this.show_settings = false;
                            cx.notify();
                        })),
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
                    .h_flex()
                    .items_center()
                    .gap_2()
                    .when(
                        !self.show_settings && !self.session.messages.is_empty(),
                        |title_row| {
                            title_row
                                .child(
                                    Button::new("header-workspace-folder")
                                        .ghost()
                                        .small()
                                        .h(px(28.))
                                        .w(px(28.))
                                        .rounded(px(8.))
                                        .icon(Glyph::Folder)
                                        .tooltip(self.workspace_dir.display().to_string())
                                        .accessibility_label(self.workspace_name()),
                                )
                                .child(
                                    div()
                                        .min_w_0()
                                        .truncate()
                                        .text_size(self.ui_text(14.))
                                        .font_semibold()
                                        .text_color(rgb(c.text_primary))
                                        .child(title),
                                )
                        },
                    ),
            )
            .child(icon_button(
                "header-help",
                self.tr("帮助", "Help"),
                Glyph::CircleQuestionMark,
            ))
            .child(
                icon_button(
                    "toggle-terminal",
                    self.tr("终端", "Toggle terminal"),
                    Glyph::Terminal,
                )
                .selected(self.terminal_visible)
                .on_click(cx.listener(|this, _, _, cx| this.toggle_terminal(cx))),
            )
            .child(
                icon_button(
                    "toggle-review",
                    if self.workflow_pane_active() {
                        self.tr("收起工作流侧栏", "Close workflow side pane")
                    } else {
                        self.tr("变更审查", "Toggle side panel")
                    },
                    if self.review_visible || self.workflow_pane_active() {
                        Glyph::PanelRightClose
                    } else {
                        Glyph::PanelRightOpen
                    },
                )
                .selected(self.review_visible || self.workflow_pane_active())
                .on_click(cx.listener(|this, _, window, cx| {
                    if this.workflow_pane_active() {
                        this.close_workflow_pane(window, cx);
                    } else {
                        this.toggle_review(window, cx);
                    }
                })),
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
    fn arm_greeting_refresh(&self, cx: &mut Context<Self>) {
        if self.greeting_refresh.borrow().is_some() {
            return;
        }
        let (hour, minute, second, millis) = local_clock();
        let delay =
            std::time::Duration::from_millis(next_greeting_delay_ms(hour, minute, second, millis));
        *self.greeting_refresh.borrow_mut() = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(delay).await;
            let _ = this.update(cx, |this, cx| {
                *this.greeting_refresh.borrow_mut() = None;
                cx.notify();
            });
        }));
    }

    fn chat(&self, window: &mut Window, c: &ThemeColors, cx: &mut Context<Self>) -> AnyElement {
        let content_width = chat_content_width(
            window.viewport_size().width,
            self.sidebar_visible,
            self.review_visible || self.workflow_pane_active(),
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
            self.arm_greeting_refresh(cx);
            let (hour, _, _, _) = local_clock();
            let (greeting_zh, greeting_en) = empty_greeting(hour);
            let greeting: SharedString = self.tr(greeting_zh, greeting_en).into();
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
            let greeting_size =
                resolve_greeting_font_size(available_width / px(1.), natural_width / px(1.));
            let logo_width = (window.viewport_size().width * 0.72).min(px(400.));
            let logo_height = logo_width * 0.8;
            let dark = Theme::global(cx).is_dark();
            return div()
                .id("empty-conversation")
                .flex_1()
                .min_w_0()
                .min_h_0()
                .overflow_y_scroll()
                .children(self.goal_banner(c, cx))
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
                                .relative()
                                .w_full()
                                .max_w(px(672.))
                                .flex_shrink_0()
                                .mb(px(if window.viewport_size().width < px(640.) {
                                    40.
                                } else {
                                    49.
                                }))
                                .child(
                                    div()
                                        .absolute()
                                        .top(relative(0.5))
                                        .left(relative(0.5))
                                        .w(logo_width)
                                        .h(logo_height)
                                        .ml(logo_width * -0.5)
                                        .mt(logo_height * -0.5 - px(40.))
                                        .child(
                                            gpui::img(empty_logo_image(
                                                dark,
                                                c.foreground_subtlest,
                                            ))
                                            .w(logo_width)
                                            .h(logo_height),
                                        ),
                                )
                                .child(
                                    div()
                                        .w_full()
                                        .px_4()
                                        .text_center()
                                        .text_size(px(greeting_size))
                                        .line_height(px(greeting_size * 1.2))
                                        .font_medium()
                                        .text_color(rgb(c.text_primary))
                                        .child(greeting),
                                ),
                        )
                        .children(self.pending_recovery_banner(content_width, c, cx))
                        .children(self.draft_onboard_banner(content_width, c, cx))
                        .child(self.queue_view(content_width, c, cx))
                        .child(div().w_full().flex_shrink_0().child(self.composer(
                            content_width,
                            c,
                            cx,
                        )))
                        .children(self.draft_suggested_prompts(content_width, c, cx))
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
            .children(self.goal_banner(c, cx))
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
                    .pb(px(20.))
                    .children({
                        let visible_messages: Vec<_> = self
                            .session
                            .messages
                            .iter()
                            .filter(|message| message.role != pi_agent_core::Role::System)
                            .collect();
                        let file_summaries = crate::file_summary::by_row(
                            &visible_messages,
                            self.task_states.get(&self.session.id),
                        );
                        // Build the rows before the temporary message list drops.
                        // A lazy iterator would borrow that list past the block.
                        let mut rows = Vec::with_capacity(visible_messages.len());
                        for (index, message) in visible_messages.iter().enumerate() {
                            let continues_work = index > 0
                                && transcript_message_is_work(visible_messages[index - 1])
                                && transcript_message_is_work(message);
                            let mut row = div()
                                .w_full()
                                .max_w(conversation_composer_width(content_width))
                                .min_w_0()
                                .px(transcript_turn_padding(content_width))
                                .v_flex()
                                .items_stretch()
                                .gap(px(20.))
                                .pt(transcript_row_top(
                                    index,
                                    message.role == pi_agent_core::Role::User,
                                    continues_work,
                                ))
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
                            if let Some(summary) = file_summaries.get(&index) {
                                row = row.child(self.file_summary_panel(
                                    &summary.turn_id,
                                    &summary.changes,
                                    c,
                                    window,
                                    cx,
                                ));
                            }
                            rows.push(row);
                        }
                        rows
                    }),
            )
            .when(self.run_error.is_some(), |root| {
                let tone = error_banner_tone(self.run_stopped);
                let target_width = conversation_composer_width(content_width);
                root.child(
                    div()
                        .w_full()
                        .px_4()
                        .pb_2()
                        .h_flex()
                        .justify_center()
                        .child(
                            div()
                                .w_full()
                                .max_w(target_width)
                                .h_flex()
                                .items_center()
                                .justify_between()
                                .gap_3()
                                .px_3()
                                .py_2()
                                .rounded(px(12.))
                                .border_1()
                                .when(tone == ErrorBannerTone::Stopped, |b| {
                                    b.border_color(rgba(c.settings_border)).bg(rgb(c.dock_bg))
                                })
                                .when(tone == ErrorBannerTone::Failure, |b| {
                                    b.border_color(rgba(c.destructive_border))
                                        .bg(rgba(c.destructive_surface))
                                })
                                .child(
                                    div()
                                        .h_flex()
                                        .items_center()
                                        .gap_2()
                                        .min_w_0()
                                        .flex_1()
                                        .child(
                                            Icon::new(match tone {
                                                ErrorBannerTone::Stopped => Glyph::Square,
                                                ErrorBannerTone::Failure => Glyph::CircleAlert,
                                            })
                                            .size(px(16.))
                                            .text_color(match tone {
                                                ErrorBannerTone::Stopped => {
                                                    rgba(c.foreground_subtle)
                                                }
                                                ErrorBannerTone::Failure => rgb(c.workflow_error),
                                            }),
                                        )
                                        .child(
                                            div()
                                                .min_w_0()
                                                .truncate()
                                                .font_medium()
                                                .text_size(self.ui_text(14.))
                                                .text_color(match tone {
                                                    ErrorBannerTone::Stopped => {
                                                        rgba(c.foreground_subtle)
                                                    }
                                                    ErrorBannerTone::Failure => {
                                                        rgb(c.workflow_error)
                                                    }
                                                })
                                                .child(self.run_error.clone().unwrap_or_default()),
                                        ),
                                )
                                .child(
                                    Button::new("retry-run")
                                        .outline()
                                        .small()
                                        .h(px(28.))
                                        .rounded(px(8.))
                                        .disabled(self.is_executing)
                                        .child(
                                            div()
                                                .h_flex()
                                                .items_center()
                                                .gap_1()
                                                .child(Icon::new(Glyph::RefreshCw).size(px(14.)))
                                                .child(self.tr("重试", "Retry")),
                                        )
                                        .on_click(cx.listener(|this, _, _, cx| this.retry_run(cx))),
                                ),
                        ),
                )
            })
            .children(self.pending_recovery_banner(content_width, c, cx))
            .child(self.queue_view(content_width, c, cx))
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
    fn present_summary(
        &self,
        copy: ToolSummaryCopy,
        chevron_open: Option<bool>,
        emphasis: KindEmphasis,
    ) -> (ToolSummary, String) {
        let kind = match copy.kind {
            ToolKindLabel::Localized(zh, en) => self.tr(zh, en).to_string(),
            ToolKindLabel::Raw(name) => name,
        };
        let status = copy
            .status
            .map(|(zh, en)| self.tr(zh, en).to_string())
            .unwrap_or_default();
        let label = summary_label(&kind, &copy.detail, &status);
        let summary = ToolSummary {
            icon: copy.icon,
            kind: kind.into(),
            detail: copy.detail.into(),
            status: status.into(),
            text_size: self.ui_text(14.),
            chevron_open,
            chevron_opacity: if chevron_open.is_some() { 1. } else { 0. },
            chevron_radians: if chevron_open == Some(true) {
                std::f32::consts::FRAC_PI_2
            } else {
                0.
            },
            emphasis,
        };
        (summary, label)
    }
    fn track_message_hover(
        &self,
        id: String,
        row: Div,
        cx: &mut Context<Self>,
    ) -> gpui::Stateful<Div> {
        let hover_id = id.clone();
        row.id(SharedString::from(format!("message-hover-{id}")))
            .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                if *hovered {
                    if this.hovered_message.as_deref() != Some(hover_id.as_str()) {
                        this.hovered_message = Some(hover_id.clone());
                        cx.notify();
                    }
                } else if this.hovered_message.as_deref() == Some(hover_id.as_str()) {
                    this.hovered_message = None;
                    cx.notify();
                }
            }))
    }

    fn arm_copy_reset(&self, id: String, cx: &mut Context<Self>) {
        *self.copy_reset.borrow_mut() = Some(cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(1200))
                .await;
            let _ = this.update(cx, |this, cx| {
                if this.copied_message.as_deref() == Some(id.as_str()) {
                    this.copied_message = None;
                    cx.notify();
                }
            });
        }));
    }

    fn arm_reasoning_roll_wake(&self, deadline: std::time::Instant, cx: &mut Context<Self>) {
        if self
            .reasoning_roll_wake
            .borrow()
            .is_some_and(|current| current <= deadline)
        {
            return;
        }
        *self.reasoning_roll_wake.borrow_mut() = Some(deadline);
        let delay = deadline.saturating_duration_since(std::time::Instant::now());
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(delay).await;
            let _ = this.update(cx, |this, cx| {
                let now = std::time::Instant::now();
                let next = {
                    let mut rolls = this.reasoning_summary_rolls.borrow_mut();
                    let mut next: Option<std::time::Instant> = None;
                    for state in rolls.values_mut() {
                        settle_reasoning_roll(state, now);
                        if let Some(until) = state.hold_until.filter(|until| *until > now) {
                            next = Some(match next {
                                Some(current) => current.min(until),
                                None => until,
                            });
                        }
                    }
                    next
                };
                {
                    let mut armed = this.reasoning_roll_wake.borrow_mut();
                    if *armed == Some(deadline) {
                        *armed = None;
                    }
                }
                if let Some(until) = next {
                    this.arm_reasoning_roll_wake(until, cx);
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn message_action_focus(&self, message_id: &str, cx: &mut Context<Self>) -> FocusHandle {
        self.message_action_focus
            .borrow_mut()
            .entry(message_id.to_owned())
            .or_insert_with(|| cx.focus_handle().tab_stop(false))
            .clone()
    }

    fn render_assistant_content(
        &self,
        message_id: &str,
        text: &str,
        c: &ThemeColors,
        fade: bool,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let segments = crate::generative_ui::parse_message_segments(text);
        let font_size = f32::from(self.preferences.ui_font_size_px);
        let is_dark = Theme::global(cx).is_dark();
        let mut elements = Vec::new();

        for (idx, segment) in segments.into_iter().enumerate() {
            match segment {
                crate::generative_ui::ContentSegment::Markdown(md) => {
                    if !md.is_empty() {
                        elements.push(
                            assistant_markdown(
                                font_size,
                                format!("assistant-{message_id}-{idx}"),
                                md,
                                c,
                                fade && idx == 0,
                                cx,
                            )
                            .into_any_element(),
                        );
                    }
                }
                crate::generative_ui::ContentSegment::GenerativeUi(node) => {
                    elements.push(
                        div()
                            .my_2()
                            .child(crate::generative_ui::render_ui_node(
                                &node,
                                c,
                                font_size,
                                is_dark,
                                |action, app: &mut PiDesktopApp, window, cx| {
                                    app.handle_generative_ui_action(action, window, cx);
                                },
                                cx,
                            ))
                            .into_any_element(),
                    );
                }
            }
        }

        elements
    }

    fn message_actions(
        &self,
        message_id: &str,
        text: String,
        time_label: Option<String>,
        c: &ThemeColors,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Div {
        let focus = self.message_action_focus(message_id, cx);
        // Opacity on this row hides every descendant, including a focused button,
        // so the row itself has to become opaque for focus-within.
        let revealed = message_actions_are_revealed(
            self.hovered_message.as_deref() == Some(message_id),
            focus.contains_focused(window, cx),
        );
        let opacity = message_action_opacity(
            format!("message-actions-{message_id}").into(),
            revealed,
            self.preferences.reduced_motion,
            window,
            cx,
        );
        let copied = self.copied_message.as_deref() == Some(message_id);
        let copy_label = if copied {
            self.tr("已复制", "Copied")
        } else {
            self.tr("复制", "Copy")
        };
        let id = message_id.to_string();
        let copy_id = id.clone();
        let is_assistant = time_label.is_some();
        let feedback = self.message_feedback.get(message_id).copied();
        let like_active = feedback == Some(MessageFeedback::Like);
        let dislike_active = feedback == Some(MessageFeedback::Dislike);
        let like_id = id.clone();
        let dislike_id = id.clone();
        let edit_text = text.clone();
        let like_label = self.tr("赞同", "Helpful");
        let dislike_label = self.tr("不满意", "Unhelpful");
        let edit_label = self.tr("编辑", "Edit");
        let mut row = div()
            .track_focus(&focus)
            .tab_stop(false)
            .mt(px(4.))
            .h_flex()
            .items_center()
            .gap(px(4.))
            .opacity(opacity)
            .child(
                Button::new(format!("copy-{message_id}"))
                    .ghost()
                    .w(px(24.))
                    .h(px(24.))
                    .p_0()
                    .rounded(px(6.))
                    .tooltip(copy_label)
                    .accessibility_label(copy_label)
                    .child(
                        Icon::new(if copied { Glyph::Check } else { Glyph::Copy })
                            .size(px(14.))
                            .text_color(if copied {
                                rgb(c.accent_green)
                            } else {
                                rgb(c.text_primary)
                            }),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        cx.write_to_clipboard(gpui::ClipboardItem::new_string(text.clone()));
                        this.copied_message = Some(copy_id.clone());
                        this.arm_copy_reset(copy_id.clone(), cx);
                        cx.notify();
                    })),
            );

        if is_assistant {
            row = row
                .child(
                    Button::new(format!("like-{message_id}"))
                        .ghost()
                        .w(px(24.))
                        .h(px(24.))
                        .p_0()
                        .rounded(px(6.))
                        .tooltip(like_label)
                        .accessibility_label(like_label)
                        .child(Icon::new(Glyph::ThumbsUp).size(px(14.)).text_color(
                            if like_active {
                                rgb(c.accent_green)
                            } else {
                                rgba(c.foreground_subtle)
                            },
                        ))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            match this.message_feedback.get(&like_id) {
                                Some(MessageFeedback::Like) => {
                                    this.message_feedback.remove(&like_id);
                                }
                                _ => {
                                    this.message_feedback
                                        .insert(like_id.clone(), MessageFeedback::Like);
                                }
                            }
                            cx.notify();
                        })),
                )
                .child(
                    Button::new(format!("dislike-{message_id}"))
                        .ghost()
                        .w(px(24.))
                        .h(px(24.))
                        .p_0()
                        .rounded(px(6.))
                        .tooltip(dislike_label)
                        .accessibility_label(dislike_label)
                        .child(Icon::new(Glyph::ThumbsDown).size(px(14.)).text_color(
                            if dislike_active {
                                rgb(c.workflow_active)
                            } else {
                                rgba(c.foreground_subtle)
                            },
                        ))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            match this.message_feedback.get(&dislike_id) {
                                Some(MessageFeedback::Dislike) => {
                                    this.message_feedback.remove(&dislike_id);
                                }
                                _ => {
                                    this.message_feedback
                                        .insert(dislike_id.clone(), MessageFeedback::Dislike);
                                }
                            }
                            cx.notify();
                        })),
                );
            if let Some(label) = time_label {
                row = row.child(
                    div()
                        .text_size(self.ui_text(12.))
                        .text_color(rgba(c.foreground_subtlest))
                        .child(label),
                );
            }
        } else {
            row = row.child(
                Button::new(format!("edit-{message_id}"))
                    .ghost()
                    .w(px(24.))
                    .h(px(24.))
                    .p_0()
                    .rounded(px(6.))
                    .tooltip(edit_label)
                    .accessibility_label(edit_label)
                    .child(
                        Icon::new(Glyph::Pencil)
                            .size(px(14.))
                            .text_color(rgba(c.foreground_subtle)),
                    )
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.input_state.update(cx, |input, cx| {
                            input.set_value(edit_text.clone(), window, cx);
                        });
                        this.drafts
                            .insert(this.session.id.clone(), edit_text.clone());
                        this.persist_active_session();
                        this.focus_composer(window, cx);
                        cx.notify();
                    })),
            );
        }

        row
    }

    fn message(
        &self,
        message: &Message,
        content_width: Pixels,
        c: &ThemeColors,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let user = message.role == pi_agent_core::Role::User;
        let mut view = div().w_full().min_w_0().v_flex().gap(px(16.));
        if user {
            let (body, epilogue) = split_user_input_epilogue(&message.content, None);
            if body.trim().is_empty() && epilogue.is_none() {
                return div().into_any_element();
            }
            let bubble_width = user_message_bubble_width(content_width);
            let bubble_surface = rgb(c.root_bg).blend(rgba(c.card_user_bg));
            let message_id = message.id.clone();
            let copy_text = message.content.clone();

            let mut bubble_card = div()
                .min_w_0()
                .max_w(bubble_width)
                .px_4()
                .py_3()
                .rounded(px(12.))
                .rounded_tr(px(2.))
                .border_1()
                .border_color(rgba(c.card_border))
                .bg(rgba(c.card_user_bg))
                .v_flex()
                .gap_2();

            if !body.trim().is_empty() {
                let text: SharedString = body.to_string().into();
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
                        self.ui_text(14.),
                        &[run],
                        Some((bubble_width - px(34.)).max(px(1.))),
                        None,
                    )
                    .ok()
                    .map(|lines| {
                        lines
                            .iter()
                            .map(|line| line.size(self.ui_text(24.)).height)
                            .sum::<Pixels>()
                    });
                let expandable = text_height.is_some_and(user_message_content_is_expandable);
                let expand_key = user_message_disclosure_key(&self.session.id, &message.id, body);
                let expanded = self.expanded_tools.contains(&expand_key);
                let toggle_label = if expanded {
                    self.tr("收起消息", "Collapse message")
                } else {
                    self.tr("展开消息", "Expand message")
                };

                let body_element = div()
                    .relative()
                    .min_w_0()
                    .v_flex()
                    .child(
                        div()
                            .min_w_0()
                            .text_size(self.ui_text(14.))
                            .line_height(self.ui_text(20.))
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
                                                gpui::linear_color_stop(bubble_surface, 1.),
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
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.toggle_tool_expansion(&expand_key);
                                            cx.notify();
                                        })),
                                ),
                        )
                    });

                bubble_card = bubble_card.child(body_element);
            }

            if let Some(epilogue_text) = epilogue {
                let epilogue_key = format!("user-epilogue-{}-{}", self.session.id, message.id);
                let epilogue_open = self.expanded_tools.contains(&epilogue_key);
                let epilogue_label = self.tr("工作流引擎指令", "Workflow engine instructions");
                let epilogue_trimmed = trim_epilogue_lead(epilogue_text);

                let epilogue_element = div()
                    .id(format!("epilogue-{}", message.id))
                    .v_flex()
                    .gap_1()
                    .min_w_0()
                    .w_full()
                    .child(
                        div()
                            .id(format!("epilogue-toggle-{}", message.id))
                            .h_flex()
                            .items_center()
                            .gap_1()
                            .cursor_pointer()
                            .child(
                                Icon::new(if epilogue_open {
                                    Glyph::ChevronDown
                                } else {
                                    Glyph::ChevronRight
                                })
                                .size(px(12.))
                                .text_color(rgba(c.foreground_subtlest)),
                            )
                            .child(
                                div()
                                    .text_size(self.ui_text(12.))
                                    .text_color(rgba(c.foreground_subtlest))
                                    .child(epilogue_label),
                            )
                            .on_click(cx.listener({
                                let key = epilogue_key.clone();
                                move |this, _, _, cx| {
                                    this.toggle_tool_expansion(&key);
                                    cx.notify();
                                }
                            })),
                    )
                    .when(epilogue_open, |f| {
                        f.child(
                            div()
                                .id(format!("epilogue-body-{}", message.id))
                                .w_full()
                                .max_h(px(240.))
                                .overflow_y_scroll()
                                .text_size(self.ui_text(13.))
                                .line_height(self.ui_text(18.))
                                .text_color(rgba(c.foreground_subtle))
                                .child(gpui_kit::base::SelectableText::new(
                                    format!("epilogue-text-{}", message.id),
                                    epilogue_trimmed.to_string(),
                                )),
                        )
                    });

                bubble_card = bubble_card.child(epilogue_element);
            }

            return self
                .track_message_hover(message_id.clone(), view.items_end(), cx)
                .child(
                    div()
                        .min_w_0()
                        .max_w(bubble_width)
                        .v_flex()
                        .items_end()
                        .child(bubble_card)
                        .when(!copy_text.is_empty(), |column| {
                            column.child(self.message_actions(
                                &message_id,
                                copy_text,
                                None,
                                c,
                                window,
                                cx,
                            ))
                        }),
                )
                .into_any_element();
        }
        if let Some(tool) = &message.tool_result {
            let id = tool.tool_call_id.clone();
            let expanded = self.expanded_tools.contains(&id);
            let arguments = tool_call_arguments(&self.session.messages, &id).cloned();
            let command_text = argument_text(arguments.as_ref(), "command")
                .filter(|_| matches!(tool.name.as_str(), "bash" | "execute" | "shell"))
                .map(str::to_owned);
            let mut copy = tool_summary_copy(
                &tool.name,
                arguments.as_ref(),
                if tool.is_error {
                    ToolSummaryPhase::Failed
                } else {
                    ToolSummaryPhase::Done
                },
            );
            // An open terminal row moves the command into the panel.
            if terminal_summary_hides_detail(&tool.name, expanded, command_text.is_some()) {
                copy.detail.clear();
            }
            let (mut summary, label) =
                self.present_summary(copy, Some(expanded), KindEmphasis::Quiet);
            let revealed = summary_disclosure_is_visible(
                expanded,
                self.hovered_tool.as_deref() == Some(id.as_str()),
            );
            let (opacity, radians) = disclosure_motion(
                format!("tool-chevron-{id}").into(),
                revealed,
                expanded,
                self.preferences.reduced_motion,
                200,
                [0.0, 0.0, 0.2, 1.0],
                window,
                cx,
            );
            summary.chevron_opacity = opacity;
            summary.chevron_radians = radians;
            let tooltip = if tool.is_error && !tool.output.trim().is_empty() {
                clip_tooltip(tool.output.trim())
            } else {
                label.clone()
            };
            return view
                .gap(px(0.))
                .child(
                    div()
                        .id(format!("tool-hover-{id}"))
                        .max_w_full()
                        .self_start()
                        .on_hover(cx.listener({
                            let hover_id = id.clone();
                            move |this, hovered: &bool, _, cx| {
                                if *hovered {
                                    if this.hovered_tool.as_deref() != Some(hover_id.as_str()) {
                                        this.hovered_tool = Some(hover_id.clone());
                                        cx.notify();
                                    }
                                } else if this.hovered_tool.as_deref() == Some(hover_id.as_str()) {
                                    this.hovered_tool = None;
                                    cx.notify();
                                }
                            }
                        }))
                        .child(
                            Button::new(format!("tool-{id}"))
                                .text()
                                .h_auto()
                                .p_0()
                                .w_auto()
                                .max_w_full()
                                .self_start()
                                .justify_start()
                                .toggled(expanded)
                                .accessibility_label(label)
                                .tooltip(tooltip)
                                .child(tool_summary_content(&summary, c))
                                .on_click({
                                    let toggle_id = id.clone();
                                    cx.listener(move |this, _, _, cx| {
                                        if this.expanded_tools.contains(&toggle_id) {
                                            let command_key =
                                                execute_command_key(&this.session.id, &toggle_id);
                                            this.expanded_tools.remove(&command_key);
                                            this.tool_output_scroll
                                                .borrow_mut()
                                                .remove(&format!("{command_key}:scroll"));
                                        }
                                        this.toggle_tool_expansion(&toggle_id);
                                        cx.notify();
                                    })
                                }),
                        ),
                )
                .when(expanded, |view| {
                    let (_, output_max_height) = raw_tool_output_presentation(&tool.name);
                    let output_font_size = raw_tool_output_font_size(
                        &tool.name,
                        f32::from(self.preferences.ui_font_size_px),
                    );
                    let chrome = tool_output_chrome(&tool.name);
                    let scroll_id = format!("tool-output-scroll-{}", message.id);
                    let scroll = self
                        .tool_output_scroll
                        .borrow_mut()
                        .entry(scroll_id.clone())
                        .or_default()
                        .clone();
                    let wheel_scroll = scroll.clone();
                    let output = tool.output.clone();
                    let empty_output = chrome.bordered && output.trim().is_empty();
                    let no_output = self.tr("没有输出。", "No output.").to_string();
                    view.child(
                        div()
                            .min_w_0()
                            .mt(chrome.before)
                            .mb(chrome.after)
                            .px(chrome.pad_x)
                            .py(chrome.pad_y)
                            .rounded(chrome.radius)
                            .v_flex()
                            .gap(px(12.))
                            .bg(if chrome.bordered {
                                rgb(c.panel_bg)
                            } else {
                                rgba(c.card_user_bg)
                            })
                            .when(chrome.bordered, |panel| {
                                panel.border_1().border_color(rgba(c.settings_border))
                            })
                            .when(command_text.is_some(), |panel| {
                                panel.child(self.execute_command_block(
                                    &id,
                                    command_text.as_deref().unwrap_or_default(),
                                    content_width,
                                    c,
                                    window,
                                    cx,
                                ))
                            })
                            .child(if empty_output {
                                div()
                                    .font_family(cx.theme().mono_font_family.clone())
                                    .text_size(output_font_size)
                                    .line_height(px(20.))
                                    .text_color(rgba(c.foreground_subtle))
                                    .child(no_output)
                                    .into_any_element()
                            } else {
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
                                        output,
                                    ))
                                    .into_any_element()
                            }),
                    )
                })
                .into_any_element();
        }
        let last_message_id = self
            .session
            .messages
            .iter()
            .rev()
            .find(|message| message.role != pi_agent_core::Role::System)
            .map(|message| message.id.as_str());
        if let Some(reasoning) = &message.reasoning {
            let id = format!("reasoning-{}", message.id);
            let expanded = self.expanded_tools.contains(&id);
            let streaming_reasoning = reasoning_summary_is_streaming(
                self.is_executing,
                last_message_id == Some(message.id.as_str()),
                message.content.is_empty(),
                message
                    .tool_calls
                    .as_ref()
                    .is_some_and(|calls| !calls.is_empty()),
            );
            let thought = if streaming_reasoning {
                self.tr("正在思考", "Thinking")
            } else {
                self.tr("思考", "Thought")
            };
            let duration = self.tr("持续了几秒", "a few seconds");
            let muted = rgba(c.foreground_subtlest);
            let text_size = self.ui_text(14.);
            // Playback can still be showing the held line, so scroll that line rather than the latest one.
            let streaming_summary =
                if reasoning_streaming_summary_is_shown(streaming_reasoning, expanded) {
                    reasoning_streaming_summary_line(reasoning).and_then(|line| {
                        let (generation, incoming, outgoing, wake) = remember_reasoning_summary(
                            &self.reasoning_summary_rolls,
                            &message.id,
                            line.index,
                            line.text,
                            std::time::Instant::now(),
                            !self.preferences.reduced_motion,
                        );
                        if let Some(deadline) = wake {
                            self.arm_reasoning_roll_wake(deadline, cx);
                        }
                        let label_width =
                            shaped_text_width(window, thought, text_size, gpui::FontWeight::MEDIUM);
                        let dot_width =
                            shaped_text_width(window, "·", text_size, gpui::FontWeight::NORMAL);
                        let budget = reasoning_summary_text_budget(
                            reasoning_summary_row_width(content_width),
                            label_width,
                            dot_width,
                        );
                        visible_reasoning_summary(
                            &incoming,
                            outgoing.as_deref(),
                            budget,
                            |sample| {
                                shaped_text_width(
                                    window,
                                    sample,
                                    text_size,
                                    gpui::FontWeight::NORMAL,
                                )
                            },
                        )
                        .map(|window| (window, generation))
                    })
                } else {
                    self.reasoning_summary_rolls
                        .borrow_mut()
                        .remove(&message.id);
                    None
                };
            let reasoning_label = if let Some((summary, _)) = streaming_summary.as_ref() {
                if summary.text.is_empty() {
                    thought.to_string()
                } else {
                    format!("{thought} · {}", summary.text)
                }
            } else if streaming_reasoning {
                thought.to_string()
            } else {
                format!("{thought} · {duration}")
            };
            let reasoning_revealed = summary_disclosure_is_visible(
                expanded,
                self.hovered_reasoning.as_deref() == Some(id.as_str()),
            );
            let (reasoning_opacity, reasoning_radians) = disclosure_motion(
                format!("reasoning-chevron-{}", message.id).into(),
                reasoning_revealed,
                expanded,
                self.preferences.reduced_motion,
                150,
                [0.4, 0.0, 0.2, 1.0],
                window,
                cx,
            );
            view = view
                .child(
                    div()
                        .id(format!("reasoning-hover-{id}"))
                        .max_w_full()
                        .self_start()
                        .on_hover(cx.listener({
                            let hover_id = id.clone();
                            move |this, hovered: &bool, _, cx| {
                                if *hovered {
                                    if this.hovered_reasoning.as_deref() != Some(hover_id.as_str())
                                    {
                                        this.hovered_reasoning = Some(hover_id.clone());
                                        cx.notify();
                                    }
                                } else if this.hovered_reasoning.as_deref()
                                    == Some(hover_id.as_str())
                                {
                                    this.hovered_reasoning = None;
                                    cx.notify();
                                }
                            }
                        }))
                        .child(
                            Button::new(id.clone())
                                .text()
                                .h_auto()
                                .p_0()
                                .w_auto()
                                .max_w_full()
                                .self_start()
                                .justify_start()
                                .toggled(expanded)
                                .accessibility_label(reasoning_label.clone())
                                .tooltip(reasoning_label)
                                .child(
                                    div()
                                        .h_flex()
                                        .items_center()
                                        .gap(px(8.))
                                        .min_w_0()
                                        .max_w_full()
                                        .text_size(text_size)
                                        .child(
                                            Icon::new(Glyph::Brain)
                                                .size(px(16.))
                                                .flex_shrink_0()
                                                .text_color(muted),
                                        )
                                        .child(if streaming_reasoning {
                                            if self.preferences.reduced_motion {
                                                medium_label(
                                                    thought.into(),
                                                    gradient_text_strong(c),
                                                )
                                            } else {
                                                sweeping_medium_label(
                                                    format!("reasoning-sweep-{}", message.id)
                                                        .into(),
                                                    thought.into(),
                                                    gradient_text_strong(c),
                                                )
                                            }
                                        } else {
                                            medium_label(thought.into(), muted)
                                        })
                                        .when(!streaming_reasoning, |row| {
                                            row.child(
                                                div().flex_shrink_0().text_color(muted).child("·"),
                                            )
                                            .child(
                                                div()
                                                    .flex_shrink_0()
                                                    .whitespace_nowrap()
                                                    .text_color(muted)
                                                    .child(duration),
                                            )
                                        })
                                        .when_some(
                                            streaming_summary,
                                            |row, (summary, generation)| {
                                                let surface = rgb(c.root_bg);
                                                let summary_color = rgba(c.foreground_subtle);
                                                row.child(
                                                    div()
                                                        .flex_shrink_0()
                                                        .font_weight(gpui::FontWeight::NORMAL)
                                                        .text_color(muted)
                                                        .child("·"),
                                                )
                                                .child(
                                                    div()
                                                        .relative()
                                                        .overflow_hidden()
                                                        .flex_shrink_0()
                                                        .when(
                                                            summary.scroll_left > px(0.),
                                                            |line| {
                                                                line.w(summary.client_width)
                                                                    .min_w(summary.client_width)
                                                                    .max_w(summary.client_width)
                                                            },
                                                        )
                                                        .font_weight(gpui::FontWeight::NORMAL)
                                                        .text_color(summary_color)
                                                        .child(rolling_reasoning_summary(
                                                            format!(
                                                                "reasoning-roll-{}-{generation}",
                                                                message.id
                                                            )
                                                            .into(),
                                                            summary.text,
                                                            summary.outgoing,
                                                            summary.scroll_left,
                                                            summary.outgoing_scroll_left,
                                                            text_size,
                                                            summary_color,
                                                        ))
                                                        .when(summary.masked, |line| {
                                                            let line_height = text_size * 2.;
                                                            line.child(reasoning_summary_edge(
                                                                ReasoningSummaryEdge::Leading,
                                                                surface,
                                                                line_height,
                                                            ))
                                                            .child(reasoning_summary_edge(
                                                                ReasoningSummaryEdge::Trailing,
                                                                surface,
                                                                line_height,
                                                            ))
                                                        }),
                                                )
                                            },
                                        )
                                        .child(
                                            div().flex_shrink_0().opacity(reasoning_opacity).child(
                                                Icon::new(Glyph::ChevronRight)
                                                    .size(px(16.))
                                                    .text_color(muted)
                                                    .rotate(gpui::radians(reasoning_radians)),
                                            ),
                                        ),
                                )
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.toggle_tool_expansion(&id);
                                    cx.notify();
                                })),
                        ),
                )
                .when(expanded, |view| {
                    let scroll_id = format!("reasoning-scroll-{}", message.id);
                    let scroll = self
                        .tool_output_scroll
                        .borrow_mut()
                        .entry(scroll_id.clone())
                        .or_default()
                        .clone();
                    let follow = self
                        .reasoning_scroll_follow
                        .borrow_mut()
                        .entry(scroll_id.clone())
                        .or_insert_with(|| std::rc::Rc::new(std::cell::Cell::new(true)))
                        .clone();
                    let wheel_scroll = scroll.clone();
                    let wheel_follow = follow.clone();
                    let scroll_top = -f32::from(scroll.offset().y);
                    let max_scroll = f32::from(scroll.max_offset().y);
                    let (mask_top, mask_bottom) = reasoning_scroll_mask(scroll_top, max_scroll);
                    let surface = rgb(c.root_bg);
                    let pin_scroll = scroll.clone();
                    let pin_follow = follow.clone();
                    let pin_entity = cx.entity_id();
                    view.child(
                        div()
                            .relative()
                            .pt(px(12.))
                            .child(
                                div()
                                    .id(scroll_id)
                                    .ml(px(8.))
                                    .pl(px(14.))
                                    .border_l_1()
                                    .border_color(rgba(c.settings_border))
                                    .max_h(px(240.))
                                    .min_w_0()
                                    .overflow_y_scroll()
                                    .track_scroll(&scroll)
                                    .on_scroll_wheel(
                                        move |event: &gpui::ScrollWheelEvent, window, cx| {
                                            let delta_y =
                                                event.delta.pixel_delta(window.line_height()).y
                                                    / px(1.);
                                            let offset = -wheel_scroll.offset().y / px(1.);
                                            let maximum = wheel_scroll.max_offset().y / px(1.);
                                            if delta_y != 0. {
                                                wheel_follow.set(reasoning_follow_after_wheel(
                                                    offset, maximum, delta_y,
                                                ));
                                            }
                                            if crate::transcript_scroll::nested_wheel_captures(
                                                delta_y, offset, maximum,
                                            ) {
                                                cx.stop_propagation();
                                            }
                                        },
                                    )
                                    .text_size(text_size)
                                    .line_height(px(text_size.as_f32() * 1.5))
                                    .text_color(muted)
                                    .child(gpui_kit::base::SelectableText::new(
                                        format!("reasoning-text-{}", message.id),
                                        reasoning.clone(),
                                    )),
                            )
                            .child(
                                gpui::canvas(
                                    |_, _, _| (),
                                    move |_, _, _, cx| {
                                        if !pin_follow.get() {
                                            return;
                                        }
                                        let top = -f32::from(pin_scroll.offset().y);
                                        let maximum = f32::from(pin_scroll.max_offset().y);
                                        if maximum - top > REASONING_BOTTOM_LOCK {
                                            let offset = pin_scroll.offset();
                                            pin_scroll.set_offset(point(offset.x, px(-maximum)));
                                            cx.notify(pin_entity);
                                        }
                                    },
                                )
                                .absolute()
                                .w(px(0.))
                                .h(px(0.)),
                            )
                            .when(mask_top, |body| {
                                body.child(reasoning_scroll_fade(true, surface))
                            })
                            .when(mask_bottom, |body| {
                                body.child(reasoning_scroll_fade(false, surface))
                            }),
                    )
                })
                .when(!expanded, |view| {
                    if let Some(follow) = self
                        .reasoning_scroll_follow
                        .borrow()
                        .get(&format!("reasoning-scroll-{}", message.id))
                    {
                        follow.set(true);
                    }
                    view
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
                    let copy = tool_summary_copy(
                        &call.name,
                        Some(&call.arguments),
                        if self.is_executing {
                            ToolSummaryPhase::Running
                        } else {
                            ToolSummaryPhase::Stopped
                        },
                    );
                    let emphasis = if self.is_executing {
                        if self.preferences.reduced_motion {
                            KindEmphasis::Running
                        } else {
                            KindEmphasis::Sweep(format!("tool-sweep-{}", call.id).into())
                        }
                    } else {
                        KindEmphasis::Quiet
                    };
                    let (summary, _) = self.present_summary(copy, None, emphasis);
                    view = view.child(tool_summary_content(&summary, c));
                }
            }
        }
        let streaming = assistant_message_is_streaming(
            self.is_executing,
            last_message_id == Some(message.id.as_str()),
            &message.role,
        );
        let copy_text = message.content.clone();
        let message_id = message.id.clone();
        let time_label = message_time_label(message.timestamp, self.language == Language::Chinese);
        let show_actions = !streaming && !copy_text.is_empty();
        let work_before_text = message
            .reasoning
            .as_ref()
            .is_some_and(|text| !text.trim().is_empty())
            || message.tool_calls.as_ref().is_some_and(|calls| {
                calls.iter().any(|call| {
                    !self.session.messages.iter().any(|candidate| {
                        candidate
                            .tool_result
                            .as_ref()
                            .is_some_and(|result| result.tool_call_id == call.id)
                    })
                })
            });
        // The work cluster uses the 16px flex gap. Prose after it is a separate
        // flow item, so add the remaining 4px to reach the 20px turn gap.
        let rendered = view.when(!copy_text.is_empty(), |view| {
            view.child(
                div()
                    .w_full()
                    .min_w_0()
                    .when(work_before_text, |text| text.mt(px(4.)))
                    .children(self.render_assistant_content(
                        &message_id,
                        &copy_text,
                        c,
                        !self.preferences.reduced_motion && streaming,
                        cx,
                    ))
                    .when(show_actions, |column| {
                        column.child(self.message_actions(
                            &message_id,
                            copy_text,
                            time_label,
                            c,
                            window,
                            cx,
                        ))
                    }),
            )
        });
        if show_actions {
            self.track_message_hover(message_id, rendered, cx)
                .into_any_element()
        } else {
            rendered.into_any_element()
        }
    }

    fn execute_command_block(
        &self,
        tool_id: &str,
        command: &str,
        content_width: Pixels,
        c: &ThemeColors,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Div {
        let key = execute_command_key(&self.session.id, tool_id);
        let open = self.expanded_tools.contains(&key);
        let font_size = self.ui_text(14.);
        let mut font = window.text_style().font();
        font.family = cx.theme().mono_font_family.clone();
        font.weight = gpui::FontWeight::NORMAL;
        let sample: SharedString = command.to_owned().into();
        let run = gpui::TextRun {
            len: sample.len(),
            font: font.clone(),
            color: rgb(c.text_primary).into(),
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let ch_run = gpui::TextRun {
            len: 1,
            font,
            color: rgb(c.text_primary).into(),
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let ch_width = window
            .text_system()
            .shape_line("0".into(), font_size, &[ch_run], None)
            .width;
        let text_width =
            execute_command_text_width(reasoning_summary_row_width(content_width), ch_width);
        let measured = window
            .text_system()
            .shape_text(sample.clone(), font_size, &[run], Some(text_width), None)
            .ok()
            .map(|lines| {
                lines
                    .iter()
                    .map(|line| line.size(px(EXECUTE_COMMAND_LINE_PX)).height)
                    .sum::<Pixels>()
            })
            .unwrap_or(px(EXECUTE_COMMAND_LINE_PX));
        let overflows = execute_command_overflows(measured);
        let toggle_label = if open {
            self.tr("收起", "Collapse")
        } else {
            self.tr("展开命令", "Show full command")
        };
        let command_scroll = self
            .tool_output_scroll
            .borrow_mut()
            .entry(format!("{key}:scroll"))
            .or_default()
            .clone();
        let wheel_scroll = command_scroll.clone();
        let keyboard_scroll = command_scroll.clone();
        let toggle_scroll = command_scroll.clone();
        let chevron = Icon::new(Glyph::ChevronDown).size(px(12.));
        let chevron = if open {
            chevron.rotate(gpui::radians(std::f32::consts::PI))
        } else {
            chevron
        };
        div()
            .min_w_0()
            .v_flex()
            .gap(px(4.))
            .font_family(cx.theme().mono_font_family.clone())
            .text_size(font_size)
            .line_height(px(EXECUTE_COMMAND_LINE_PX))
            .child(
                div()
                    .id(format!("{key}:viewport"))
                    .min_w_0()
                    .mr(px(-12.))
                    .pr(px(12.))
                    .max_h(execute_command_limit(open))
                    .track_scroll(&command_scroll)
                    .when(open, |body| body.overflow_y_scroll().focusable())
                    .when(!open, |body| body.overflow_hidden())
                    .on_scroll_wheel(move |event: &gpui::ScrollWheelEvent, window, cx| {
                        let delta_y = event.delta.pixel_delta(window.line_height()).y / px(1.);
                        let offset = -wheel_scroll.offset().y / px(1.);
                        let maximum = wheel_scroll.max_offset().y / px(1.);
                        if crate::transcript_scroll::nested_wheel_captures(delta_y, offset, maximum)
                        {
                            cx.stop_propagation();
                        }
                    })
                    .on_key_down(move |event: &gpui::KeyDownEvent, window, cx| {
                        let modifiers = event.keystroke.modifiers;
                        if !open || modifiers.control || modifiers.platform || modifiers.alt {
                            return;
                        }
                        let offset = keyboard_scroll.offset();
                        if let Some((top, _)) = crate::transcript_scroll::keyboard_target(
                            &event.keystroke.key,
                            modifiers.shift,
                            -offset.y / px(1.),
                            keyboard_scroll.max_offset().y / px(1.),
                            keyboard_scroll.bounds().size.height / px(1.),
                        ) {
                            keyboard_scroll.set_offset(point(offset.x, px(-top)));
                            cx.stop_propagation();
                            window.refresh();
                        }
                    })
                    .child(
                        div()
                            .h_flex()
                            .items_start()
                            .gap(px(8.))
                            .min_w_0()
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .text_color(rgba(c.foreground_subtle))
                                    .child("$"),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .text_color(rgb(c.text_primary))
                                    .child(gpui_kit::base::SelectableText::new(
                                        format!("{key}:text"),
                                        sample,
                                    )),
                            ),
                    ),
            )
            .when(overflows || open, |block| {
                block.child(
                    div().pl(ch_width + px(8.)).child(
                        Button::new(format!("execute-command-toggle-{tool_id}"))
                            .text()
                            .h_auto()
                            .px_0()
                            .font_family(cx.theme().font_family.clone())
                            .accessibility_label(toggle_label)
                            .child(
                                div()
                                    .h_flex()
                                    .items_center()
                                    .gap_1()
                                    .text_size(self.ui_text(12.))
                                    .font_weight(gpui::FontWeight::NORMAL)
                                    .text_color(rgba(c.foreground_subtle))
                                    .child(toggle_label)
                                    .child(chevron),
                            )
                            .on_click(cx.listener(move |this, _, _, cx| {
                                toggle_scroll.set_offset(point(px(0.), px(0.)));
                                this.toggle_tool_expansion(&key);
                                cx.notify();
                            })),
                    ),
                )
            })
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
            .when(empty, |dock| dock.px_4())
            .when(!empty, |dock| dock.pb_4())
            .v_flex()
            .items_center()
            .child(
                div()
                    .w_full()
                    .max_w(target_width)
                    // An existing turn column keeps the dock's 16px inset inside
                    // the shared content width. The draft timeline already insets
                    // the whole column, so its shell is not padded twice.
                    .when(!empty, |column| column.px_4())
                    .v_flex()
                    .rounded(px(16.))
                    .when(empty, |card| {
                        card.bg(if Theme::global(cx).is_dark() {
                            rgb(0x222222)
                        } else {
                            rgba(c.selected)
                        })
                        .shadow_xs()
                        .child(
                            div().px_3().py(px(6.)).h_flex().items_center().child(
                                Button::new("composer-workspace")
                                    .ghost()
                                    .small()
                                    .h(px(28.))
                                    .rounded(px(8.))
                                    .px_2()
                                    .child(
                                        div()
                                            .h_flex()
                                            .items_center()
                                            .gap_1()
                                            .child(
                                                Icon::new(Glyph::Folder)
                                                    .size(px(14.))
                                                    .text_color(rgba(c.foreground_subtle)),
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(13.))
                                                    .text_color(rgba(c.foreground_subtle))
                                                    .child(self.tr("选择项目", "Select Project")),
                                            )
                                            .child(
                                                Icon::new(Glyph::ChevronDown)
                                                    .size(px(12.))
                                                    .text_color(rgba(c.foreground_subtlest)),
                                            ),
                                    )
                                    .tooltip(self.workspace_dir.display().to_string())
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.toggle_review(window, cx)
                                     })),
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
                                    .min_h(px(28.))
                                    .max_h(px(160.))
                                    .text_size(self.ui_text(14.))
                                    .line_height(self.ui_text(20.)),
                            )
                            .child(
                                div()
                                    .h_flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .h_flex()
                                            .items_center()
                                            .gap(px(6.))
                                            .child(
                                                Button::new("composer-actions")
                                                    .ghost()
                                                    .w(px(26.))
                                                    .h(px(26.))
                                                    .rounded(px(8.))
                                                    .icon(Glyph::Plus)
                                                    .accessibility_label(self.tr("更多操作", "More actions"))
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
                                                                    move |_, window, cx| {
                                                                        let _ = files_app.update(
                                                                            cx,
                                                                            |this, cx| {
                                                                                this.toggle_review(window, cx)
                                                                            },
                                                                        );
                                                                    },
                                                                ),
                                                            )
                                                        }
                                                    }),
                                            )
                                            .child(
                                                Button::new("composer-approval")
                                                    .ghost()
                                                    .small()
                                                    .h(px(28.))
                                                    .rounded(px(8.))
                                                    .px_2()
                                                    .child(
                                                        div()
                                                            .h_flex()
                                                            .items_center()
                                                            .gap_1()
                                                            .child(
                                                                Icon::new(Glyph::Hand)
                                                                    .size(px(14.))
                                                                    .text_color(rgba(c.foreground_subtle)),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_size(px(13.))
                                                                    .text_color(rgba(c.foreground_subtle))
                                                                    .child(self.tr("变更前确认", "Confirm before edit")),
                                                            )
                                                            .child(
                                                                Icon::new(Glyph::ChevronDown)
                                                                    .size(px(12.))
                                                                    .text_color(rgba(c.foreground_subtlest)),
                                                            ),
                                                    )
                                                    .tooltip(self.tr("变更前确认", "Confirm before edit")),
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
                                                                div()
                                                                    .min_w_0()
                                                                    .truncate()
                                                                    .text_size(px(13.))
                                                                    .text_color(rgba(c.foreground_subtle))
                                                                    .child(if empty && self.model_identifier.is_empty() {
                                                                        self.tr("管理模型", "Manage Models").to_string()
                                                                    } else {
                                                                        self.current_model()
                                                                    }),
                                                            )
                                                            .child(
                                                                Icon::new(Glyph::ChevronDown)
                                                                    .size(px(12.))
                                                                    .flex_shrink_0()
                                                                    .text_color(rgba(
                                                                        c.foreground_subtlest,
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
                                                Button::new("stop")
                                                    .secondary()
                                                    .w(px(32.))
                                                    .h(px(32.))
                                                    .rounded(px(8.))
                                                    .icon(Glyph::Square)
                                                    .accessibility_label(self.tr("停止", "Stop"))
                                                    .on_click(cx.listener(|this, _, _, cx| {
                                                        this.stop_run(cx)
                                                    }))
                                            } else {
                                                let dark = Theme::global(cx).is_dark();
                                                let brand_btn = if dark {
                                                    ButtonCustomVariant::new(cx)
                                                        .color(rgb(0xffffff).into())
                                                        .foreground(rgb(0x000000).into())
                                                        .hover(rgba(0xffffffcc).into())
                                                        .active(rgba(0xffffffe6).into())
                                                } else {
                                                    ButtonCustomVariant::new(cx)
                                                        .color(rgb(0x000000).into())
                                                        .foreground(rgb(0xffffff).into())
                                                        .hover(rgba(0x000000cc).into())
                                                        .active(rgba(0x000000e6).into())
                                                };
                                                Button::new("send")
                                                    .custom(brand_btn)
                                                    .w(px(26.))
                                                    .h(px(26.))
                                                    .rounded(px(8.))
                                                    .icon(Glyph::ArrowUp)
                                                    .accessibility_label(self.t().send_btn)
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

    fn draft_suggested_prompts(
        &self,
        content_width: Pixels,
        c: &ThemeColors,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        if !self.show_quick_actions {
            return None;
        }
        let target_width = content_width.min(px(672.));
        Some(
            div()
                .id("draft-suggested-prompts")
                .w_full()
                .mt(px(24.))
                .px_4()
                .h_flex()
                .justify_center()
                .child(
                    div()
                        .w_full()
                        .max_w(target_width)
                        .h_flex()
                        .items_center()
                        .justify_center()
                        .flex_wrap()
                        .gap(px(8.))
                        .children(DRAFT_SUGGESTED_PROMPTS.iter().map(|item| {
                            let label = self.tr(item.label_zh, item.label_en);
                            let prompt = self.tr(item.prompt_zh, item.prompt_en).to_string();
                            Button::new(format!("suggested-prompt-{}", item.id))
                                .outline()
                                .h(px(32.))
                                .rounded(px(16.))
                                .px_3()
                                .disabled(self.is_executing)
                                .tooltip(prompt.clone())
                                .child(
                                    div()
                                        .h_flex()
                                        .items_center()
                                        .gap(px(6.))
                                        .child(
                                            Icon::new(item.glyph)
                                                .size(px(16.))
                                                .text_color(rgba(c.foreground_subtle)),
                                        )
                                        .child(
                                            div()
                                                .max_w(px(256.))
                                                .truncate()
                                                .text_size(self.ui_text(14.))
                                                .font_normal()
                                                .text_color(rgba(c.foreground_subtle))
                                                .child(label),
                                        ),
                                )
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.input_state.update(cx, |input, cx| {
                                        input.set_value(prompt.clone(), window, cx);
                                    });
                                    this.drafts.insert(this.session.id.clone(), prompt.clone());
                                    this.persist_active_session();
                                    this.focus_composer(window, cx);
                                }))
                        })),
                )
                .into_any_element(),
        )
    }

    pub(crate) fn goal_banner(
        &self,
        c: &ThemeColors,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let state = self.task_states.get(&self.session.id)?;
        let goal = state.goal.as_ref()?;
        Some(
            div()
                .id("conversation-goal-banner")
                .w_full()
                .h_flex()
                .items_center()
                .gap_2()
                .px_3()
                .py(px(6.))
                .border_b_1()
                .border_color(rgba(c.dock_border))
                .bg(rgb(c.card_bg))
                .text_size(self.ui_text(12.))
                .child(
                    div()
                        .text_color(rgba(c.foreground_subtle))
                        .child(self.tr("目标", "Goal")),
                )
                .child(
                    div()
                        .min_w_0()
                        .flex_1()
                        .truncate()
                        .text_color(rgb(c.text_primary))
                        .child(goal.objective.clone()),
                )
                .child(
                    div()
                        .flex_shrink_0()
                        .rounded(px(4.))
                        .border_1()
                        .border_color(rgba(c.dock_border))
                        .px(px(6.))
                        .py(px(2.))
                        .text_color(rgba(c.foreground_subtle))
                        .child(format!("{} · #{}", goal.status, goal.iteration)),
                )
                .child(
                    icon_button("goal-clear", self.tr("清除目标", "Clear goal"), Glyph::X)
                        .on_click(cx.listener(|this, _, _, cx| {
                            if let Some(state) = this.task_states.get_mut(&this.session.id) {
                                state.goal = None;
                            }
                            this.persist_active_session();
                            cx.notify();
                        })),
                )
                .into_any_element(),
        )
    }

    pub(crate) fn draft_onboard_banner(
        &self,
        content_width: Pixels,
        c: &ThemeColors,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        if !self.show_onboard_banner {
            return None;
        }
        let target_width = content_width.min(px(672.));
        Some(
            div()
                .id("draft-onboard-notice")
                .w_full()
                .h_flex()
                .justify_center()
                .child(
                    div()
                        .w_full()
                        .max_w(target_width)
                        .mb(px(24.))
                        .px_4()
                        .py_2()
                        .rounded(px(12.))
                        .border_1()
                        .border_color(if Theme::global(cx).is_dark() {
                            rgb(0x393939)
                        } else {
                            rgba(c.card_border)
                        })
                        .bg(if Theme::global(cx).is_dark() {
                            rgb(0x222222)
                        } else {
                            rgb(c.card_bg)
                        })
                        .h_flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .h_flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    Icon::new(Glyph::Info)
                                        .size(px(14.))
                                        .text_color(rgba(c.foreground_subtle)),
                                )
                                .child(
                                    div()
                                        .text_size(px(13.))
                                        .text_color(rgba(c.foreground_subtle))
                                        .child(self.tr(
                                            "当前没有可用模型。请开通编程套餐或配置自定义模型。",
                                            "No model available. Please subscribe or configure custom models.",
                                        )),
                                ),
                        )
                        .child(
                            div()
                                .h_flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    Button::new("onboard-upgrade")
                                        .ghost()
                                        .small()
                                        .h(px(22.))
                                        .rounded(px(11.))
                                        .px(px(10.))
                                        .bg(rgb(0x494a58))
                                        .child(
                                            div()
                                                .h_flex()
                                                .items_center()
                                                .gap_1()
                                                .child(Icon::new(Glyph::Rocket).size(px(12.)).text_color(rgb(0xffffff)))
                                                .child(
                                                    div()
                                                        .text_size(px(12.))
                                                        .font_medium()
                                                        .text_color(rgb(0xffffff))
                                                        .child(self.tr("升级", "Upgrade")),
                                                ),
                                        ),
                                )
                                .child(
                                    Button::new("onboard-settings")
                                        .ghost()
                                        .small()
                                        .h(px(24.))
                                        .rounded(px(6.))
                                        .border_1()
                                        .border_color(if Theme::global(cx).is_dark() {
                                            rgb(0x393939)
                                        } else {
                                            rgba(c.card_border)
                                        })
                                        .px_2()
                                        .child(
                                            div()
                                                .h_flex()
                                                .items_center()
                                                .gap_1()
                                                .child(Icon::new(Glyph::Settings).size(px(12.)))
                                                .child(
                                                    div()
                                                        .text_size(px(12.))
                                                        .child(self.tr("配置", "Configure")),
                                                ),
                                        )
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.show_settings = true;
                                            cx.notify();
                                        })),
                                )
                                .child(
                                    Button::new("onboard-dismiss")
                                        .ghost()
                                        .small()
                                        .h(px(24.))
                                        .w(px(24.))
                                        .p_0()
                                        .icon(Glyph::X)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.show_onboard_banner = false;
                                            cx.notify();
                                        })),
                                ),
                        ),
                )
                .into_any_element(),
        )
    }

    pub(crate) fn pending_recovery_banner(
        &self,
        content_width: Pixels,
        c: &ThemeColors,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let state = self.task_states.get(&self.session.id)?;
        let pending_text = state.pending_recovery.clone()?;
        let target_width = conversation_composer_width(content_width);
        let retry_text = pending_text.clone();
        Some(
            div()
                .id("pending-command-recovery-banner")
                .w_full()
                .h_flex()
                .justify_center()
                .child(
                    div()
                        .w_full()
                        .max_w(target_width)
                        .mb(px(12.))
                        .h_flex()
                        .items_center()
                        .justify_between()
                        .gap_2()
                        .p_3()
                        .rounded(px(12.))
                        .border_1()
                        .border_color(rgba(c.dock_border))
                        .bg(rgb(c.card_bg))
                        .shadow_xs()
                        .child(
                            div()
                                .h_flex()
                                .items_center()
                                .gap_2()
                                .min_w_0()
                                .flex_1()
                                .child(
                                    Icon::new(Glyph::CircleAlert)
                                        .size(px(16.))
                                        .text_color(rgba(c.foreground_subtle)),
                                )
                                .child(
                                    div()
                                        .min_w_0()
                                        .truncate()
                                        .text_size(self.ui_text(14.))
                                        .text_color(rgb(c.text_primary))
                                        .child(self.tr(
                                            "CLI 重启前已提交的输入没有进入对话",
                                            "A command submitted before the CLI restarted did not enter the conversation.",
                                        )),
                                ),
                        )
                        .child(
                            div()
                                .h_flex()
                                .items_center()
                                .gap_2()
                                .flex_shrink_0()
                                .child(
                                    Button::new("resend-pending-command")
                                        .outline()
                                        .small()
                                        .h(px(28.))
                                        .rounded(px(8.))
                                        .child(
                                            div()
                                                .h_flex()
                                                .items_center()
                                                .gap_1()
                                                .child(Icon::new(Glyph::RefreshCw).size(px(14.)))
                                                .child(self.tr("重新发送", "Resend")),
                                        )
                                        .on_click(cx.listener(move |this, _, window, cx| {
                                            if let Some(state) = this.task_states.get_mut(&this.session.id) {
                                                state.pending_recovery = None;
                                            }
                                            this.input_state.update(cx, |input, cx| {
                                                input.set_value(retry_text.clone(), window, cx);
                                            });
                                            this.drafts.insert(this.session.id.clone(), retry_text.clone());
                                            this.persist_active_session();
                                            this.focus_composer(window, cx);
                                        })),
                                )
                                .child(
                                    Button::new("dismiss-pending-command")
                                        .ghost()
                                        .small()
                                        .h(px(28.))
                                        .rounded(px(8.))
                                        .child(self.tr("忽略", "Dismiss"))
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            if let Some(state) = this.task_states.get_mut(&this.session.id) {
                                                state.pending_recovery = None;
                                            }
                                            this.persist_active_session();
                                            cx.notify();
                                        })),
                                ),
                        ),
                )
                .into_any_element(),
        )
    }

    pub(crate) fn file_summary_panel(
        &self,
        message_id: &str,
        changes: &crate::task_state::TurnFileChanges,
        c: &ThemeColors,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let key = crate::file_summary::disclosure_key(&self.session.id, message_id);
        let open = self.expanded_tools.contains(&key);
        let focus = window
            .use_keyed_state(
                ElementId::Name(format!("{key}:focus").into()),
                cx,
                |_, cx| cx.focus_handle().tab_stop(true),
            )
            .read(cx)
            .clone();
        let count = changes.files_count();
        let focus_border: Hsla = rgba(c.dock_border_active).into();
        let focus_surface = rgb(c.card_bg);
        let additions = changes.total_additions();
        let deletions = changes.total_deletions();
        let mid_owned = key.clone();

        let count_label = if count == 1 {
            self.tr("1 个文件已修改", "1 file changed").to_string()
        } else {
            format!("{} {}", count, self.tr("个文件已修改", "files changed"))
        };

        div()
            .id(ElementId::Name(
                format!("file-summary-{}", message_id).into(),
            ))
            .w_full()
            .min_w_0()
            .rounded(px(12.))
            .border_1()
            .border_color(rgba(c.settings_border))
            .bg(rgb(c.card_bg))
            .overflow_hidden()
            .child(
                div()
                    .id(ElementId::Name(
                        format!("file-summary-header-{}", message_id).into(),
                    ))
                    .h(px(40.))
                    .px_2()
                    .h_flex()
                    .items_center()
                    .gap_3()
                    .hover(|s| s.bg(rgba(c.selected)))
                    .child(
                        div()
                            .id(ElementId::Name(format!("{key}:trigger").into()))
                            .track_focus(&focus)
                            .role(Role::Button)
                            .aria_label(self.tr("展开或收起文件摘要", "Toggle changed files"))
                            .aria_expanded(open)
                            .h_full()
                            .flex_1()
                            .px_1()
                            .h_flex()
                            .items_center()
                            .gap_2()
                            .min_w_0()
                            .cursor_pointer()
                            .focus_visible(move |style| {
                                style.bg(focus_surface).shadow(vec![BoxShadow::new(
                                    px(0.),
                                    px(0.),
                                    focus_border,
                                )
                                .blur_radius(px(0.))
                                .spread_radius(px(2.))])
                            })
                            .on_click(cx.listener({
                                let key = key.clone();
                                move |this, _, _, cx| {
                                    this.toggle_tool_expansion(&key);
                                    cx.stop_propagation();
                                    cx.notify();
                                }
                            }))
                            .child(
                                Icon::new(if open {
                                    Glyph::ChevronDown
                                } else {
                                    Glyph::ChevronRight
                                })
                                .size(px(14.))
                                .text_color(rgba(c.foreground_subtle)),
                            )
                            .child(
                                div()
                                    .font_medium()
                                    .text_size(self.ui_text(14.))
                                    .text_color(rgb(c.text_primary))
                                    .truncate()
                                    .child(count_label),
                            )
                            .when_some(
                                additions.zip(deletions),
                                |trigger, (additions, deletions)| {
                                    trigger.child(
                                        div()
                                            .h_flex()
                                            .items_center()
                                            .gap_1()
                                            .flex_shrink_0()
                                            .font_family(cx.theme().mono_font_family.clone())
                                            .text_size(self.ui_text(14.))
                                            .child(
                                                div()
                                                    .text_color(rgb(c.accent_green))
                                                    .child(format!("+{additions}")),
                                            )
                                            .child(
                                                div()
                                                    .text_color(rgb(c.workflow_error))
                                                    .child(format!("-{deletions}")),
                                            ),
                                    )
                                },
                            ),
                    )
                    .child(
                        div().h_flex().items_center().gap_2().child(
                            Button::new(ElementId::Name(format!("rewind-{}", mid_owned).into()))
                                .ghost()
                                .small()
                                .h(px(28.))
                                .rounded(px(8.))
                                .disabled(true)
                                .accessibility_label(
                                    self.tr("撤销（暂不可用）", "Rewind (unavailable)"),
                                )
                                .child(
                                    div()
                                        .h_flex()
                                        .items_center()
                                        .gap_1()
                                        .child(Icon::new(Glyph::Undo2).size(px(14.)))
                                        .child(self.tr("撤销", "Rewind")),
                                ),
                        ),
                    ),
            )
            .when(open, |panel| {
                let items = changes.items.clone();
                panel.child(
                    div()
                        .border_t_1()
                        .border_color(rgba(c.settings_border))
                        .v_flex()
                        .children(items.into_iter().enumerate().map(|(idx, item)| {
                            let open_path = item.path.clone();
                            let p = std::path::Path::new(&item.path);
                            let file_name = p
                                .file_name()
                                .and_then(|s| s.to_str())
                                .unwrap_or(&item.path)
                                .to_string();
                            let parent_dir = p
                                .parent()
                                .and_then(|s| s.to_str())
                                .unwrap_or("")
                                .to_string();

                            div()
                                .h(px(36.))
                                .px_3()
                                .h_flex()
                                .items_center()
                                .justify_between()
                                .gap_2()
                                .min_w_0()
                                .hover(|s| s.bg(rgba(c.selected)))
                                .child(
                                    div()
                                        .h_flex()
                                        .items_center()
                                        .gap_2()
                                        .flex_1()
                                        .min_w_0()
                                        .child(
                                            Icon::new(Glyph::FileText)
                                                .size(px(14.))
                                                .text_color(rgba(c.foreground_subtle)),
                                        )
                                        .child(
                                            div()
                                                .font_medium()
                                                .text_size(self.ui_text(14.))
                                                .text_color(rgb(c.text_primary))
                                                .min_w_0()
                                                .truncate()
                                                .child(file_name),
                                        )
                                        .when(!parent_dir.is_empty(), |d| {
                                            d.child(
                                                div()
                                                    .min_w_0()
                                                    .truncate()
                                                    .text_size(self.ui_text(14.))
                                                    .text_color(rgba(c.foreground_subtle))
                                                    .child(parent_dir),
                                            )
                                        })
                                        .child(
                                            div()
                                                .h_flex()
                                                .items_center()
                                                .gap_1()
                                                .flex_shrink_0()
                                                .font_family(cx.theme().mono_font_family.clone())
                                                .text_size(self.ui_text(14.))
                                                .when_some(
                                                    item.additions.filter(|count| *count > 0),
                                                    |d, count| {
                                                        d.child(
                                                            div()
                                                                .text_color(rgb(c.accent_green))
                                                                .child(format!("+{count}")),
                                                        )
                                                    },
                                                )
                                                .when_some(
                                                    item.deletions.filter(|count| *count > 0),
                                                    |d, count| {
                                                        d.child(
                                                            div()
                                                                .text_color(rgb(c.workflow_error))
                                                                .child(format!("-{count}")),
                                                        )
                                                    },
                                                ),
                                        ),
                                )
                                .child(
                                    Button::new(ElementId::Name(
                                        format!("review-file-{}-{}", mid_owned, idx).into(),
                                    ))
                                    .outline()
                                    .small()
                                    .h(px(28.))
                                    .rounded(px(8.))
                                    .flex_shrink_0()
                                    .accessibility_label(format!(
                                        "{} {}",
                                        self.tr("审查", "Review"),
                                        open_path
                                    ))
                                    .tooltip(open_path.clone())
                                    .child(self.tr("审查", "Review"))
                                    .on_click(cx.listener({
                                        let target_path = open_path;
                                        move |this, _, window, cx| {
                                            this.open_review_file(target_path.clone(), window, cx);
                                        }
                                    })),
                                )
                        })),
                )
            })
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
            .min_w_0()
            .min_h_0()
            .overflow_y_scroll()
            .v_flex()
            .p_6()
            .gap_4()
            .child(heading(self.t().skills_tools))
            .child(Input::new(&self.skill_search_input))
            .child(
                div()
                    .w_full()
                    .min_w_0()
                    .h_flex()
                    .flex_wrap()
                    .gap_2()
                    .children(
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
                    .w_full()
                    .min_w_0()
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
    pub(crate) fn open_skill_detail(
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
                        .border_color(rgba(c.card_border))
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
                                    .text_color(rgb(c.workflow_error))
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
                        .border_color(rgba(c.card_border))
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
                                    .hover(rgba(c.hover).into())
                                    .active(rgba(c.selected).into())
                            )
                                .w_full().h_full().min_w_0().p_3().rounded(px(12.))
                                .border_1().border_color(rgba(c.card_border))
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
const USER_MESSAGE_COLLAPSED_MAX_HEIGHT: f32 = 120.;
const USER_MESSAGE_OVERFLOW_TOLERANCE: f32 = 1.;

fn user_message_bubble_width(content_width: Pixels) -> Pixels {
    let available = transcript_content_width(content_width);
    if content_width >= USER_MESSAGE_BREAKPOINT {
        available.min(USER_MESSAGE_MAX_WIDTH)
    } else {
        available
    }
}

fn user_message_content_is_expandable(measured_height: Pixels) -> bool {
    measured_height / px(1.) > USER_MESSAGE_COLLAPSED_MAX_HEIGHT + USER_MESSAGE_OVERFLOW_TOLERANCE
}

/// Separates the user's prompt text from an engine-injected epilogue (e.g., quality
/// standards, result JSON schema, or engine instructions).
///
/// Matches ZCode `splitUserInputEpilogue`:
/// - If `epilogue_start` is Some, out-of-bounds yields no epilogue; in-bounds splits.
/// - If `epilogue_start` is None, auto-detects the standard engine rule separator `\n\n---\n`.
pub(crate) fn split_user_input_epilogue<'a>(
    text: &'a str,
    epilogue_start: Option<usize>,
) -> (&'a str, Option<&'a str>) {
    let start = match epilogue_start {
        Some(pos) => {
            if pos > text.len() {
                return (text, None);
            }
            pos
        }
        None => {
            if let Some(pos) = text.find("\n\n---\nStandard for this result:") {
                pos
            } else if let Some(pos) = text.find("\n\n---\nWorkflow engine instructions") {
                pos
            } else if let Some(pos) = text.find("\n\n---\n") {
                pos
            } else {
                return (text, None);
            }
        }
    };
    let body = &text[..start];
    let epilogue = &text[start..];
    (body, Some(epilogue))
}

/// Trims leading whitespace and the leading `---` separator line from the epilogue,
/// matching ZCode's `trimEpilogueLead`.
pub(crate) fn trim_epilogue_lead(text: &str) -> &str {
    let mut trimmed = text.trim_start();
    if let Some(rest) = trimmed.strip_prefix("---") {
        let rest = rest.trim_start_matches(|c: char| c == ' ' || c == '\t');
        if let Some(after_newline) = rest.strip_prefix('\n') {
            trimmed = after_newline;
        } else if let Some(after_crlf) = rest.strip_prefix("\r\n") {
            trimmed = after_crlf;
        }
    }
    trimmed.trim_end()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SlashCommand {
    Compact,
    SendGoal { objective: String },
    ResumeGoal,
    ClearGoal,
    EmptyGoal,
}

pub(crate) fn parse_slash_command(input: &str) -> Option<SlashCommand> {
    let trimmed = input.trim();
    if !trimmed.starts_with('/') {
        return None;
    }
    let mut parts = trimmed.splitn(2, char::is_whitespace);
    let cmd = parts.next()?.trim_start_matches('/').to_lowercase();
    let rest = parts.next().map(str::trim).unwrap_or("");

    match cmd.as_str() {
        "compact" | "compress" => Some(SlashCommand::Compact),
        "goal" | "target" => {
            if rest.is_empty() {
                Some(SlashCommand::EmptyGoal)
            } else if rest.eq_ignore_ascii_case("resume") {
                Some(SlashCommand::ResumeGoal)
            } else if rest.eq_ignore_ascii_case("clear") || rest.eq_ignore_ascii_case("pause") {
                Some(SlashCommand::ClearGoal)
            } else {
                let objective = if let Some(stripped) = rest.strip_prefix("replace ") {
                    stripped.trim()
                } else {
                    rest
                };
                if objective.is_empty() {
                    Some(SlashCommand::EmptyGoal)
                } else {
                    Some(SlashCommand::SendGoal {
                        objective: objective.to_string(),
                    })
                }
            }
        }
        _ => None,
    }
}

struct ToolOutputChrome {
    pad_x: Pixels,
    pad_y: Pixels,
    radius: Pixels,
    before: Pixels,
    after: Pixels,
    bordered: bool,
}

/// Expanded tool output. Terminal tools use the execute panel; everything else
/// uses the fallback surface. The content shell's 8px top padding is included.
fn tool_output_chrome(name: &str) -> ToolOutputChrome {
    let terminal = matches!(name, "bash" | "execute" | "shell");
    ToolOutputChrome {
        pad_x: px(16.),
        pad_y: px(12.),
        radius: px(12.),
        before: px(if terminal { 8. } else { 12. }),
        after: px(if terminal { 8. } else { 0. }),
        bordered: terminal,
    }
}

fn raw_tool_output_presentation(name: &str) -> (Pixels, Pixels) {
    // Default 14px interface size. Rendered text uses the live interface token.
    let terminal = matches!(name, "bash" | "execute" | "shell");
    (
        raw_tool_output_font_size(name, 14.),
        px(if terminal { 100. } else { 200. }),
    )
}

/// Terminal output is `text-ui-base`. Fallback raw output is `text-ui-xs`.
fn raw_tool_output_font_size(name: &str, interface_px: f32) -> Pixels {
    if matches!(name, "bash" | "execute" | "shell") {
        px(interface_px)
    } else {
        px((interface_px - 4.).max(0.))
    }
}

fn terminal_summary_hides_detail(name: &str, expanded: bool, has_command: bool) -> bool {
    expanded && has_command && matches!(name, "bash" | "execute" | "shell")
}

fn execute_command_key(session_id: &str, tool_id: &str) -> String {
    format!(
        "execute-command:{}:{session_id}:{}:{tool_id}",
        session_id.len(),
        tool_id.len()
    )
}

fn execute_command_text_width(row_width: Pixels, prompt_width: Pixels) -> Pixels {
    // Panel padding and border (34px), then the prompt and its 8px gap.
    (row_width - px(34.) - px(8.) - prompt_width).max(px(1.))
}

const EXECUTE_COMMAND_LINE_PX: f32 = 20.;

/// Collapsed command is `max-h-[3lh]`; the open body is `max-h-[10lh]`.
fn execute_command_limit(expanded: bool) -> Pixels {
    px(EXECUTE_COMMAND_LINE_PX * if expanded { 10. } else { 3. })
}

/// The reference shows the toggle once the collapsed body overflows by more than 1px.
fn execute_command_overflows(measured_height: Pixels) -> bool {
    measured_height.as_f32() > EXECUTE_COMMAND_LINE_PX * 3. + 1.
}

/// `zcode-stream-text-in`: 900ms, cubic-bezier(0.16, 1, 0.3, 1). The stylesheet
/// leaves the per-token delay at 0, so one update fades together.
fn assistant_stream_motion() -> gpui_kit::base::TextViewMotion {
    gpui_kit::base::TextViewMotion::default()
        .with_stream_fade(std::time::Duration::from_millis(900))
        .with_stream_fade_easing(
            gpui_kit::base::Easing::cubic_bezier(0.16, 1.0, 0.3, 1.0).expect("stream fade curve"),
        )
}

fn assistant_markdown(
    font_size: f32,
    id: impl Into<gpui::ElementId>,
    markdown: impl Into<SharedString>,
    colors: &ThemeColors,
    fade: bool,
    cx: &gpui::App,
) -> gpui_kit::base::TextView {
    let mut view = gpui_kit::base::TextView::markdown(id, markdown)
        .selectable(true)
        .text_size(px(font_size))
        .line_height(px(font_size * 1.75))
        .style(assistant_text_style(font_size, colors, cx));
    if fade {
        view = view.motion(assistant_stream_motion());
    }
    view
}

/// ZCode `rounded-xl` on markdown tables and fenced code.
const ASSISTANT_TABLE_RADIUS: Pixels = px(12.);
const ASSISTANT_CODE_RADIUS: Pixels = px(12.);
/// ZCode code blocks default to 14px monospace. The renderer otherwise uses 13px.
const ASSISTANT_CODE_FONT: Pixels = px(14.);
/// ZCode `py-2`. Horizontal cell padding stays the renderer's 12px `px-3`.
const ASSISTANT_TABLE_CELL_PADDING_Y: Pixels = px(8.);
/// Streamdown `space-y-4`. One rem is 16px at the default root size.
const ASSISTANT_BLOCK_GAP_REM: f32 = 1.;
/// Added above a heading so the previous 16px block gap plus this padding is `mt-6`.
const ASSISTANT_HEADING_PADDING_TOP: f32 = 8.;
/// ZCode heading `mb-4`.
const ASSISTANT_HEADING_PADDING_BOTTOM: f32 = 16.;

/// Theme-backed rich text, with the reference heading, table, and link colors.
fn assistant_text_style(
    font_size: f32,
    colors: &ThemeColors,
    cx: &gpui::App,
) -> gpui_kit::base::TextViewStyle {
    let theme = Theme::global(cx);
    let code_radius = ASSISTANT_CODE_RADIUS.into();
    let mut code_block = gpui::StyleRefinement::default().text_size(ASSISTANT_CODE_FONT);
    code_block.corner_radii.top_left = Some(code_radius);
    code_block.corner_radii.top_right = Some(code_radius);
    code_block.corner_radii.bottom_left = Some(code_radius);
    code_block.corner_radii.bottom_right = Some(code_radius);
    // The frame and header both start from other surfaces: the frame falls
    // back to the theme surface, and the header is painted with the code
    // background before this refinement. Pin both to the chat surface so the
    // header does not read as a filled band. Fenced code keeps the card fill.
    let surface = rgb(colors.root_bg);
    let table_radius = ASSISTANT_TABLE_RADIUS.into();
    let mut table = gpui::StyleRefinement::default().bg(surface);
    table.corner_radii.top_left = Some(table_radius);
    table.corner_radii.top_right = Some(table_radius);
    table.corner_radii.bottom_left = Some(table_radius);
    table.corner_radii.bottom_right = Some(table_radius);
    let table_head = gpui::StyleRefinement::default()
        .bg(surface)
        .text_color(rgba(colors.foreground_subtlest))
        .font_weight(assistant_table_header_weight());
    let table_cell = gpui::StyleRefinement::default()
        .py(ASSISTANT_TABLE_CELL_PADDING_Y)
        .self_start();
    let heading_color = theme.foreground;
    gpui_kit::base::TextViewStyle::default()
        .with_foreground(theme.foreground)
        .with_muted_foreground(theme.muted_foreground)
        .with_link(rgb(colors.link).into())
        .with_selection(theme.selection)
        .with_code_background(rgb(colors.card_bg).into())
        .with_border(theme.border)
        .with_paragraph_gap(gpui::rems(ASSISTANT_BLOCK_GAP_REM))
        .with_code_block(code_block)
        .with_table(table)
        .with_table_head(table_head)
        .with_table_cell(table_cell)
        .with_inline_code(gpui::HighlightStyle {
            background_color: Some(rgba(colors.inline_code_bg).into()),
            ..Default::default()
        })
        .with_dark(theme.is_dark())
        .with_heading(move |level| {
            gpui::StyleRefinement::default()
                .text_size(px(assistant_heading_size(level, font_size)))
                .font_weight(assistant_heading_weight(level))
                .line_height(gpui::relative(1.75))
                .pt(px(ASSISTANT_HEADING_PADDING_TOP))
                .pb(px(ASSISTANT_HEADING_PADDING_BOTTOM))
                .text_color(heading_color)
        })
}

/// Reference heading tokens add to the interface font. The markdown renderer otherwise scales level 1 by 1.8.
fn assistant_heading_size(level: u8, font_size: f32) -> f32 {
    match level {
        1 => font_size + 4.,
        2 => font_size + 2.,
        _ => font_size,
    }
}

/// Header text is regular. Body cells keep the surrounding text weight.
fn assistant_table_header_weight() -> gpui::FontWeight {
    gpui::FontWeight::NORMAL
}

/// Levels 1–4 are semibold, level 5 is medium, and level 6 is regular.
fn assistant_heading_weight(level: u8) -> gpui::FontWeight {
    match level {
        6 => gpui::FontWeight::NORMAL,
        5 => gpui::FontWeight::MEDIUM,
        _ => gpui::FontWeight::SEMIBOLD,
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

/// Local hour bands from the draft empty state. This app has no office-mode override.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GreetingSlot {
    MorningEarly,
    Morning,
    Noon,
    Afternoon,
    Evening,
    LateNight,
}

fn greeting_slot(hour: u32) -> GreetingSlot {
    match hour {
        5..=8 => GreetingSlot::MorningEarly,
        9..=11 => GreetingSlot::Morning,
        12..=13 => GreetingSlot::Noon,
        14..=17 => GreetingSlot::Afternoon,
        18..=22 => GreetingSlot::Evening,
        _ => GreetingSlot::LateNight,
    }
}

fn empty_greeting(hour: u32) -> (&'static str, &'static str) {
    match greeting_slot(hour) {
        GreetingSlot::MorningEarly => ("早上好呀，新的一天开始啦", "Morning, ready when you are"),
        GreetingSlot::Morning => ("上午好呀，有什么想让我帮忙的吗", "Morning, how can I help?"),
        GreetingSlot::Noon => ("中午好呀，要不要先休息一下", "Noon break?"),
        GreetingSlot::Afternoon => (
            "下午好呀，接下来交给我吧",
            "Good afternoon! Leave the rest to me.",
        ),
        GreetingSlot::Evening => ("晚上好呀，今天辛苦啦", "Evening, nice work today"),
        GreetingSlot::LateNight => (
            "夜深啦，别忘了照顾好自己哦",
            "It's late—remember to take care of yourself.",
        ),
    }
}

/// Milliseconds until the next 05:00, 09:00, 12:00, 14:00, 18:00, or 23:00.
fn next_greeting_delay_ms(hour: u32, minute: u32, second: u32, millis: u32) -> u64 {
    const BOUNDARIES_MS: [u32; 6] = [
        5 * 3_600_000,
        9 * 3_600_000,
        12 * 3_600_000,
        14 * 3_600_000,
        18 * 3_600_000,
        23 * 3_600_000,
    ];
    let now = ((hour * 60 + minute) * 60 + second) * 1000 + millis;
    for boundary in BOUNDARIES_MS {
        if boundary > now {
            return u64::from(boundary - now).max(1);
        }
    }
    u64::from((24 + 5) * 3_600_000 - now).max(1)
}

/// 30px until the measured line no longer fits, then 1px steps down to 20.
fn resolve_greeting_font_size(available_width: f32, natural_width: f32) -> f32 {
    if !available_width.is_finite()
        || !natural_width.is_finite()
        || available_width <= 0.
        || natural_width <= 0.
        || available_width >= natural_width
    {
        30.
    } else {
        (30. * (available_width / natural_width))
            .floor()
            .clamp(20., 30.)
    }
}

/// Draft suggestion chip from ZCode `ConversationDraftSuggestedPrompts`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DraftSuggestedPrompt {
    pub id: &'static str,
    pub label_zh: &'static str,
    pub label_en: &'static str,
    pub prompt_zh: &'static str,
    pub prompt_en: &'static str,
    pub glyph: Glyph,
}

pub const DRAFT_SUGGESTED_PROMPTS: &[DraftSuggestedPrompt] = &[
    DraftSuggestedPrompt {
        id: "weekly-report",
        label_zh: "周报总结",
        label_en: "Weekly report",
        prompt_zh: "帮我总结本周的工作进展，生成结构清晰的周报内容。",
        prompt_en: "Summarize this week's progress and generate a well-structured weekly report.",
        glyph: Glyph::AlarmClock,
    },
    DraftSuggestedPrompt {
        id: "bug-fix",
        label_zh: "报错修复",
        label_en: "Fix errors",
        prompt_zh: "分析当前报错日志或代码异常，指出根因并提供修复方案。",
        prompt_en: "Analyze current error logs or exceptions, identify root causes, and provide fixes.",
        glyph: Glyph::Bug,
    },
    DraftSuggestedPrompt {
        id: "make-ppt",
        label_zh: "PPT 制作",
        label_en: "Make slides",
        prompt_zh: "根据项目主题与核心要点，生成一份结构完整的大纲与 PPT 内容。",
        prompt_en: "Generate a complete presentation outline and slide contents based on the project topic.",
        glyph: Glyph::Presentation,
    },
    DraftSuggestedPrompt {
        id: "idle-tasks",
        label_zh: "闲时任务",
        label_en: "Idle tasks",
        prompt_zh: "利用闲时自动扫描代码坏味道、未使用的依赖和安全漏洞并提出改进意见。",
        prompt_en: "Scan for code smells, unused dependencies, and security vulnerabilities during idle time.",
        glyph: Glyph::Moon,
    },
];

/// Error banner tone follows ZCode's `ChatErrorBanner`: stopped runs stay neutral,
/// while provider and runtime errors use destructive treatment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErrorBannerTone {
    Stopped,
    Failure,
}

pub(crate) fn error_banner_tone(stopped: bool) -> ErrorBannerTone {
    if stopped {
        ErrorBannerTone::Stopped
    } else {
        ErrorBannerTone::Failure
    }
}

fn local_clock() -> (u32, u32, u32, u32) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs() as i64;
    let millis = now.subsec_millis();
    let (hour, minute, second) = local_hms(secs).unwrap_or_else(|| {
        let day = secs.rem_euclid(86_400) as u32;
        (day / 3600, (day % 3600) / 60, day % 60)
    });
    (hour, minute, second, millis)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CivilTime {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
}

fn local_hms(secs: i64) -> Option<(u32, u32, u32)> {
    local_civil(secs).map(|time| (time.hour, time.minute, time.second))
}

fn local_civil(secs: i64) -> Option<CivilTime> {
    #[repr(C)]
    struct LibcTm {
        tm_sec: i32,
        tm_min: i32,
        tm_hour: i32,
        tm_mday: i32,
        tm_mon: i32,
        tm_year: i32,
        tm_wday: i32,
        tm_yday: i32,
        tm_isdst: i32,
        tm_gmtoff: i64,
        tm_zone: *const std::ffi::c_char,
    }
    extern "C" {
        fn localtime_r(time: *const i64, result: *mut LibcTm) -> *mut LibcTm;
    }
    unsafe {
        let mut parts = std::mem::zeroed::<LibcTm>();
        if localtime_r(&secs, &mut parts).is_null() {
            return None;
        }
        Some(CivilTime {
            year: parts.tm_year + 1900,
            month: parts.tm_mon as u32 + 1,
            day: parts.tm_mday as u32,
            hour: parts.tm_hour as u32,
            minute: parts.tm_min as u32,
            second: parts.tm_sec as u32,
        })
    }
}

/// Days since Unix epoch. Used to tell yesterday from a same-year date.
fn civil_day_number(time: CivilTime) -> i64 {
    let mut year = time.year;
    let month = time.month;
    if month <= 2 {
        year -= 1;
    }
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = (year - era * 400) as u64;
    let month_prime = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + time.day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year as u64;
    era as i64 * 146097 + day_of_era as i64 - 719468
}

fn format_clock(hour: u32, minute: u32, chinese: bool) -> String {
    if chinese {
        return format!("{hour:02}:{minute:02}");
    }
    let period = if hour < 12 { "AM" } else { "PM" };
    let hour12 = match hour % 12 {
        0 => 12,
        hour => hour,
    };
    format!("{hour12:02}:{minute:02} {period}")
}

/// ZCode message time: clock today, yesterday plus clock, otherwise the locale date.
fn format_message_time(message: CivilTime, now: CivilTime, chinese: bool) -> String {
    let clock = format_clock(message.hour, message.minute, chinese);
    let message_day = civil_day_number(message);
    let now_day = civil_day_number(now);
    if message_day == now_day {
        return clock;
    }
    if message_day + 1 == now_day {
        return if chinese {
            format!("昨天 {clock}")
        } else {
            format!("Yesterday {clock}")
        };
    }
    if message.year == now.year {
        return if chinese {
            format!("{}/{} {clock}", message.month, message.day)
        } else {
            format!("{}/{}, {clock}", message.month, message.day)
        };
    }
    if chinese {
        format!("{}/{}/{} {clock}", message.year, message.month, message.day)
    } else {
        format!(
            "{}/{}/{}, {clock}",
            message.month, message.day, message.year
        )
    }
}

/// ZCode keeps the action row in layout at opacity 0, then shows it for row hover
/// or for focus inside the action row (`group-hover` / `focus-within`).
fn message_actions_are_revealed(row_hovered: bool, action_focus_within: bool) -> bool {
    row_hovered || action_focus_within
}

/// Tailwind `transition-opacity`: 150ms and cubic-bezier(0.4, 0, 0.2, 1).
const MESSAGE_ACTION_FADE_MS: u64 = 150;
const MESSAGE_ACTION_FADE_EASE: [f32; 4] = [0.4, 0.0, 0.2, 1.0];

fn message_action_fade_ms(reduced_motion: bool) -> u64 {
    if reduced_motion {
        0
    } else {
        MESSAGE_ACTION_FADE_MS
    }
}

fn message_action_opacity(
    id: SharedString,
    revealed: bool,
    reduced_motion: bool,
    window: &mut Window,
    cx: &mut App,
) -> f32 {
    use gpui_kit::base::{transition, Transition};
    transition(
        (id, "opacity"),
        if revealed { 1. } else { 0. },
        Transition::new(std::time::Duration::from_millis(message_action_fade_ms(
            reduced_motion,
        )))
        .ease(gpui_kit::base::animation::cubic_bezier(
            MESSAGE_ACTION_FADE_EASE[0],
            MESSAGE_ACTION_FADE_EASE[1],
            MESSAGE_ACTION_FADE_EASE[2],
            MESSAGE_ACTION_FADE_EASE[3],
        )),
        window,
        cx,
    )
}

/// ZCode hides a disclosure chevron until its trigger is hovered or the body is open.
fn summary_disclosure_is_visible(expanded: bool, hovered: bool) -> bool {
    expanded || hovered
}

fn disclosure_motion(
    id: SharedString,
    revealed: bool,
    expanded: bool,
    reduced_motion: bool,
    duration_ms: u64,
    ease: [f32; 4],
    window: &mut Window,
    cx: &mut App,
) -> (f32, f32) {
    use gpui_kit::base::{transition, Transition};
    let policy = Transition::new(std::time::Duration::from_millis(if reduced_motion {
        0
    } else {
        duration_ms
    }))
    .ease(gpui_kit::base::animation::cubic_bezier(
        ease[0], ease[1], ease[2], ease[3],
    ));
    let opacity = transition(
        (id.clone(), "opacity"),
        if revealed { 1. } else { 0. },
        policy.clone(),
        window,
        cx,
    );
    let radians = transition(
        (id, "angle"),
        if expanded {
            std::f32::consts::FRAC_PI_2
        } else {
            0.
        },
        policy,
        window,
        cx,
    );
    (opacity, radians)
}

/// The streaming line is only on a collapsed run. An open body already shows the text.
fn reasoning_streaming_summary_is_shown(streaming: bool, expanded: bool) -> bool {
    streaming && !expanded
}

struct ReasoningSummaryLine<'a> {
    index: usize,
    text: &'a str,
}

/// Last non-empty reasoning line. ZCode treats both `\r` and `\n` as breaks, then trims.
fn reasoning_streaming_summary_line(text: &str) -> Option<ReasoningSummaryLine<'_>> {
    let bytes = text.as_bytes();
    let mut start = 0;
    let mut cursor = 0;
    let mut line_index = 0;
    let mut latest = None;
    while cursor < bytes.len() {
        let carriage_return = bytes[cursor] == b'\r';
        if !carriage_return && bytes[cursor] != b'\n' {
            cursor += 1;
            continue;
        }
        if let Some(line) = nonempty_trimmed(&text[start..cursor]) {
            latest = Some(ReasoningSummaryLine {
                index: line_index,
                text: line,
            });
        }
        if carriage_return && bytes.get(cursor + 1) == Some(&b'\n') {
            cursor += 1;
        }
        cursor += 1;
        start = cursor;
        line_index += 1;
    }
    if let Some(line) = nonempty_trimmed(&text[start..]) {
        latest = Some(ReasoningSummaryLine {
            index: line_index,
            text: line,
        });
    }
    latest
}

fn nonempty_trimmed(line: &str) -> Option<&str> {
    let line = line.trim();
    if line.is_empty() {
        None
    } else {
        Some(line)
    }
}

/// Summary viewport is the turn content box: the shared column minus `px-4` or `px-6`.
fn reasoning_summary_row_width(content_width: Pixels) -> Pixels {
    transcript_content_width(content_width)
}

/// 16px brain, 16px chevron, and the four 8px gaps around the label, dot, and summary.
const REASONING_SUMMARY_CHROME: Pixels = px(64.);

fn reasoning_summary_text_budget(
    row_width: Pixels,
    label_width: Pixels,
    dot_width: Pixels,
) -> Pixels {
    (row_width - REASONING_SUMMARY_CHROME - label_width - dot_width).max(px(0.))
}

const REASONING_SUMMARY_OVERFLOW_SLOP: Pixels = px(1.);

/// `scrollWidth > clientWidth + 1`, matching the reference before it applies the mask.
fn reasoning_summary_is_overflowing(scroll_width: Pixels, client_width: Pixels) -> bool {
    let scroll_width = scroll_width.as_f32();
    let client_width = client_width.as_f32();
    scroll_width.is_finite()
        && client_width.is_finite()
        && scroll_width > client_width + REASONING_SUMMARY_OVERFLOW_SLOP.as_f32()
}

/// Browser clamp of `scrollLeft = scrollWidth`. The newest glyph stays on the right edge.
fn reasoning_summary_scroll_left(scroll_width: Pixels, client_width: Pixels) -> Pixels {
    let scroll_width = scroll_width.as_f32();
    let client_width = client_width.as_f32();
    if !scroll_width.is_finite() || !client_width.is_finite() {
        return px(0.);
    }
    px((scroll_width - client_width.max(0.)).max(0.))
}

struct ReasoningSummaryWindow {
    text: String,
    outgoing: Option<String>,
    /// Visible width. Content-sized when the in-flow line fits; otherwise the remaining budget.
    client_width: Pixels,
    scroll_left: Pixels,
    outgoing_scroll_left: Pixels,
    masked: bool,
}

/// Full lines in one nowrap viewport. A line wider than the row is scrolled, not dropped.
fn visible_reasoning_summary(
    incoming: &str,
    outgoing: Option<&str>,
    available: Pixels,
    mut measure: impl FnMut(&str) -> Pixels,
) -> Option<ReasoningSummaryWindow> {
    if incoming.is_empty() {
        return None;
    }
    let available = if available.as_f32().is_finite() {
        available.max(px(0.))
    } else {
        px(0.)
    };
    let incoming_width = measure(incoming).max(px(0.));
    let client_width = incoming_width.min(available);
    let scroll_left = reasoning_summary_scroll_left(incoming_width, client_width);
    let mut masked = reasoning_summary_is_overflowing(incoming_width, client_width);
    let (outgoing, outgoing_scroll_left) = match outgoing {
        Some(line) if !line.is_empty() => {
            let width = measure(line).max(px(0.));
            masked |= reasoning_summary_is_overflowing(width, client_width);
            (
                Some(line.to_owned()),
                reasoning_summary_scroll_left(width, client_width),
            )
        }
        _ => (None, px(0.)),
    };
    Some(ReasoningSummaryWindow {
        text: incoming.to_owned(),
        outgoing,
        client_width,
        scroll_left,
        outgoing_scroll_left,
        masked,
    })
}

const SUMMARY_ROLL_TRANSITION_MS: u64 = 300;
const SUMMARY_ROLL_HOLD_MS: u64 = 500;
const SUMMARY_ROLL_TOTAL_MS: u64 = SUMMARY_ROLL_TRANSITION_MS + SUMMARY_ROLL_HOLD_MS;
const SUMMARY_ROLL_DRIFT_SKIP_MS: u64 = 250;
const SUMMARY_ROLL_MAX_PENDING: usize = 2;

#[derive(Clone)]
struct QueuedReasoningLine {
    line_index: usize,
    text: String,
}

pub(super) struct ReasoningSummaryRoll {
    line_index: usize,
    generation: u64,
    incoming: String,
    outgoing: Option<String>,
    pending: Vec<QueuedReasoningLine>,
    hold_until: Option<std::time::Instant>,
}

fn replace_reasoning_roll(state: &mut ReasoningSummaryRoll, line_index: usize, text: &str) {
    state.pending.clear();
    state.hold_until = None;
    state.outgoing = None;
    if state.line_index == line_index {
        state.incoming = text.to_owned();
        return;
    }
    state.incoming = text.to_owned();
    state.line_index = line_index;
    state.generation = state.generation.saturating_add(1);
}

fn enqueue_reasoning_line(pending: &mut Vec<QueuedReasoningLine>, line_index: usize, text: &str) {
    if let Some(existing) = pending
        .iter_mut()
        .find(|item| item.line_index == line_index)
    {
        existing.text = text.to_owned();
        return;
    }
    if pending.is_empty() {
        pending.push(QueuedReasoningLine {
            line_index,
            text: text.to_owned(),
        });
        return;
    }
    // The next line stays. A newer arrival only replaces the slot after it.
    pending.truncate(1);
    pending.push(QueuedReasoningLine {
        line_index,
        text: text.to_owned(),
    });
    debug_assert!(pending.len() <= SUMMARY_ROLL_MAX_PENDING);
}

fn resolve_queued_summary_playback(
    queued: &[QueuedReasoningLine],
    timer_drift_ms: u64,
) -> Vec<QueuedReasoningLine> {
    if timer_drift_ms > SUMMARY_ROLL_DRIFT_SKIP_MS && queued.len() > 1 {
        queued.last().cloned().into_iter().collect()
    } else {
        queued.to_vec()
    }
}

fn promote_reasoning_line(
    state: &mut ReasoningSummaryRoll,
    line_index: usize,
    text: &str,
    now: std::time::Instant,
) {
    state.outgoing = Some(std::mem::replace(&mut state.incoming, text.to_owned()));
    state.line_index = line_index;
    state.generation = state.generation.saturating_add(1);
    state.hold_until = Some(now + std::time::Duration::from_millis(SUMMARY_ROLL_TOTAL_MS));
}

fn settle_reasoning_roll(state: &mut ReasoningSummaryRoll, now: std::time::Instant) -> bool {
    let Some(until) = state.hold_until else {
        return false;
    };
    if now < until {
        return false;
    }
    let drift_ms =
        u64::try_from(now.saturating_duration_since(until).as_millis()).unwrap_or(u64::MAX);
    let queue = resolve_queued_summary_playback(&state.pending, drift_ms);
    state.pending.clear();
    state.hold_until = None;
    let mut queued = queue.into_iter();
    let Some(next) = queued.next() else {
        state.outgoing = None;
        return false;
    };
    state.pending.extend(queued);
    promote_reasoning_line(state, next.line_index, &next.text, now);
    true
}

fn sync_reasoning_summary_roll(
    state: &mut ReasoningSummaryRoll,
    line_index: usize,
    text: &str,
    now: std::time::Instant,
    animate: bool,
) -> Option<std::time::Instant> {
    if !animate {
        replace_reasoning_roll(state, line_index, text);
        return None;
    }
    let expired = state.hold_until.is_some_and(|until| now >= until);
    if state.line_index == line_index && !expired {
        if state.incoming != text {
            state.incoming = text.to_owned();
        }
        return state.hold_until;
    }
    if expired {
        if state.line_index != line_index {
            enqueue_reasoning_line(&mut state.pending, line_index, text);
        } else if state.incoming != text {
            state.incoming = text.to_owned();
        }
        settle_reasoning_roll(state, now);
        return state.hold_until.filter(|until| *until > now);
    }
    if state.hold_until.is_none() && !state.pending.is_empty() {
        let next = state.pending.remove(0);
        promote_reasoning_line(state, next.line_index, &next.text, now);
        enqueue_reasoning_line(&mut state.pending, line_index, text);
        return state.hold_until;
    }
    if state.hold_until.is_some() {
        enqueue_reasoning_line(&mut state.pending, line_index, text);
        return state.hold_until;
    }
    promote_reasoning_line(state, line_index, text, now);
    state.hold_until
}

fn remember_reasoning_summary(
    rolls: &std::cell::RefCell<std::collections::HashMap<String, ReasoningSummaryRoll>>,
    message_id: &str,
    line_index: usize,
    text: &str,
    now: std::time::Instant,
    animate: bool,
) -> (u64, String, Option<String>, Option<std::time::Instant>) {
    let mut rolls = rolls.borrow_mut();
    let Some(state) = rolls.get_mut(message_id) else {
        rolls.insert(
            message_id.to_owned(),
            ReasoningSummaryRoll {
                line_index,
                generation: 0,
                incoming: text.to_owned(),
                outgoing: None,
                pending: Vec::new(),
                hold_until: None,
            },
        );
        return (0, text.to_owned(), None, None);
    };
    let wake = sync_reasoning_summary_roll(state, line_index, text, now, animate);
    (
        state.generation,
        state.incoming.clone(),
        state.outgoing.clone(),
        wake,
    )
}

const REASONING_ROLL_EM: f32 = 0.8;

fn reasoning_roll_enter(progress: f32) -> (f32, f32) {
    let progress = progress.clamp(0.0, 1.0);
    (REASONING_ROLL_EM * (1.0 - progress), progress)
}

fn reasoning_roll_exit(progress: f32) -> (f32, f32) {
    let progress = progress.clamp(0.0, 1.0);
    (-REASONING_ROLL_EM * progress, 1.0 - progress)
}

fn shift_reasoning_summary(scroll: Pixels) -> Pixels {
    px(-scroll.max(px(0.)).as_f32())
}

fn rolling_reasoning_summary(
    id: SharedString,
    incoming: String,
    outgoing: Option<String>,
    incoming_scroll: Pixels,
    outgoing_scroll: Pixels,
    text_size: Pixels,
    color: Rgba,
) -> AnyElement {
    let Some(outgoing) = outgoing else {
        return summary_roll_line(incoming, px(0.), 1., color)
            .ml(shift_reasoning_summary(incoming_scroll))
            .into_any_element();
    };
    div()
        .relative()
        .overflow_hidden()
        .with_animation(
            id,
            gpui::Animation::new(std::time::Duration::from_millis(SUMMARY_ROLL_TRANSITION_MS))
                .with_easing(gpui_kit::base::animation::cubic_bezier(0.4, 0.0, 0.2, 1.0)),
            move |row, progress| {
                let (exit_em, exit_opacity) = reasoning_roll_exit(progress);
                let (enter_em, enter_opacity) = reasoning_roll_enter(progress);
                row.child(
                    summary_roll_line(outgoing.clone(), text_size * exit_em, exit_opacity, color)
                        .absolute()
                        .left(shift_reasoning_summary(outgoing_scroll)),
                )
                .child(
                    summary_roll_line(incoming.clone(), text_size * enter_em, enter_opacity, color)
                        .relative()
                        .ml(shift_reasoning_summary(incoming_scroll)),
                )
            },
        )
        .into_any_element()
}

fn summary_roll_line(text: String, offset: Pixels, opacity: f32, color: Rgba) -> Div {
    div()
        .top(offset)
        .opacity(opacity)
        .whitespace_nowrap()
        .text_color(color)
        .child(text)
}

enum ReasoningSummaryEdge {
    Leading,
    Trailing,
}

const REASONING_SCROLL_MASK_EDGE: Pixels = px(24.);
const REASONING_SCROLL_MASK_THRESHOLD: f32 = 1.;
const REASONING_BOTTOM_LOCK: f32 = 2.;

/// Top and bottom fades from `scrollMask.ts`. A body that cannot scroll has neither.
fn reasoning_scroll_mask(scroll_top: f32, max_scroll: f32) -> (bool, bool) {
    if !scroll_top.is_finite()
        || !max_scroll.is_finite()
        || max_scroll <= REASONING_SCROLL_MASK_THRESHOLD
    {
        return (false, false);
    }
    let top = scroll_top.clamp(0., max_scroll);
    (
        top > REASONING_SCROLL_MASK_THRESHOLD,
        top < max_scroll - REASONING_SCROLL_MASK_THRESHOLD,
    )
}

fn reasoning_scroll_at_bottom(scroll_top: f32, max_scroll: f32) -> bool {
    if !scroll_top.is_finite() || !max_scroll.is_finite() {
        return true;
    }
    let max_scroll = max_scroll.max(0.);
    let top = scroll_top.clamp(0., max_scroll);
    max_scroll - top <= REASONING_BOTTOM_LOCK
}

/// Positive wheel delta moves toward older text. Stay pinned only inside 2px of the end.
fn reasoning_follow_after_wheel(scroll_top: f32, max_scroll: f32, delta_y: f32) -> bool {
    let max_scroll = max_scroll.max(0.);
    let next = (scroll_top - delta_y).clamp(0., max_scroll);
    reasoning_scroll_at_bottom(next, max_scroll)
}

/// GPUI has no mask image. Paint the 24px edge over the chat surface without a hit target.
fn reasoning_scroll_fade(top: bool, surface: Rgba) -> gpui::Canvas<()> {
    let transparent = surface.opacity(0.);
    gpui::canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let (from, to) = if top {
                (surface, transparent)
            } else {
                (transparent, surface)
            };
            window.paint_quad(gpui::fill(
                bounds,
                gpui::linear_gradient(
                    180.,
                    gpui::linear_color_stop(from, 0.),
                    gpui::linear_color_stop(to, 1.),
                ),
            ));
        },
    )
    .absolute()
    .left_0()
    .right_0()
    .h(REASONING_SCROLL_MASK_EDGE)
    .when(top, |fade| fade.top(px(12.)))
    .when(!top, |fade| fade.bottom_0())
}

const REASONING_SUMMARY_EDGE: Pixels = px(16.);

/// Flat-surface stand-in for the reference mask: 16px at each end of an overflowing line.
fn reasoning_summary_edge(edge: ReasoningSummaryEdge, surface: Rgba, height: Pixels) -> Div {
    let leading = matches!(edge, ReasoningSummaryEdge::Leading);
    let transparent = surface.opacity(0.);
    let (from, to) = if leading {
        (surface, transparent)
    } else {
        (transparent, surface)
    };
    div()
        .absolute()
        .top_0()
        .h(height)
        .w(REASONING_SUMMARY_EDGE)
        .when(leading, |fade| fade.left_0())
        .when(!leading, |fade| fade.right_0())
        .bg(gpui::linear_gradient(
            90.,
            gpui::linear_color_stop(from, 0.),
            gpui::linear_color_stop(to, 1.),
        ))
}

fn shaped_text_width(
    window: &Window,
    text: &str,
    font_size: Pixels,
    weight: gpui::FontWeight,
) -> Pixels {
    if text.is_empty() {
        return px(0.);
    }
    let mut font = window.text_style().font();
    font.weight = weight;
    let text: SharedString = text.to_owned().into();
    let run = gpui::TextRun {
        len: text.len(),
        font,
        color: rgb(0).into(),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    window
        .text_system()
        .shape_line(text, font_size, &[run], None)
        .width
}

fn message_time_label(timestamp_ms: i64, chinese: bool) -> Option<String> {
    if timestamp_ms <= 0 {
        return None;
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    Some(format_message_time(
        local_civil(timestamp_ms / 1000)?,
        local_civil(now)?,
        chinese,
    ))
}

const LIGHT_EMPTY_LOGO_PATH: &str = "M398.97 0.5L147.576 319.5H1.03027L37.5996 273.081L120.167 169.603L120.171 169.598L215.342 47.5605L215.343 47.5615L252.424 0.5H398.97ZM264.544 273.271H372.527L336.082 319.498H189.886L202.642 303.307C217.584 284.34 240.398 273.271 264.544 273.271ZM209.164 0.5L202.786 8.58887C183.782 32.6885 154.782 46.752 124.091 46.752H25.9805L62.4268 0.5H209.164Z";

fn light_empty_logo_svg(color: u32) -> String {
    let channel_r = (color >> 24) & 0xff;
    let channel_g = (color >> 16) & 0xff;
    let channel_b = (color >> 8) & 0xff;
    let opacity = ((color & 0xff) as f32 / 255.) * 0.7;
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 400 320\" fill=\"none\"><defs><linearGradient id=\"fade\" x1=\"0\" y1=\"0\" x2=\"0\" y2=\"1\"><stop offset=\"0%\" stop-color=\"white\"/><stop offset=\"70%\" stop-color=\"white\" stop-opacity=\"0\"/><stop offset=\"100%\" stop-color=\"white\" stop-opacity=\"0\"/></linearGradient><mask id=\"mark\"><rect width=\"400\" height=\"320\" fill=\"url(#fade)\"/></mask></defs><path d=\"{LIGHT_EMPTY_LOGO_PATH}\" stroke=\"rgb({channel_r},{channel_g},{channel_b})\" stroke-opacity=\"{opacity:.4}\" mask=\"url(#mark)\"/></svg>"
    )
}

fn empty_logo_image(dark: bool, subtlest: u32) -> Arc<gpui::Image> {
    if dark {
        static DARK: std::sync::OnceLock<Arc<gpui::Image>> = std::sync::OnceLock::new();
        return DARK
            .get_or_init(|| {
                Arc::new(gpui::Image::from_bytes(
                    gpui::ImageFormat::Svg,
                    include_bytes!("../assets/empty-state-dark.svg").to_vec(),
                ))
            })
            .clone();
    }
    static LIGHT: std::sync::OnceLock<Arc<gpui::Image>> = std::sync::OnceLock::new();
    LIGHT
        .get_or_init(|| {
            Arc::new(gpui::Image::from_bytes(
                gpui::ImageFormat::Svg,
                light_empty_logo_svg(subtlest).into_bytes(),
            ))
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::{
        assistant_heading_size, assistant_heading_weight, assistant_message_is_streaming,
        assistant_stream_motion, assistant_table_header_weight, automation_grid_columns,
        automation_schedule_label, chat_content_width, civil_day_number,
        compact_header_actions_width, compact_header_controls_fit, compact_token_count,
        conversation_composer_width, empty_greeting, error_banner_tone, execute_command_key,
        execute_command_limit, execute_command_overflows, execute_command_text_width,
        format_message_time, gradient_glyph_alphas, gradient_text_alpha, greeting_slot,
        header_uses_compact_navigation, message_action_fade_ms, message_actions_are_revealed,
        next_greeting_delay_ms, parse_slash_command, raw_tool_output_font_size,
        raw_tool_output_presentation, reasoning_follow_after_wheel, reasoning_roll_enter,
        reasoning_roll_exit, reasoning_scroll_at_bottom, reasoning_scroll_mask,
        reasoning_streaming_summary_is_shown, reasoning_streaming_summary_line,
        reasoning_summary_is_overflowing, reasoning_summary_is_streaming,
        reasoning_summary_row_width, reasoning_summary_scroll_left, reasoning_summary_text_budget,
        replace_reasoning_roll, resolve_greeting_font_size, resolve_queued_summary_playback,
        settle_reasoning_roll, sidebar_is_visible, split_user_input_epilogue,
        summary_disclosure_is_visible, sync_reasoning_summary_roll, terminal_summary_hides_detail,
        tool_output_chrome, tool_summary_copy, transcript_content_width,
        transcript_message_is_work, transcript_row_top, transcript_turn_padding,
        trim_epilogue_lead, user_message_bubble_width, user_message_content_is_expandable,
        visible_reasoning_summary, CivilTime, ErrorBannerTone, Glyph, GreetingSlot,
        ReasoningSummaryRoll, SlashCommand, ThemeColors, ToolKindLabel, ToolSummaryPhase,
        ASSISTANT_BLOCK_GAP_REM, ASSISTANT_CODE_FONT, ASSISTANT_CODE_RADIUS,
        ASSISTANT_HEADING_PADDING_BOTTOM, ASSISTANT_HEADING_PADDING_TOP,
        ASSISTANT_TABLE_CELL_PADDING_Y, ASSISTANT_TABLE_RADIUS, DRAFT_SUGGESTED_PROMPTS,
        SIDEBAR_HIDE_BREAKPOINT, SIDEBAR_HISTORY_ROW_HEIGHT, SIDEBAR_NAV_RADIUS,
        SIDEBAR_NAV_ROW_HEIGHT, SIDEBAR_NAV_TO_TASKS_GAP, SIDEBAR_WIDTH,
        SUMMARY_ROLL_DRIFT_SKIP_MS, SUMMARY_ROLL_TOTAL_MS,
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
    fn tool_summaries_follow_the_reference_row() {
        let command = serde_json::json!({"command": "ls\n-la", "timeout_secs": 60});
        let running = tool_summary_copy("bash", Some(&command), ToolSummaryPhase::Running);
        assert_eq!(running.icon, Glyph::SquareTerminal);
        assert_eq!(
            running.kind,
            ToolKindLabel::Localized("正在执行", "Running")
        );
        assert_eq!(running.detail, "ls -la");
        assert!(running.status.is_none());

        let done = tool_summary_copy("bash", Some(&command), ToolSummaryPhase::Done);
        assert_eq!(done.kind, ToolKindLabel::Localized("终端", "Terminal"));
        assert!(done.status.is_none());

        let failed = tool_summary_copy(
            "read_file",
            Some(&serde_json::json!({"path": "src/main.rs"})),
            ToolSummaryPhase::Failed,
        );
        assert_eq!(failed.icon, Glyph::Search);
        assert_eq!(failed.kind, ToolKindLabel::Localized("读取", "Read"));
        assert_eq!(failed.detail, "src/main.rs");
        assert_eq!(failed.status, Some(("执行失败", "Failed")));

        let editing = tool_summary_copy(
            "edit_file",
            Some(&serde_json::json!({"path": "a.rs"})),
            ToolSummaryPhase::Running,
        );
        assert_eq!(editing.icon, Glyph::FilePen);
        assert_eq!(
            editing.kind,
            ToolKindLabel::Localized("正在编辑", "Editing")
        );

        let search = tool_summary_copy(
            "grep",
            Some(&serde_json::json!({"pattern": "foo"})),
            ToolSummaryPhase::Done,
        );
        assert_eq!(search.kind, ToolKindLabel::Localized("搜索", "Search"));
        assert_eq!(search.detail, "foo");
        assert!(search.status.is_none());

        let generic = tool_summary_copy(
            "calc",
            Some(&serde_json::json!({"action": "sum"})),
            ToolSummaryPhase::Done,
        );
        assert_eq!(generic.icon, Glyph::Calculator);
        assert_eq!(generic.kind, ToolKindLabel::Raw("calc".into()));
        assert_eq!(generic.detail, "sum");
        assert_eq!(generic.status, Some(("已执行", "Completed")));

        let stopped = tool_summary_copy("office", None, ToolSummaryPhase::Stopped);
        assert_eq!(stopped.icon, Glyph::File);
        assert_eq!(stopped.status, Some(("已停止", "Stopped")));

        let unknown = tool_summary_copy("custom", None, ToolSummaryPhase::Running);
        assert_eq!(unknown.icon, Glyph::Wrench);
        assert_eq!(unknown.status, Some(("执行中", "Running")));

        assert!(reasoning_summary_is_streaming(true, true, true, false));
        assert!(!reasoning_summary_is_streaming(true, true, false, false));
        assert!(!reasoning_summary_is_streaming(true, true, true, true));
        assert!(!reasoning_summary_is_streaming(false, true, true, false));
    }

    #[test]
    fn message_times_follow_the_locale_calendar() {
        let now = CivilTime {
            year: 2026,
            month: 3,
            day: 1,
            hour: 15,
            minute: 0,
            second: 0,
        };
        let today = CivilTime {
            hour: 14,
            minute: 5,
            ..now
        };
        assert_eq!(format_message_time(today, now, true), "14:05");
        assert_eq!(format_message_time(today, now, false), "02:05 PM");
        assert_eq!(
            format_message_time(
                CivilTime {
                    hour: 0,
                    minute: 0,
                    ..today
                },
                now,
                false
            ),
            "12:00 AM"
        );
        assert_eq!(
            format_message_time(
                CivilTime {
                    hour: 12,
                    minute: 0,
                    ..today
                },
                now,
                false
            ),
            "12:00 PM"
        );
        let yesterday = CivilTime {
            year: 2026,
            month: 2,
            day: 28,
            hour: 9,
            minute: 7,
            second: 0,
        };
        assert_eq!(format_message_time(yesterday, now, true), "昨天 09:07");
        assert_eq!(
            format_message_time(yesterday, now, false),
            "Yesterday 09:07 AM"
        );
        let earlier = CivilTime {
            day: 2,
            ..yesterday
        };
        assert_eq!(format_message_time(earlier, now, true), "2/2 09:07");
        assert_eq!(format_message_time(earlier, now, false), "2/2, 09:07 AM");
        let last_year = CivilTime {
            year: 2025,
            month: 1,
            day: 3,
            hour: 9,
            minute: 7,
            second: 0,
        };
        assert_eq!(format_message_time(last_year, now, true), "2025/1/3 09:07");
        assert_eq!(
            format_message_time(last_year, now, false),
            "1/3/2025, 09:07 AM"
        );
        assert_eq!(
            civil_day_number(CivilTime {
                year: 1970,
                month: 1,
                day: 1,
                hour: 0,
                minute: 0,
                second: 0,
            }),
            0
        );
    }

    #[test]
    fn reasoning_chevron_appears_on_hover_or_when_open() {
        assert!(!summary_disclosure_is_visible(false, false));
        assert!(summary_disclosure_is_visible(false, true));
        assert!(summary_disclosure_is_visible(true, false));
        assert!(summary_disclosure_is_visible(true, true));
    }

    #[test]
    fn collapsed_reasoning_summary_scrolls_the_full_line_to_its_end() {
        assert!(!reasoning_streaming_summary_is_shown(false, false));
        assert!(!reasoning_streaming_summary_is_shown(true, true));
        assert!(reasoning_streaming_summary_is_shown(true, false));

        let alpha = reasoning_streaming_summary_line("alpha\n \n\t").unwrap();
        assert_eq!(alpha.text, "alpha");
        assert_eq!(alpha.index, 0);
        let second = reasoning_streaming_summary_line("a\r\nb\r").unwrap();
        assert_eq!(second.text, "b");
        assert_eq!(second.index, 1);
        assert_eq!(reasoning_streaming_summary_line("a\rb").unwrap().text, "b");
        assert_eq!(
            reasoning_streaming_summary_line("  padded  ").unwrap().text,
            "padded"
        );
        assert!(reasoning_streaming_summary_line("\r\n").is_none());

        assert_eq!(reasoning_summary_row_width(px(912.)), px(768.));
        assert_eq!(reasoning_summary_row_width(px(480.)), px(432.));
        assert_eq!(reasoning_summary_row_width(px(1180.)), px(848.));
        assert_eq!(reasoning_summary_row_width(px(447.)), px(415.));
        assert_eq!(reasoning_summary_row_width(px(448.)), px(400.));
        assert_eq!(reasoning_summary_row_width(px(767.)), px(719.));
        assert_eq!(reasoning_summary_row_width(px(768.)), px(720.));
        assert_eq!(reasoning_summary_row_width(px(0.)), px(0.));
        assert_eq!(
            reasoning_summary_text_budget(px(200.), px(40.), px(8.)),
            px(88.)
        );
        assert_eq!(
            reasoning_summary_text_budget(px(70.), px(20.), px(10.)),
            px(0.)
        );

        let per_char = |text: &str| px(text.chars().count() as f32);
        let fitted = visible_reasoning_summary("newest tail", None, px(11.), per_char).unwrap();
        assert_eq!(fitted.text, "newest tail");
        assert_eq!(fitted.client_width, px(11.));
        assert_eq!(fitted.scroll_left, px(0.));
        assert!(!fitted.masked);
        let scrolled = visible_reasoning_summary("newest tail", None, px(4.), per_char).unwrap();
        assert_eq!(scrolled.text, "newest tail");
        assert_eq!(scrolled.client_width, px(4.));
        assert_eq!(scrolled.scroll_left, px(7.));
        assert!(scrolled.masked);
        let partial = visible_reasoning_summary("ééé", None, px(1.), per_char).unwrap();
        assert_eq!(partial.text, "ééé");
        assert_eq!(partial.client_width, px(1.));
        assert_eq!(partial.scroll_left, px(2.));
        assert!(partial.masked);
        let wider = visible_reasoning_summary("ab", None, px(1.), |text| {
            px(text.chars().count() as f32 * 2.)
        })
        .unwrap();
        assert_eq!(wider.text, "ab");
        assert_eq!(wider.client_width, px(1.));
        assert_eq!(wider.scroll_left, px(3.));
        assert!(wider.masked);
        let squeezed = visible_reasoning_summary("keep", None, px(0.), per_char).unwrap();
        assert_eq!(squeezed.text, "keep");
        assert_eq!(squeezed.client_width, px(0.));
        assert_eq!(squeezed.scroll_left, px(4.));
        assert!(squeezed.masked);
        assert!(visible_reasoning_summary("", None, px(8.), per_char).is_none());
        let within_slop = visible_reasoning_summary("abcd", None, px(3.5), per_char).unwrap();
        assert_eq!(within_slop.client_width, px(3.5));
        assert_eq!(within_slop.scroll_left, px(0.5));
        assert!(!within_slop.masked);
        assert!(!reasoning_summary_is_overflowing(px(11.), px(10.)));
        assert!(reasoning_summary_is_overflowing(px(11.01), px(10.)));
        assert_eq!(reasoning_summary_scroll_left(px(40.), px(10.)), px(30.));
        assert_eq!(reasoning_summary_scroll_left(px(4.), px(10.)), px(0.));
    }

    #[test]
    fn reasoning_scroll_mask_follows_the_one_pixel_edges() {
        assert_eq!(reasoning_scroll_mask(0., 0.), (false, false));
        assert_eq!(reasoning_scroll_mask(0., 1.), (false, false));
        assert_eq!(reasoning_scroll_mask(0., 100.), (false, true));
        assert_eq!(reasoning_scroll_mask(1., 100.), (false, true));
        assert_eq!(reasoning_scroll_mask(1.1, 100.), (true, true));
        assert_eq!(reasoning_scroll_mask(99., 100.), (true, false));
        assert_eq!(reasoning_scroll_mask(98.9, 100.), (true, true));
    }

    #[test]
    fn reasoning_follow_releases_beyond_two_pixels_and_relocks_at_the_end() {
        assert!(reasoning_scroll_at_bottom(100., 100.));
        assert!(reasoning_scroll_at_bottom(98., 100.));
        assert!(!reasoning_scroll_at_bottom(97.9, 100.));
        assert!(!reasoning_follow_after_wheel(100., 100., 3.));
        assert!(reasoning_follow_after_wheel(100., 100., 2.));
        assert!(reasoning_follow_after_wheel(90., 100., -12.));
        assert!(!reasoning_follow_after_wheel(90., 100., -7.));
    }

    #[test]
    fn reasoning_summary_holds_a_line_before_playing_the_next() {
        let start = std::time::Instant::now();
        let at = |millis: u64| start + std::time::Duration::from_millis(millis);
        let mut state = ReasoningSummaryRoll {
            line_index: 0,
            generation: 0,
            incoming: "first".into(),
            outgoing: None,
            pending: Vec::new(),
            hold_until: None,
        };
        assert!(sync_reasoning_summary_roll(&mut state, 0, "first more", at(0), true).is_none());
        assert_eq!(state.incoming, "first more");
        assert!(state.outgoing.is_none());
        assert_eq!(state.generation, 0);

        let hold = sync_reasoning_summary_roll(&mut state, 1, "second", at(10), true).unwrap();
        assert_eq!(hold, at(10 + SUMMARY_ROLL_TOTAL_MS));
        assert_eq!(state.incoming, "second");
        assert_eq!(state.outgoing.as_deref(), Some("first more"));
        assert_eq!(state.generation, 1);

        assert_eq!(
            sync_reasoning_summary_roll(&mut state, 1, "second more", at(20), true),
            Some(hold)
        );
        assert_eq!(state.incoming, "second more");
        assert!(state.pending.is_empty());

        sync_reasoning_summary_roll(&mut state, 2, "third", at(30), true);
        sync_reasoning_summary_roll(&mut state, 3, "fourth", at(40), true);
        sync_reasoning_summary_roll(&mut state, 4, "fifth", at(50), true);
        sync_reasoning_summary_roll(&mut state, 4, "fifth more", at(60), true);
        assert_eq!(
            state
                .pending
                .iter()
                .map(|item| (item.line_index, item.text.as_str()))
                .collect::<Vec<_>>(),
            vec![(2, "third"), (4, "fifth more")]
        );
        assert_eq!(state.incoming, "second more");

        assert!(settle_reasoning_roll(
            &mut state,
            at(10 + SUMMARY_ROLL_TOTAL_MS)
        ));
        assert_eq!(state.incoming, "third");
        assert_eq!(state.outgoing.as_deref(), Some("second more"));
        assert_eq!(state.pending.len(), 1);

        let mut late = ReasoningSummaryRoll {
            line_index: 0,
            generation: 0,
            incoming: "a".into(),
            outgoing: None,
            pending: Vec::new(),
            hold_until: None,
        };
        sync_reasoning_summary_roll(&mut late, 1, "b", at(0), true);
        sync_reasoning_summary_roll(&mut late, 2, "c", at(10), true);
        sync_reasoning_summary_roll(&mut late, 3, "d", at(20), true);
        sync_reasoning_summary_roll(
            &mut late,
            4,
            "e",
            at(SUMMARY_ROLL_TOTAL_MS + SUMMARY_ROLL_DRIFT_SKIP_MS + 1),
            true,
        );
        assert_eq!(late.incoming, "e");
        assert!(late.pending.is_empty());
        assert_eq!(late.outgoing.as_deref(), Some("b"));

        let queued = state.pending.clone();
        let pair = [queued[0].clone(), queued[0].clone()];
        assert_eq!(
            resolve_queued_summary_playback(&pair, SUMMARY_ROLL_DRIFT_SKIP_MS).len(),
            2
        );
        assert_eq!(
            resolve_queued_summary_playback(&pair, SUMMARY_ROLL_DRIFT_SKIP_MS + 1).len(),
            1
        );

        replace_reasoning_roll(&mut state, 9, "snapped");
        assert_eq!(state.incoming, "snapped");
        assert!(state.outgoing.is_none());
        assert!(state.pending.is_empty());
        assert!(state.hold_until.is_none());

        let per_char = |text: &str| px(text.chars().count() as f32);
        let visible =
            visible_reasoning_summary("held line", Some("older"), px(4.), per_char).unwrap();
        assert_eq!(visible.text, "held line");
        assert_eq!(visible.client_width, px(4.));
        assert_eq!(visible.scroll_left, px(5.));
        assert_eq!(visible.outgoing.as_deref(), Some("older"));
        assert_eq!(visible.outgoing_scroll_left, px(1.));
        assert!(visible.masked);
        let short_incoming =
            visible_reasoning_summary("ab", Some("older line"), px(10.), per_char).unwrap();
        assert_eq!(short_incoming.text, "ab");
        assert_eq!(short_incoming.client_width, px(2.));
        assert_eq!(short_incoming.scroll_left, px(0.));
        assert_eq!(short_incoming.outgoing.as_deref(), Some("older line"));
        assert_eq!(short_incoming.outgoing_scroll_left, px(8.));
        assert!(short_incoming.masked);

        assert_eq!(reasoning_roll_enter(0.0), (0.8, 0.0));
        assert_eq!(reasoning_roll_enter(1.0), (0.0, 1.0));
        assert_eq!(reasoning_roll_exit(0.0), (0.0, 1.0));
        assert_eq!(reasoning_roll_exit(1.0), (-0.8, 0.0));
    }

    #[test]
    fn running_labels_follow_the_gradient_sweep() {
        assert_eq!(gradient_text_alpha(0.0, 0.5), 1.0);
        assert_eq!(gradient_text_alpha(0.5, 0.5), 1.0);
        assert!((gradient_text_alpha(0.25, 0.5) - 0.22).abs() < 0.001);
        assert!((gradient_text_alpha(0.125, 0.0) - 0.22).abs() < 0.001);
        assert!((gradient_text_alpha(0.375, 1.0) - 0.22).abs() < 0.001);

        let sweeping = gradient_glyph_alphas("正在思考", 0.25);
        assert!(sweeping[1] < sweeping[0]);
        assert!(sweeping[2] < sweeping[3]);
        assert!(sweeping.iter().all(|alpha| (0.22..=1.0).contains(alpha)));
        assert!(gradient_glyph_alphas("正在思考", 0.0)
            .iter()
            .all(|alpha| *alpha == 1.0));
        assert!(gradient_glyph_alphas("Thinking", 0.5)
            .iter()
            .all(|alpha| *alpha == 1.0));
    }

    #[test]
    fn message_actions_reveal_on_hover_or_focus_within() {
        assert!(!message_actions_are_revealed(false, false));
        assert!(message_actions_are_revealed(true, false));
        assert!(message_actions_are_revealed(false, true));
        assert!(message_actions_are_revealed(true, true));
        assert_eq!(message_action_fade_ms(false), 150);
        assert_eq!(message_action_fade_ms(true), 0);
    }

    #[test]
    fn message_feedback_toggles_correctly() {
        use crate::MessageFeedback;
        let mut feedback = std::collections::HashMap::new();
        let id = "msg-1".to_string();
        assert_eq!(feedback.get(&id), None);
        feedback.insert(id.clone(), MessageFeedback::Like);
        assert_eq!(feedback.get(&id), Some(&MessageFeedback::Like));
        feedback.remove(&id);
        assert_eq!(feedback.get(&id), None);
        feedback.insert(id.clone(), MessageFeedback::Dislike);
        assert_eq!(feedback.get(&id), Some(&MessageFeedback::Dislike));
    }

    #[test]
    fn transcript_rows_follow_the_turn_spacing() {
        assert_eq!(transcript_row_top(0, true, false), px(56.));
        assert_eq!(transcript_row_top(1, true, false), px(76.));
        assert_eq!(transcript_row_top(0, false, false), px(16.));
        assert_eq!(transcript_row_top(2, false, false), px(20.));
        assert_eq!(transcript_row_top(2, false, true), px(16.));
        assert_eq!(transcript_row_top(1, true, true), px(76.));
        assert_eq!(transcript_turn_padding(px(447.)), px(16.));
        assert_eq!(transcript_turn_padding(px(448.)), px(24.));
        assert_eq!(transcript_content_width(px(447.)), px(415.));
        assert_eq!(transcript_content_width(px(448.)), px(400.));

        let prose = pi_agent_core::Message::assistant("done");
        assert!(!transcript_message_is_work(&prose));
        let mut reasoning = pi_agent_core::Message::assistant("");
        reasoning.reasoning = Some("thinking".into());
        assert!(transcript_message_is_work(&reasoning));
        let mut blank = pi_agent_core::Message::assistant("  ");
        blank.reasoning = Some("  ".into());
        assert!(!transcript_message_is_work(&blank));
        assert!(transcript_message_is_work(
            &pi_agent_core::Message::tool_result("1", "bash", "ok", false)
        ));
        assert!(!transcript_message_is_work(&pi_agent_core::Message::user(
            "hi"
        )));
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
        assert_eq!(user_message_bubble_width(px(0.)), px(0.));
        assert_eq!(user_message_bubble_width(px(447.)), px(415.));
        assert_eq!(user_message_bubble_width(px(448.)), px(400.));
        assert_eq!(user_message_bubble_width(px(480.)), px(432.));
        assert_eq!(user_message_bubble_width(px(623.)), px(575.));
        assert_eq!(user_message_bubble_width(px(624.)), px(576.));
        assert_eq!(user_message_bubble_width(px(625.)), px(576.));
        assert_eq!(user_message_bubble_width(px(647.)), px(576.));
        assert_eq!(user_message_bubble_width(px(671.)), px(576.));
        assert_eq!(user_message_bubble_width(px(672.)), px(576.));
        assert_eq!(user_message_bubble_width(px(1180.)), px(576.));
        for width in 0..1600 {
            assert!(
                user_message_bubble_width(px(width as f32))
                    <= transcript_content_width(px(width as f32))
            );
        }
        assert!(!user_message_content_is_expandable(px(121.)));
        assert!(user_message_content_is_expandable(px(121.01)));
    }

    #[test]
    fn expanded_tool_output_uses_the_execute_panel_or_fallback_surface() {
        let terminal = tool_output_chrome("execute");
        assert_eq!(terminal.pad_x, px(16.));
        assert_eq!(terminal.pad_y, px(12.));
        assert_eq!(terminal.radius, px(12.));
        assert_eq!(terminal.before, px(8.));
        assert_eq!(terminal.after, px(8.));
        assert!(terminal.bordered);
        assert!(tool_output_chrome("bash").bordered);
        assert!(tool_output_chrome("shell").bordered);

        let fallback = tool_output_chrome("read_file");
        assert_eq!(fallback.pad_x, px(16.));
        assert_eq!(fallback.pad_y, px(12.));
        assert_eq!(fallback.before, px(12.));
        assert_eq!(fallback.after, px(0.));
        assert!(!fallback.bordered);
    }

    #[test]
    fn raw_tool_output_uses_terminal_or_fallback_metrics() {
        assert_eq!(raw_tool_output_presentation("bash"), (px(14.), px(100.)));
        assert_eq!(raw_tool_output_presentation("execute"), (px(14.), px(100.)));
        assert_eq!(raw_tool_output_presentation("shell"), (px(14.), px(100.)));
        assert_eq!(
            raw_tool_output_presentation("read_file"),
            (px(10.), px(200.))
        );
        assert_eq!(raw_tool_output_font_size("bash", 16.), px(16.));
        assert_eq!(raw_tool_output_font_size("read_file", 14.), px(10.));
        assert_eq!(raw_tool_output_font_size("read_file", 12.), px(8.));
    }

    #[test]
    fn execute_command_preview_keeps_three_lines_and_opens_to_ten() {
        assert_eq!(execute_command_limit(false), px(60.));
        assert_eq!(execute_command_limit(true), px(200.));
        assert!(!execute_command_overflows(px(60.)));
        assert!(!execute_command_overflows(px(61.)));
        assert!(execute_command_overflows(px(61.01)));
        assert_eq!(execute_command_text_width(px(448.), px(8.)), px(398.));
        assert_eq!(execute_command_text_width(px(768.), px(12.)), px(714.));
        assert_eq!(execute_command_text_width(px(30.), px(8.)), px(1.));
    }

    #[test]
    fn execute_command_disclosures_do_not_leak_between_conversations() {
        assert_ne!(
            execute_command_key("chat-a", "call-1"),
            execute_command_key("chat-b", "call-1")
        );
        assert_ne!(
            execute_command_key("a:b", "c"),
            execute_command_key("a", "b:c")
        );
        assert_eq!(
            execute_command_key("chat-a", "call-1"),
            execute_command_key("chat-a", "call-1")
        );
    }

    #[test]
    fn open_terminal_moves_existing_command_out_of_the_summary() {
        assert!(terminal_summary_hides_detail("bash", true, true));
        assert!(terminal_summary_hides_detail("execute", true, true));
        assert!(terminal_summary_hides_detail("shell", true, true));
        assert!(!terminal_summary_hides_detail("bash", false, true));
        assert!(!terminal_summary_hides_detail("read", true, true));
        assert!(!terminal_summary_hides_detail("bash", true, false));
    }

    #[test]
    fn assistant_streaming_uses_the_reference_fade() {
        let motion = assistant_stream_motion();
        assert_eq!(motion.stream_fade(), std::time::Duration::from_millis(900));
        assert_eq!(motion.stream_fade_stagger(), std::time::Duration::ZERO);
    }

    #[test]
    fn assistant_headings_follow_the_interface_font() {
        assert_eq!(assistant_heading_size(1, 14.), 18.);
        assert_eq!(assistant_heading_size(2, 14.), 16.);
        assert_eq!(assistant_heading_size(3, 14.), 14.);
        assert_eq!(assistant_heading_size(6, 20.), 20.);
        assert_eq!(assistant_heading_size(1, 12.), 16.);
        assert_eq!(assistant_heading_weight(1), gpui_kit::FontWeight::SEMIBOLD);
        assert_eq!(assistant_heading_weight(4), gpui_kit::FontWeight::SEMIBOLD);
        assert_eq!(assistant_heading_weight(5), gpui_kit::FontWeight::MEDIUM);
        assert_eq!(assistant_heading_weight(6), gpui_kit::FontWeight::NORMAL);
    }

    #[test]
    fn empty_greeting_follows_local_hour_bands() {
        assert_eq!(greeting_slot(4), GreetingSlot::LateNight);
        assert_eq!(greeting_slot(5), GreetingSlot::MorningEarly);
        assert_eq!(greeting_slot(8), GreetingSlot::MorningEarly);
        assert_eq!(greeting_slot(9), GreetingSlot::Morning);
        assert_eq!(greeting_slot(12), GreetingSlot::Noon);
        assert_eq!(greeting_slot(14), GreetingSlot::Afternoon);
        assert_eq!(greeting_slot(18), GreetingSlot::Evening);
        assert_eq!(greeting_slot(23), GreetingSlot::LateNight);
        assert_eq!(
            empty_greeting(5),
            ("早上好呀，新的一天开始啦", "Morning, ready when you are")
        );
        assert_eq!(empty_greeting(12).0, "中午好呀，要不要先休息一下");
        assert_eq!(next_greeting_delay_ms(4, 59, 59, 0), 1_000);
        assert_eq!(next_greeting_delay_ms(5, 0, 0, 0), 4 * 3_600_000);
        assert_eq!(next_greeting_delay_ms(23, 0, 0, 0), 6 * 3_600_000);
        assert_eq!(resolve_greeting_font_size(406., 360.), 30.);
        assert_eq!(resolve_greeting_font_size(359., 360.), 29.);
        assert_eq!(resolve_greeting_font_size(300., 360.), 25.);
        assert_eq!(resolve_greeting_font_size(120., 360.), 20.);
        assert_eq!(resolve_greeting_font_size(0., 360.), 30.);
        assert_eq!(resolve_greeting_font_size(406., f32::NAN), 30.);
    }

    #[test]
    fn assistant_tables_and_links_follow_the_message_contract() {
        assert_eq!(ASSISTANT_TABLE_RADIUS, px(12.));
        assert_eq!(ASSISTANT_TABLE_CELL_PADDING_Y, px(8.));
        assert_eq!(
            assistant_table_header_weight(),
            gpui_kit::FontWeight::NORMAL
        );
        assert_eq!(ThemeColors::light().link, 0x0066dd);
        assert_eq!(ThemeColors::dark().link, 0x80beff);
        assert_eq!(ASSISTANT_CODE_RADIUS, px(12.));
        assert_eq!(ASSISTANT_CODE_FONT, px(14.));
        assert_eq!(ASSISTANT_BLOCK_GAP_REM, 1.);
        assert_eq!(ASSISTANT_HEADING_PADDING_TOP, 8.);
        assert_eq!(ASSISTANT_HEADING_PADDING_BOTTOM, 16.);
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

    #[test]
    fn draft_suggested_prompts_match_reference_contract() {
        assert_eq!(DRAFT_SUGGESTED_PROMPTS.len(), 4);
        assert_eq!(DRAFT_SUGGESTED_PROMPTS[0].id, "weekly-report");
        assert_eq!(DRAFT_SUGGESTED_PROMPTS[0].label_zh, "周报总结");
        assert_eq!(DRAFT_SUGGESTED_PROMPTS[0].label_en, "Weekly report");
        assert_eq!(DRAFT_SUGGESTED_PROMPTS[1].id, "bug-fix");
        assert_eq!(DRAFT_SUGGESTED_PROMPTS[1].label_zh, "报错修复");
        assert_eq!(DRAFT_SUGGESTED_PROMPTS[2].id, "make-ppt");
        assert_eq!(DRAFT_SUGGESTED_PROMPTS[3].id, "idle-tasks");
        for prompt in DRAFT_SUGGESTED_PROMPTS {
            assert!(!prompt.prompt_zh.is_empty());
            assert!(!prompt.prompt_en.is_empty());
        }
    }

    #[test]
    fn error_banner_distinguishes_stopped_and_failure() {
        assert_eq!(error_banner_tone(true), ErrorBannerTone::Stopped);
        assert_eq!(error_banner_tone(false), ErrorBannerTone::Failure);
    }

    #[test]
    fn user_input_epilogue_split_and_trim_match_reference_contract() {
        let ask = "Summarize the repo";
        let epilogue = "\n\n---\nStandard for this result:\n- Every finding cites what you read.";
        let full = format!("{ask}{epilogue}");

        let (body, found_epilogue) = split_user_input_epilogue(&full, Some(ask.len()));
        assert_eq!(body, ask);
        assert_eq!(found_epilogue, Some(epilogue));
        let trimmed = trim_epilogue_lead(found_epilogue.unwrap());
        assert_eq!(
            trimmed,
            "Standard for this result:\n- Every finding cites what you read."
        );
        assert!(!trimmed.starts_with("---"));

        let nudge = "You ended your turn without submitting a result.";
        let (nudge_body, nudge_epilogue) = split_user_input_epilogue(nudge, Some(0));
        assert!(nudge_body.is_empty());
        assert_eq!(nudge_epilogue, Some(nudge));
        assert_eq!(trim_epilogue_lead(nudge_epilogue.unwrap()), nudge);

        let (out_body, out_epilogue) = split_user_input_epilogue(&full, Some(full.len() + 10));
        assert_eq!(out_body, full.as_str());
        assert!(out_epilogue.is_none());

        let plain = "Plain user question";
        let (plain_body, plain_epilogue) = split_user_input_epilogue(plain, None);
        assert_eq!(plain_body, plain);
        assert!(plain_epilogue.is_none());

        let (auto_body, auto_epilogue) = split_user_input_epilogue(&full, None);
        assert_eq!(auto_body, ask);
        assert_eq!(auto_epilogue, Some(epilogue));
    }

    #[test]
    fn slash_command_parsing_matches_reference_contract() {
        assert_eq!(parse_slash_command("/compact"), Some(SlashCommand::Compact));
        assert_eq!(
            parse_slash_command("/compress"),
            Some(SlashCommand::Compact)
        );
        assert_eq!(parse_slash_command("/goal"), Some(SlashCommand::EmptyGoal));
        assert_eq!(parse_slash_command("/goal "), Some(SlashCommand::EmptyGoal));
        assert_eq!(
            parse_slash_command("/goal resume"),
            Some(SlashCommand::ResumeGoal)
        );
        assert_eq!(
            parse_slash_command("/goal clear"),
            Some(SlashCommand::ClearGoal)
        );
        assert_eq!(
            parse_slash_command("/goal Run overnight tests"),
            Some(SlashCommand::SendGoal {
                objective: "Run overnight tests".into(),
            })
        );
        assert_eq!(
            parse_slash_command("/goal replace Implement login"),
            Some(SlashCommand::SendGoal {
                objective: "Implement login".into(),
            })
        );
        assert_eq!(
            parse_slash_command("/target Deploy app"),
            Some(SlashCommand::SendGoal {
                objective: "Deploy app".into(),
            })
        );
        assert_eq!(parse_slash_command("regular user prompt"), None);
    }

    #[test]
    fn sidebar_primary_navigation_order_and_metrics_match_contract() {
        assert_eq!(SIDEBAR_NAV_ROW_HEIGHT, px(32.));
        assert_eq!(SIDEBAR_HISTORY_ROW_HEIGHT, px(32.));
        assert_eq!(SIDEBAR_NAV_RADIUS, px(8.));
        assert_eq!(SIDEBAR_WIDTH, px(268.));
        assert_eq!(SIDEBAR_HIDE_BREAKPOINT, px(820.));
        assert_eq!(SIDEBAR_NAV_TO_TASKS_GAP, px(21.));
    }
}
