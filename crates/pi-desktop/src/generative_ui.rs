use crate::theme::ThemeColors;
use gpui_kit::base::{Disableable, StyledExt};
use gpui_kit::component::button::{Button, ButtonCustomVariant, ButtonVariants};
use gpui_kit::gpui::*;
use gpui_kit::prelude::FluentBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UiNode {
    Card {
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        subtitle: Option<String>,
        #[serde(default)]
        badge: Option<BadgeNode>,
        child: Box<UiNode>,
    },
    Container {
        #[serde(default)]
        direction: LayoutDirection,
        #[serde(default)]
        gap: Option<f32>,
        #[serde(default)]
        align: Option<LayoutAlign>,
        children: Vec<UiNode>,
    },
    Text {
        content: String,
        #[serde(default)]
        variant: TextVariant,
    },
    Badge(BadgeNode),
    Button {
        id: String,
        label: String,
        #[serde(default)]
        variant: ButtonVariant,
        #[serde(default)]
        disabled: bool,
    },
    KeyValueList {
        items: Vec<KeyValueItem>,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Progress {
        value: f32,
        #[serde(default = "default_max")]
        max: f32,
        #[serde(default)]
        label: Option<String>,
        #[serde(default)]
        tone: BadgeTone,
    },
    Divider,
}

fn default_max() -> f32 {
    100.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LayoutDirection {
    #[default]
    Column,
    Row,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LayoutAlign {
    #[default]
    Start,
    Center,
    End,
    Between,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TextVariant {
    #[default]
    Body,
    Title,
    Heading,
    Subtle,
    Code,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BadgeNode {
    pub label: String,
    #[serde(default)]
    pub tone: BadgeTone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BadgeTone {
    #[default]
    Neutral,
    Success,
    Warning,
    Danger,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Outline,
    Ghost,
    Danger,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyValueItem {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub badge: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiAction {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContentSegment {
    Markdown(String),
    GenerativeUi(UiNode),
}

/// Parses message text containing optional ```json-ui, ```ui, ```generative-ui,
/// or ```adaptive-card fenced blocks into a sequence of Markdown text and native UI nodes.
pub fn parse_message_segments(text: &str) -> Vec<ContentSegment> {
    let mut segments = Vec::new();
    let mut current_offset = 0;
    let markers = [
        "```json-ui",
        "```ui",
        "```generative-ui",
        "```adaptive-card",
    ];

    while current_offset < text.len() {
        let remaining = &text[current_offset..];
        let mut earliest_match: Option<(usize, &str)> = None;

        for marker in &markers {
            // Case-insensitive search for marker
            if let Some(pos) = remaining.to_lowercase().find(marker) {
                match earliest_match {
                    None => earliest_match = Some((pos, marker)),
                    Some((earliest_pos, _)) if pos < earliest_pos => {
                        earliest_match = Some((pos, marker));
                    }
                    _ => {}
                }
            }
        }

        if let Some((start_rel, marker)) = earliest_match {
            let block_start = current_offset + start_rel;
            let content_start = block_start + marker.len();

            // Find closing fence ```
            if let Some(end_rel) = text[content_start..].find("```") {
                let content_end = content_start + end_rel;
                let json_candidate = text[content_start..content_end].trim();

                match serde_json::from_str::<UiNode>(json_candidate) {
                    Ok(node) => {
                        if block_start > current_offset {
                            segments.push(ContentSegment::Markdown(
                                text[current_offset..block_start].to_string(),
                            ));
                        }
                        segments.push(ContentSegment::GenerativeUi(node));
                        current_offset = content_end + 3; // skip closing ```
                        continue;
                    }
                    Err(_) => {
                        // Not valid JSON, skip this marker occurrence to treat as raw text
                        let next_offset = block_start + marker.len();
                        if next_offset > current_offset {
                            current_offset = next_offset;
                            continue;
                        }
                    }
                }
            }
        }

        // No more valid generative UI blocks
        if current_offset < text.len() {
            segments.push(ContentSegment::Markdown(text[current_offset..].to_string()));
        }
        break;
    }

    segments
}

/// Renders a `UiNode` into native GPUI elements using semantic theme colors and typography.
pub fn render_ui_node<T: 'static>(
    node: &UiNode,
    c: &ThemeColors,
    font_size: f32,
    is_dark: bool,
    on_action: impl Fn(&UiAction, &mut T, &mut Window, &mut Context<T>) + 'static + Clone,
    cx: &mut Context<T>,
) -> AnyElement {
    match node {
        UiNode::Card {
            title,
            subtitle,
            badge,
            child,
        } => {
            let mut header = div().w_full().h_flex().items_center().justify_between();
            let has_header = title.is_some() || subtitle.is_some() || badge.is_some();

            if title.is_some() || subtitle.is_some() {
                let mut title_col = div().v_flex().gap(px(2.));
                if let Some(t) = title {
                    title_col = title_col.child(
                        div()
                            .text_size(px(font_size + 1.))
                            .font_semibold()
                            .text_color(rgb(c.text_primary))
                            .child(t.clone()),
                    );
                }
                if let Some(s) = subtitle {
                    title_col = title_col.child(
                        div()
                            .text_size(px(font_size - 1.))
                            .text_color(rgba(c.foreground_subtle))
                            .child(s.clone()),
                    );
                }
                header = header.child(title_col);
            }

            if let Some(b) = badge {
                header = header.child(render_badge(b, c));
            }

            div()
                .w_full()
                .p_4()
                .rounded(px(12.))
                .bg(rgb(c.card_bg))
                .border_1()
                .border_color(rgba(c.card_border))
                .shadow_xs()
                .v_flex()
                .gap(px(12.))
                .when(has_header, |card| card.child(header))
                .child(render_ui_node(child, c, font_size, is_dark, on_action, cx))
                .into_any_element()
        }
        UiNode::Container {
            direction,
            gap,
            align,
            children,
        } => {
            let mut container = div().w_full();
            match direction {
                LayoutDirection::Column => {
                    container = container.v_flex();
                }
                LayoutDirection::Row => {
                    container = container.h_flex();
                }
            }

            let g = gap.unwrap_or(8.);
            container = container.gap(px(g));

            match align.unwrap_or_default() {
                LayoutAlign::Start => {
                    container = container.items_start();
                }
                LayoutAlign::Center => {
                    container = container.items_center();
                }
                LayoutAlign::End => {
                    container = container.items_end();
                }
                LayoutAlign::Between => {
                    container = container.justify_between();
                }
            }

            let on_action_clone = on_action;
            container
                .children(children.iter().map(|child| {
                    render_ui_node(child, c, font_size, is_dark, on_action_clone.clone(), cx)
                }))
                .into_any_element()
        }
        UiNode::Text { content, variant } => {
            let mut text_el = div().child(content.clone());
            match variant {
                TextVariant::Title => {
                    text_el = text_el
                        .text_size(px(font_size + 4.))
                        .font_semibold()
                        .text_color(rgb(c.text_primary));
                }
                TextVariant::Heading => {
                    text_el = text_el
                        .text_size(px(font_size + 2.))
                        .font_medium()
                        .text_color(rgb(c.text_primary));
                }
                TextVariant::Body => {
                    text_el = text_el
                        .text_size(px(font_size))
                        .text_color(rgb(c.text_primary));
                }
                TextVariant::Subtle => {
                    text_el = text_el
                        .text_size(px(font_size - 1.))
                        .text_color(rgba(c.foreground_subtle));
                }
                TextVariant::Code => {
                    text_el = text_el
                        .text_size(px(font_size - 1.))
                        .font_family(".AppleSystemUIFontMonospaced")
                        .px_1()
                        .rounded(px(4.))
                        .bg(rgba(c.inline_code_bg))
                        .text_color(rgb(c.text_primary));
                }
            }
            text_el.into_any_element()
        }
        UiNode::Badge(badge) => render_badge(badge, c).into_any_element(),
        UiNode::Button {
            id,
            label,
            variant,
            disabled,
        } => {
            let action = UiAction {
                id: id.clone(),
                label: label.clone(),
                payload: None,
            };

            let mut btn = Button::new(format!("gen-ui-btn-{}", id))
                .h(px(30.))
                .rounded(px(8.))
                .px_3()
                .disabled(*disabled)
                .child(
                    div()
                        .text_size(px(font_size - 1.))
                        .font_medium()
                        .child(label.clone()),
                );

            match variant {
                ButtonVariant::Primary => {
                    let brand_btn = if is_dark {
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
                    btn = btn.custom(brand_btn);
                }
                ButtonVariant::Secondary => {
                    btn = btn.secondary();
                }
                ButtonVariant::Outline => {
                    btn = btn.outline();
                }
                ButtonVariant::Ghost => {
                    btn = btn.ghost();
                }
                ButtonVariant::Danger => {
                    btn = btn.danger();
                }
            }

            let on_action_cb = on_action;
            btn.on_click(cx.listener(move |this, _, window, cx| {
                on_action_cb(&action, this, window, cx);
            }))
            .into_any_element()
        }
        UiNode::KeyValueList { items } => div()
            .w_full()
            .v_flex()
            .gap(px(6.))
            .children(items.iter().map(|item| {
                div()
                    .w_full()
                    .h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(font_size - 1.))
                            .text_color(rgba(c.foreground_subtle))
                            .child(item.key.clone()),
                    )
                    .child(
                        div()
                            .h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_size(px(font_size - 1.))
                                    .font_medium()
                                    .text_color(rgb(c.text_primary))
                                    .child(item.value.clone()),
                            )
                            .when_some(item.badge.as_ref(), |row, b| {
                                row.child(
                                    div()
                                        .px(px(6.))
                                        .py(px(1.))
                                        .rounded(px(4.))
                                        .bg(rgba(c.card_user_bg))
                                        .text_size(px(font_size - 2.))
                                        .text_color(rgba(c.foreground_subtle))
                                        .child(b.clone()),
                                )
                            }),
                    )
            }))
            .into_any_element(),
        UiNode::Table { headers, rows } => {
            let col_count = headers.len().max(1);
            let mut table_col = div()
                .w_full()
                .rounded(px(8.))
                .border_1()
                .border_color(rgba(c.card_border))
                .overflow_hidden()
                .v_flex();

            // Header row
            let header_row = div()
                .w_full()
                .h_flex()
                .bg(rgba(c.card_user_bg))
                .py_2()
                .px_3()
                .border_b_1()
                .border_color(rgba(c.card_border))
                .children(headers.iter().map(|h| {
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_size(px(font_size - 2.))
                        .font_medium()
                        .text_color(rgba(c.foreground_subtlest))
                        .child(h.clone())
                }));
            table_col = table_col.child(header_row);

            // Data rows
            for (idx, row) in rows.iter().enumerate() {
                let is_last = idx == rows.len().saturating_sub(1);
                let row_el = div()
                    .w_full()
                    .h_flex()
                    .py_2()
                    .px_3()
                    .when(!is_last, |r| {
                        r.border_b_1().border_color(rgba(c.card_border))
                    })
                    .children((0..col_count).map(|c_idx| {
                        let text = row.get(c_idx).cloned().unwrap_or_default();
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_size(px(font_size - 1.))
                            .text_color(rgb(c.text_primary))
                            .child(text)
                    }));
                table_col = table_col.child(row_el);
            }

            table_col.into_any_element()
        }
        UiNode::Progress {
            value,
            max,
            label,
            tone,
        } => {
            let pct = if *max > 0.0 {
                (value / max).clamp(0.0, 1.0)
            } else {
                0.0
            };

            let fill_color = match tone {
                BadgeTone::Success => rgb(c.accent_green),
                BadgeTone::Warning => rgb(c.workflow_active),
                BadgeTone::Danger => rgb(c.workflow_error),
                BadgeTone::Info => rgb(c.link),
                BadgeTone::Neutral => rgb(c.text_primary),
            };

            div()
                .w_full()
                .v_flex()
                .gap(px(4.))
                .when_some(label.as_ref(), |this, l| {
                    this.child(
                        div()
                            .h_flex()
                            .justify_between()
                            .text_size(px(font_size - 2.))
                            .text_color(rgba(c.foreground_subtle))
                            .child(l.clone())
                            .child(format!("{:.0}%", pct * 100.0)),
                    )
                })
                .child(
                    div()
                        .w_full()
                        .h(px(6.))
                        .rounded(px(3.))
                        .bg(rgba(c.card_user_bg))
                        .child(
                            div()
                                .w(relative(pct))
                                .h_full()
                                .rounded(px(3.))
                                .bg(fill_color),
                        ),
                )
                .into_any_element()
        }
        UiNode::Divider => div()
            .w_full()
            .h(px(1.))
            .bg(rgba(c.card_border))
            .into_any_element(),
    }
}

fn render_badge(badge: &BadgeNode, c: &crate::theme::ThemeColors) -> AnyElement {
    let (bg, border, text_color) = match badge.tone {
        BadgeTone::Neutral => (
            rgba(c.card_user_bg),
            rgba(c.card_border),
            rgba(c.foreground_subtle),
        ),
        BadgeTone::Success => (
            rgba(c.accent_green & 0x00ffffff | 0x20000000),
            rgba(c.accent_green & 0x00ffffff | 0x4d000000),
            rgb(c.accent_green),
        ),
        BadgeTone::Warning => (
            rgba(c.workflow_active & 0x00ffffff | 0x20000000),
            rgba(c.workflow_active & 0x00ffffff | 0x4d000000),
            rgb(c.workflow_active),
        ),
        BadgeTone::Danger => (
            rgba(c.destructive_surface),
            rgba(c.destructive_border),
            rgb(c.workflow_error),
        ),
        BadgeTone::Info => (
            rgba(c.link & 0x00ffffff | 0x20000000),
            rgba(c.link & 0x00ffffff | 0x4d000000),
            rgb(c.link),
        ),
    };

    div()
        .px(px(8.))
        .py(px(2.))
        .rounded(px(6.))
        .bg(bg)
        .border_1()
        .border_color(border)
        .child(
            div()
                .text_size(px(11.))
                .font_medium()
                .text_color(text_color)
                .child(badge.label.clone()),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::{parse_message_segments, BadgeTone, ContentSegment, TextVariant, UiNode};

    #[test]
    fn parses_simple_card_json() {
        let json = r#"{
            "type": "card",
            "title": "任务审批",
            "subtitle": "需要人工复核",
            "badge": { "label": "待处理", "tone": "warning" },
            "child": {
                "type": "container",
                "direction": "column",
                "gap": 8.0,
                "children": [
                    { "type": "text", "content": "确认部署至预发环境？", "variant": "body" },
                    { "type": "button", "id": "btn_approve", "label": "批准部署", "variant": "primary" }
                ]
            }
        }"#;

        let node: Result<UiNode, _> = serde_json::from_str(json);
        assert!(node.is_ok());
        if let Ok(UiNode::Card {
            title,
            badge,
            child,
            ..
        }) = node
        {
            assert_eq!(title, Some("任务审批".to_string()));
            assert_eq!(badge.map(|b| b.tone), Some(BadgeTone::Warning));
            if let UiNode::Container { children, .. } = *child {
                assert_eq!(children.len(), 2);
            } else {
                panic!("Expected Container child");
            }
        } else {
            panic!("Expected Card");
        }
    }

    #[test]
    fn parses_message_segments_with_embedded_ui() {
        let msg = "这是前置文本：\n\n```json-ui\n{\n  \"type\": \"text\",\n  \"content\": \"卡片内容\"\n}\n```\n这是后续说明。";
        let segments = parse_message_segments(msg);
        assert_eq!(segments.len(), 3);
        assert_eq!(
            segments[0],
            ContentSegment::Markdown("这是前置文本：\n\n".to_string())
        );
        assert_eq!(
            segments[1],
            ContentSegment::GenerativeUi(UiNode::Text {
                content: "卡片内容".to_string(),
                variant: TextVariant::Body,
            })
        );
        assert_eq!(
            segments[2],
            ContentSegment::Markdown("\n这是后续说明。".to_string())
        );
    }

    #[test]
    fn ignores_invalid_json_blocks_gracefully() {
        let msg = "普通代码示例：\n```json-ui\n{ invalid json syntax }\n```";
        let segments = parse_message_segments(msg);
        // Falls back to regular markdown
        assert_eq!(segments.len(), 1);
        assert!(matches!(segments[0], ContentSegment::Markdown(_)));
    }
}
