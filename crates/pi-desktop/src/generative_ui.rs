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
        #[serde(default)]
        action: Option<UiAction>,
    },
    ChoiceGroup {
        id: String,
        #[serde(default)]
        label: Option<String>,
        options: Vec<ChoiceOption>,
    },
    Select {
        id: String,
        #[serde(default)]
        label: Option<String>,
        options: Vec<String>,
        #[serde(default)]
        default_value: Option<String>,
        #[serde(default)]
        button_label: Option<String>,
    },
    InputField {
        id: String,
        #[serde(default)]
        label: Option<String>,
        #[serde(default)]
        placeholder: Option<String>,
        #[serde(default)]
        default_value: Option<String>,
        #[serde(default)]
        button_label: Option<String>,
        #[serde(default)]
        action_prompt: Option<String>,
    },
    Toggle {
        id: String,
        label: String,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        checked: bool,
        #[serde(default)]
        action: Option<UiAction>,
    },
    CheckboxGroup {
        id: String,
        #[serde(default)]
        label: Option<String>,
        items: Vec<CheckboxItem>,
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
    Metric {
        label: String,
        value: String,
        #[serde(default)]
        change: Option<String>,
        #[serde(default)]
        change_tone: Option<BadgeTone>,
        #[serde(default)]
        subtext: Option<String>,
    },
    BarChart {
        #[serde(default)]
        title: Option<String>,
        items: Vec<BarItem>,
        #[serde(default)]
        direction: LayoutDirection,
        #[serde(default)]
        max_value: Option<f32>,
    },
    LineChart {
        #[serde(default)]
        title: Option<String>,
        points: Vec<f32>,
        #[serde(default)]
        tone: Option<BadgeTone>,
        #[serde(default)]
        summary: Option<String>,
    },
    Sparkline {
        #[serde(default)]
        title: Option<String>,
        points: Vec<f32>,
        #[serde(default)]
        tone: Option<BadgeTone>,
        #[serde(default)]
        summary: Option<String>,
    },
    Callout {
        #[serde(default)]
        tone: BadgeTone,
        #[serde(default)]
        title: Option<String>,
        message: String,
    },
    Accordion {
        title: String,
        #[serde(default)]
        subtitle: Option<String>,
        child: Box<UiNode>,
    },
    Artifact {
        id: String,
        title: String,
        #[serde(default = "default_artifact_kind")]
        kind: String,
        content: String,
    },
    Divider,
}

fn default_max() -> f32 {
    100.0
}

fn default_artifact_kind() -> String {
    "code".to_string()
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
pub struct ChoiceOption {
    pub label: String,
    pub value: String,
    #[serde(default)]
    pub tone: Option<BadgeTone>,
    #[serde(default)]
    pub prompt: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckboxItem {
    pub label: String,
    pub value: String,
    #[serde(default)]
    pub checked: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BarItem {
    pub label: String,
    pub value: f32,
    #[serde(default)]
    pub display_value: Option<String>,
    #[serde(default)]
    pub tone: Option<BadgeTone>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiAction {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub copy_text: Option<String>,
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
                        current_offset = content_end + 3;
                        continue;
                    }
                    Err(_) => {
                        let next_offset = block_start + marker.len();
                        if next_offset > current_offset {
                            current_offset = next_offset;
                            continue;
                        }
                    }
                }
            }
        }

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
            action,
        } => {
            let act = action.clone().unwrap_or_else(|| UiAction {
                id: id.clone(),
                label: label.clone(),
                prompt: None,
                url: None,
                copy_text: None,
                payload: None,
            });

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
                on_action_cb(&act, this, window, cx);
            }))
            .into_any_element()
        }
        UiNode::ChoiceGroup { id, label, options } => {
            let mut col = div().w_full().v_flex().gap(px(6.));
            if let Some(lbl) = label {
                col = col.child(
                    div()
                        .text_size(px(font_size - 2.))
                        .font_medium()
                        .text_color(rgba(c.foreground_subtle))
                        .child(lbl.clone()),
                );
            }
            let mut row = div().w_full().h_flex().flex_wrap().gap(px(6.));
            for opt in options {
                let opt_val = opt.value.clone();
                let opt_lbl = opt.label.clone();
                let act = UiAction {
                    id: format!("{}:{}", id, opt_val),
                    label: opt_lbl.clone(),
                    prompt: opt
                        .prompt
                        .clone()
                        .or_else(|| Some(format!("选择【{}】", opt_lbl))),
                    url: None,
                    copy_text: None,
                    payload: Some(serde_json::json!({ "id": id, "value": opt_val })),
                };
                let on_action_cb = on_action.clone();
                let opt_btn = Button::new(format!("choice-{}-{}", id, opt_val))
                    .h(px(28.))
                    .rounded(px(6.))
                    .px_3()
                    .secondary()
                    .child(
                        div()
                            .text_size(px(font_size - 1.))
                            .font_medium()
                            .child(opt_lbl),
                    )
                    .on_click(cx.listener(move |this, _, window, cx| {
                        on_action_cb(&act, this, window, cx);
                    }));
                row = row.child(opt_btn);
            }
            col.child(row).into_any_element()
        }
        UiNode::Select {
            id,
            label,
            options,
            default_value,
            button_label,
        } => {
            let mut col = div().w_full().v_flex().gap(px(6.));
            if let Some(lbl) = label {
                col = col.child(
                    div()
                        .text_size(px(font_size - 2.))
                        .font_medium()
                        .text_color(rgba(c.foreground_subtle))
                        .child(lbl.clone()),
                );
            }
            let selected_val = default_value
                .as_deref()
                .or_else(|| options.first().map(|s| s.as_str()))
                .unwrap_or("")
                .to_string();
            let btn_txt = button_label.as_deref().unwrap_or("确认");
            let act = UiAction {
                id: id.clone(),
                label: format!("选择: {}", selected_val),
                prompt: Some(format!("已选择【{}】", selected_val)),
                url: None,
                copy_text: None,
                payload: Some(serde_json::json!({ "id": id, "selected": selected_val })),
            };
            let on_action_cb = on_action;

            let select_box = div()
                .w_full()
                .h_flex()
                .items_center()
                .justify_between()
                .gap(px(8.))
                .p_2()
                .rounded(px(8.))
                .bg(rgba(c.card_user_bg))
                .border_1()
                .border_color(rgba(c.card_border))
                .child(
                    div()
                        .h_flex()
                        .items_center()
                        .gap(px(6.))
                        .child(
                            div()
                                .text_size(px(font_size - 1.))
                                .font_medium()
                                .text_color(rgb(c.text_primary))
                                .child(selected_val),
                        )
                        .child(
                            div()
                                .text_size(px(font_size - 2.))
                                .text_color(rgba(c.foreground_subtle))
                                .child("▾"),
                        ),
                )
                .child(
                    Button::new(format!("gen-ui-sel-btn-{}", id))
                        .h(px(26.))
                        .rounded(px(6.))
                        .px_2()
                        .secondary()
                        .child(
                            div()
                                .text_size(px(font_size - 2.))
                                .font_medium()
                                .child(btn_txt.to_string()),
                        )
                        .on_click(cx.listener(move |this, _, window, cx| {
                            on_action_cb(&act, this, window, cx);
                        })),
                );
            col.child(select_box).into_any_element()
        }
        UiNode::InputField {
            id,
            label,
            placeholder,
            default_value,
            button_label,
            action_prompt,
        } => {
            let mut col = div().w_full().v_flex().gap(px(6.));
            if let Some(lbl) = label {
                col = col.child(
                    div()
                        .text_size(px(font_size - 2.))
                        .font_medium()
                        .text_color(rgba(c.foreground_subtle))
                        .child(lbl.clone()),
                );
            }
            let btn_txt = button_label.as_deref().unwrap_or("提交");
            let prompt_val = action_prompt
                .clone()
                .unwrap_or_else(|| default_value.clone().unwrap_or_default());
            let act = UiAction {
                id: id.clone(),
                label: btn_txt.to_string(),
                prompt: Some(prompt_val),
                url: None,
                copy_text: None,
                payload: Some(serde_json::json!({ "id": id, "value": default_value })),
            };
            let display_val = default_value
                .as_deref()
                .or(placeholder.as_deref())
                .unwrap_or("");
            let on_action_cb = on_action;

            let input_box = div()
                .w_full()
                .h_flex()
                .items_center()
                .justify_between()
                .gap(px(8.))
                .p_2()
                .rounded(px(8.))
                .bg(rgba(c.card_user_bg))
                .border_1()
                .border_color(rgba(c.card_border))
                .child(
                    div()
                        .flex_1()
                        .text_size(px(font_size - 1.))
                        .text_color(if default_value.is_some() {
                            rgb(c.text_primary)
                        } else {
                            rgba(c.foreground_subtlest)
                        })
                        .child(display_val.to_string()),
                )
                .child(
                    Button::new(format!("gen-ui-input-btn-{}", id))
                        .h(px(26.))
                        .rounded(px(6.))
                        .px_2()
                        .secondary()
                        .child(
                            div()
                                .text_size(px(font_size - 2.))
                                .font_medium()
                                .child(btn_txt.to_string()),
                        )
                        .on_click(cx.listener(move |this, _, window, cx| {
                            on_action_cb(&act, this, window, cx);
                        })),
                );
            col.child(input_box).into_any_element()
        }
        UiNode::Toggle {
            id,
            label,
            description,
            checked,
            action,
        } => {
            let act = action.clone().unwrap_or_else(|| UiAction {
                id: id.clone(),
                label: label.clone(),
                prompt: Some(format!(
                    "切换【{}】状态为：{}",
                    label,
                    if !checked { "开启" } else { "关闭" }
                )),
                url: None,
                copy_text: None,
                payload: Some(serde_json::json!({ "id": id, "checked": !checked })),
            });
            let on_action_cb = on_action;
            let knob = div()
                .w(px(10.))
                .h(px(10.))
                .rounded_full()
                .bg(if *checked {
                    rgb(c.accent_green)
                } else {
                    rgba(c.foreground_subtle)
                });

            let toggle_btn = Button::new(format!("toggle-{}", id))
                .h(px(26.))
                .rounded(px(6.))
                .px_2()
                .ghost()
                .child(
                    div()
                        .h_flex()
                        .items_center()
                        .gap(px(6.))
                        .child(knob)
                        .child(
                            div()
                                .text_size(px(11.))
                                .font_medium()
                                .child(if *checked { "已开启" } else { "已关闭" }),
                        ),
                )
                .on_click(cx.listener(move |this, _, window, cx| {
                    on_action_cb(&act, this, window, cx);
                }));

            div()
                .w_full()
                .h_flex()
                .items_center()
                .justify_between()
                .py_1()
                .child(
                    div()
                        .v_flex()
                        .gap(px(2.))
                        .child(
                            div()
                                .text_size(px(font_size - 1.))
                                .font_medium()
                                .text_color(rgb(c.text_primary))
                                .child(label.clone()),
                        )
                        .when_some(description.as_ref(), |this, desc| {
                            this.child(
                                div()
                                    .text_size(px(font_size - 2.))
                                    .text_color(rgba(c.foreground_subtle))
                                    .child(desc.clone()),
                            )
                        }),
                )
                .child(toggle_btn)
                .into_any_element()
        }
        UiNode::CheckboxGroup { id, label, items } => {
            let mut col = div().w_full().v_flex().gap(px(6.));
            if let Some(lbl) = label {
                col = col.child(
                    div()
                        .text_size(px(font_size - 2.))
                        .font_medium()
                        .text_color(rgba(c.foreground_subtle))
                        .child(lbl.clone()),
                );
            }
            let mut row = div().w_full().h_flex().flex_wrap().gap(px(6.));
            for item in items {
                let itm_val = item.value.clone();
                let itm_lbl = item.label.clone();
                let itm_checked = item.checked;
                let act = UiAction {
                    id: format!("{}:{}", id, itm_val),
                    label: itm_lbl.clone(),
                    prompt: Some(format!(
                        "选择复选【{}】（{}）",
                        itm_lbl,
                        if !itm_checked { "勾选" } else { "取消勾选" }
                    )),
                    url: None,
                    copy_text: None,
                    payload: Some(
                        serde_json::json!({ "id": id, "value": itm_val, "checked": !itm_checked }),
                    ),
                };
                let on_action_cb = on_action.clone();
                let check_icon = if itm_checked { "☑ " } else { "☐ " };
                let chk_btn = Button::new(format!("chk-{}-{}", id, itm_val))
                    .h(px(26.))
                    .rounded(px(6.))
                    .px_2()
                    .when(itm_checked, |b| b.secondary())
                    .when(!itm_checked, |b| b.ghost())
                    .child(
                        div()
                            .text_size(px(font_size - 1.))
                            .child(format!("{}{}", check_icon, itm_lbl)),
                    )
                    .on_click(cx.listener(move |this, _, window, cx| {
                        on_action_cb(&act, this, window, cx);
                    }));
                row = row.child(chk_btn);
            }
            col.child(row).into_any_element()
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
        UiNode::Metric {
            label,
            value,
            change,
            change_tone,
            subtext,
        } => div()
            .w_full()
            .v_flex()
            .gap(px(4.))
            .child(
                div()
                    .w_full()
                    .h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(font_size - 2.))
                            .font_medium()
                            .text_color(rgba(c.foreground_subtle))
                            .child(label.clone()),
                    )
                    .when_some(change.as_ref(), |row, ch| {
                        let tone = change_tone.unwrap_or(BadgeTone::Neutral);
                        row.child(render_badge(
                            &BadgeNode {
                                label: ch.clone(),
                                tone,
                            },
                            c,
                        ))
                    }),
            )
            .child(
                div()
                    .text_size(px(font_size + 8.))
                    .font_bold()
                    .text_color(rgb(c.text_primary))
                    .child(value.clone()),
            )
            .when_some(subtext.as_ref(), |col, st| {
                col.child(
                    div()
                        .text_size(px(font_size - 2.))
                        .text_color(rgba(c.foreground_subtlest))
                        .child(st.clone()),
                )
            })
            .into_any_element(),
        UiNode::BarChart {
            title,
            items,
            direction,
            max_value,
        } => {
            let mut chart = div().w_full().v_flex().gap(px(8.));
            if let Some(t) = title {
                chart = chart.child(
                    div()
                        .text_size(px(font_size))
                        .font_semibold()
                        .text_color(rgb(c.text_primary))
                        .child(t.clone()),
                );
            }
            let max_val = max_value
                .unwrap_or_else(|| items.iter().map(|i| i.value).fold(0.0, f32::max))
                .max(0.001);

            match direction {
                LayoutDirection::Row => {
                    let bars = div()
                        .w_full()
                        .v_flex()
                        .gap(px(6.))
                        .children(items.iter().map(|item| {
                            let ratio = (item.value / max_val).clamp(0.0, 1.0);
                            let fill = match item.tone.unwrap_or(BadgeTone::Info) {
                                BadgeTone::Success => rgb(c.accent_green),
                                BadgeTone::Warning => rgb(c.workflow_active),
                                BadgeTone::Danger => rgb(c.workflow_error),
                                BadgeTone::Info => rgb(c.link),
                                BadgeTone::Neutral => rgb(c.text_primary),
                            };
                            let display = item
                                .display_value
                                .clone()
                                .unwrap_or_else(|| format!("{:.1}", item.value));

                            div()
                                .w_full()
                                .h_flex()
                                .items_center()
                                .gap(px(8.))
                                .child(
                                    div()
                                        .w(px(70.))
                                        .text_size(px(font_size - 2.))
                                        .text_color(rgba(c.foreground_subtle))
                                        .child(item.label.clone()),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .h(px(10.))
                                        .rounded(px(5.))
                                        .bg(rgba(c.card_user_bg))
                                        .child(
                                            div()
                                                .w(relative(ratio))
                                                .h_full()
                                                .rounded(px(5.))
                                                .bg(fill),
                                        ),
                                )
                                .child(
                                    div()
                                        .w(px(40.))
                                        .text_size(px(font_size - 2.))
                                        .font_medium()
                                        .text_color(rgb(c.text_primary))
                                        .child(display),
                                )
                        }));
                    chart = chart.child(bars);
                }
                LayoutDirection::Column => {
                    let bars = div()
                        .w_full()
                        .h(px(80.))
                        .h_flex()
                        .items_end()
                        .justify_around()
                        .gap(px(6.))
                        .children(items.iter().map(|item| {
                            let ratio = (item.value / max_val).clamp(0.0, 1.0);
                            let fill = match item.tone.unwrap_or(BadgeTone::Info) {
                                BadgeTone::Success => rgb(c.accent_green),
                                BadgeTone::Warning => rgb(c.workflow_active),
                                BadgeTone::Danger => rgb(c.workflow_error),
                                BadgeTone::Info => rgb(c.link),
                                BadgeTone::Neutral => rgb(c.text_primary),
                            };
                            div()
                                .flex_1()
                                .h_full()
                                .v_flex()
                                .items_center()
                                .justify_end()
                                .gap(px(4.))
                                .child(
                                    div()
                                        .w_full()
                                        .max_w(px(24.))
                                        .h(relative(ratio))
                                        .rounded(px(4.))
                                        .bg(fill),
                                )
                                .child(
                                    div()
                                        .text_size(px(font_size - 2.))
                                        .text_color(rgba(c.foreground_subtle))
                                        .child(item.label.clone()),
                                )
                        }));
                    chart = chart.child(bars);
                }
            }
            chart.into_any_element()
        }
        UiNode::LineChart {
            title,
            points,
            tone,
            summary,
        } => {
            let mut wrap = div().w_full().v_flex().gap(px(8.));
            if title.is_some() || summary.is_some() {
                let mut row = div().w_full().h_flex().items_center().justify_between();
                if let Some(t) = title {
                    row = row.child(
                        div()
                            .text_size(px(font_size))
                            .font_semibold()
                            .text_color(rgb(c.text_primary))
                            .child(t.clone()),
                    );
                }
                if let Some(s) = summary {
                    row = row.child(
                        div()
                            .text_size(px(font_size - 1.))
                            .font_medium()
                            .text_color(rgba(c.foreground_subtle))
                            .child(s.clone()),
                    );
                }
                wrap = wrap.child(row);
            }

            if points.is_empty() {
                return wrap.into_any_element();
            }

            let min_val = points.iter().copied().fold(f32::INFINITY, f32::min);
            let max_val = points.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            let val_range = (max_val - min_val).max(0.001);

            let tone_color = match tone.unwrap_or(BadgeTone::Info) {
                BadgeTone::Success => c.accent_green,
                BadgeTone::Warning => c.workflow_active,
                BadgeTone::Danger => c.workflow_error,
                BadgeTone::Info => c.link,
                BadgeTone::Neutral => c.text_primary,
            };
            let stroke_color = rgb(tone_color);
            let fill_color = rgba((tone_color & 0x00ffffff) | 0x26000000);

            let pts = points.clone();
            let chart_height = 110.0f32;

            let chart_canvas = gpui::canvas(
                |_, _, _| (),
                move |bounds, _, window, _| {
                    let n = pts.len();
                    if n < 2 {
                        return;
                    }
                    let w = f32::from(bounds.size.width);
                    let h = f32::from(bounds.size.height);
                    let padding_y = 12.0f32;
                    let eff_h = (h - padding_y * 2.0).max(10.0);

                    let coord = |i: usize| -> Point<Pixels> {
                        let x = (i as f32 / (n - 1) as f32) * w;
                        let norm = (pts[i] - min_val) / val_range;
                        let y = h - padding_y - norm * eff_h;
                        bounds.origin + point(px(x), px(y))
                    };

                    let mut area = gpui::PathBuilder::fill();
                    area.move_to(bounds.origin + point(px(0.), px(h)));
                    for i in 0..n {
                        area.line_to(coord(i));
                    }
                    area.line_to(bounds.origin + point(px(w), px(h)));
                    area.close();

                    if let Ok(path) = area.build() {
                        window.paint_path(path, fill_color);
                    }

                    for i in 0..n {
                        let pt = coord(i);
                        let dot_bounds = Bounds {
                            origin: pt - point(px(3.), px(3.)),
                            size: size(px(6.), px(6.)),
                        };
                        window.paint_quad(gpui::fill(dot_bounds, stroke_color));
                    }
                },
            )
            .w_full()
            .h(px(chart_height))
            .rounded(px(8.))
            .bg(rgba(c.card_user_bg))
            .border_1()
            .border_color(rgba(c.card_border));

            let footer = div()
                .w_full()
                .h_flex()
                .justify_between()
                .text_size(px(font_size - 2.))
                .text_color(rgba(c.foreground_subtlest))
                .child(format!("最小值: {:.1}", min_val))
                .child(format!(
                    "均值: {:.1}",
                    points.iter().sum::<f32>() / points.len() as f32
                ))
                .child(format!("峰值: {:.1}", max_val));

            wrap.child(chart_canvas).child(footer).into_any_element()
        }
        UiNode::Sparkline {
            title,
            points,
            tone,
            summary,
        } => {
            let mut wrap = div().w_full().v_flex().gap(px(4.));
            if let Some(t) = title {
                wrap = wrap.child(
                    div()
                        .w_full()
                        .h_flex()
                        .justify_between()
                        .child(
                            div()
                                .text_size(px(font_size - 2.))
                                .font_medium()
                                .text_color(rgba(c.foreground_subtle))
                                .child(t.clone()),
                        )
                        .when_some(summary.as_ref(), |row, s| {
                            row.child(
                                div()
                                    .text_size(px(font_size - 2.))
                                    .text_color(rgb(c.text_primary))
                                    .child(s.clone()),
                            )
                        }),
                );
            }
            let max_val = points.iter().copied().fold(0.0, f32::max).max(0.001);
            let fill = match tone.unwrap_or(BadgeTone::Info) {
                BadgeTone::Success => rgb(c.accent_green),
                BadgeTone::Warning => rgb(c.workflow_active),
                BadgeTone::Danger => rgb(c.workflow_error),
                BadgeTone::Info => rgb(c.link),
                BadgeTone::Neutral => rgb(c.text_primary),
            };

            let bars = div()
                .w_full()
                .h(px(28.))
                .h_flex()
                .items_end()
                .gap(px(2.))
                .children(points.iter().map(|&val| {
                    let ratio = (val / max_val).clamp(0.08, 1.0);
                    div()
                        .flex_1()
                        .h(relative(ratio))
                        .rounded(px(2.))
                        .bg(fill)
                }));
            wrap.child(bars).into_any_element()
        }
        UiNode::Callout {
            tone,
            title,
            message,
        } => {
            let (bg, border, text_color) = match tone {
                BadgeTone::Neutral => (
                    rgba(c.card_user_bg),
                    rgba(c.card_border),
                    rgb(c.text_primary),
                ),
                BadgeTone::Success => (
                    rgba(c.accent_green & 0x00ffffff | 0x18000000),
                    rgb(c.accent_green),
                    rgb(c.accent_green),
                ),
                BadgeTone::Warning => (
                    rgba(c.workflow_active & 0x00ffffff | 0x18000000),
                    rgb(c.workflow_active),
                    rgb(c.workflow_active),
                ),
                BadgeTone::Danger => (
                    rgba(c.destructive_surface),
                    rgb(c.workflow_error),
                    rgb(c.workflow_error),
                ),
                BadgeTone::Info => (
                    rgba(c.link & 0x00ffffff | 0x18000000),
                    rgb(c.link),
                    rgb(c.link),
                ),
            };

            div()
                .w_full()
                .h_flex()
                .rounded(px(8.))
                .bg(bg)
                .border_1()
                .border_color(border)
                .overflow_hidden()
                .child(div().w(px(4.)).h_full().bg(border))
                .child(
                    div()
                        .flex_1()
                        .p_3()
                        .v_flex()
                        .gap(px(4.))
                        .when_some(title.as_ref(), |col, t| {
                            col.child(
                                div()
                                    .text_size(px(font_size))
                                    .font_semibold()
                                    .text_color(text_color)
                                    .child(t.clone()),
                            )
                        })
                        .child(
                            div()
                                .text_size(px(font_size - 1.))
                                .text_color(rgb(c.text_primary))
                                .child(message.clone()),
                        ),
                )
                .into_any_element()
        }
        UiNode::Accordion {
            title,
            subtitle,
            child,
        } => div()
            .w_full()
            .rounded(px(8.))
            .border_1()
            .border_color(rgba(c.card_border))
            .overflow_hidden()
            .v_flex()
            .child(
                div()
                    .w_full()
                    .h_flex()
                    .items_center()
                    .justify_between()
                    .p_3()
                    .bg(rgba(c.card_user_bg))
                    .child(
                        div()
                            .h_flex()
                            .items_center()
                            .gap(px(6.))
                            .child(
                                div()
                                    .text_size(px(font_size - 2.))
                                    .text_color(rgba(c.foreground_subtle))
                                    .child("▼"),
                            )
                            .child(
                                div()
                                    .text_size(px(font_size - 1.))
                                    .font_medium()
                                    .text_color(rgb(c.text_primary))
                                    .child(title.clone()),
                            )
                            .when_some(subtitle.as_ref(), |row, sub| {
                                row.child(
                                    div()
                                        .text_size(px(font_size - 2.))
                                        .text_color(rgba(c.foreground_subtle))
                                        .child(format!("({})", sub)),
                                )
                            }),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .p_3()
                    .child(render_ui_node(child, c, font_size, is_dark, on_action, cx)),
            )
            .into_any_element(),
        UiNode::Artifact {
            id,
            title,
            kind,
            content,
        } => {
            let line_count = content.lines().count().max(1);
            let copy_act = UiAction {
                id: format!("copy-art-{}", id),
                label: "复制代码".to_string(),
                prompt: None,
                url: None,
                copy_text: Some(content.clone()),
                payload: None,
            };
            let sidebar_act = UiAction {
                id: format!("sidebar-art-{}", id),
                label: format!("侧栏预览: {}", title),
                prompt: None,
                url: None,
                copy_text: None,
                payload: Some(serde_json::json!({
                    "action": "open_sidebar",
                    "id": id,
                    "title": title,
                    "kind": kind,
                    "content": content
                })),
            };
            let is_web = matches!(kind.to_lowercase().as_str(), "html" | "html5" | "canvas" | "svg" | "web");
            let web_act = if is_web {
                Some(UiAction {
                    id: format!("web-art-{}", id),
                    label: format!("独立 Web 视窗: {}", title),
                    prompt: None,
                    url: None,
                    copy_text: None,
                    payload: Some(serde_json::json!({
                        "action": "open_web_window",
                        "id": id,
                        "title": title,
                        "kind": kind,
                        "content": content
                    })),
                })
            } else {
                None
            };

            let on_action_copy = on_action.clone();
            let on_action_sidebar = on_action.clone();
            let on_action_web = on_action;

            let header = div()
                .w_full()
                .h_flex()
                .items_center()
                .justify_between()
                .px_3()
                .py_2()
                .bg(rgba(c.card_user_bg))
                .border_b_1()
                .border_color(rgba(c.card_border))
                .child(
                    div()
                        .h_flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .px(px(6.))
                                .py(px(1.))
                                .rounded(px(4.))
                                .bg(rgba(c.hover))
                                .text_size(px(10.))
                                .font_semibold()
                                .text_color(rgb(c.link))
                                .child(kind.to_uppercase()),
                        )
                        .child(
                            div()
                                .text_size(px(font_size - 1.))
                                .font_medium()
                                .text_color(rgb(c.text_primary))
                                .child(title.clone()),
                        )
                        .child(
                            div()
                                .text_size(px(font_size - 2.))
                                .text_color(rgba(c.foreground_subtle))
                                .child(format!("{} 行", line_count)),
                        ),
                )
                .child(
                    div()
                        .h_flex()
                        .items_center()
                        .gap_2()
                        .child(
                            Button::new(format!("btn-copy-art-{}", id))
                                .h(px(24.))
                                .rounded(px(6.))
                                .px_2()
                                .ghost()
                                .child(
                                    div()
                                        .text_size(px(11.))
                                        .child("复制代码".to_string()),
                                )
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    on_action_copy(&copy_act, this, window, cx);
                                })),
                        )
                        .child(
                            Button::new(format!("btn-sidebar-art-{}", id))
                                .h(px(24.))
                                .rounded(px(6.))
                                .px_2()
                                .secondary()
                                .child(
                                    div()
                                        .text_size(px(11.))
                                        .child("侧栏预览".to_string()),
                                )
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    on_action_sidebar(&sidebar_act, this, window, cx);
                                })),
                        )
                        .when_some(web_act, |actions, act| {
                            actions.child(
                                Button::new(format!("btn-web-art-{}", id))
                                    .h(px(24.))
                                    .rounded(px(6.))
                                    .px_2()
                                    .outline()
                                    .child(
                                        div()
                                            .text_size(px(11.))
                                            .child("独立Web视窗".to_string()),
                                    )
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        on_action_web(&act, this, window, cx);
                                    })),
                            )
                        }),
                );

            let body = div()
                .w_full()
                .max_h(px(220.))
                .p_3()
                .bg(rgba(c.inline_code_bg))
                .overflow_hidden()
                .child(
                    div()
                        .text_size(px(font_size - 1.))
                        .font_family(".AppleSystemUIFontMonospaced")
                        .text_color(rgb(c.text_primary))
                        .child(content.clone()),
                );

            div()
                .w_full()
                .rounded(px(8.))
                .border_1()
                .border_color(rgba(c.card_border))
                .overflow_hidden()
                .v_flex()
                .child(header)
                .child(body)
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
    use super::{
        parse_message_segments, BadgeTone, ContentSegment, LayoutDirection, TextVariant, UiNode,
    };

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
        assert_eq!(segments.len(), 1);
        assert!(matches!(segments[0], ContentSegment::Markdown(_)));
    }

    #[test]
    fn parses_interactive_form_controls() {
        let json = r#"{
            "type": "choice_group",
            "id": "env_select",
            "label": "选择部署目标环境",
            "options": [
                { "label": "开发环境", "value": "dev" },
                { "label": "预发环境", "value": "staging" },
                { "label": "生产环境", "value": "prod", "tone": "danger" }
            ]
        }"#;
        let node: Result<UiNode, _> = serde_json::from_str(json);
        assert!(node.is_ok());
        if let Ok(UiNode::ChoiceGroup { options, .. }) = node {
            assert_eq!(options.len(), 3);
            assert_eq!(options[2].tone, Some(BadgeTone::Danger));
        } else {
            panic!("Expected ChoiceGroup");
        }

        let sel_json = r#"{
            "type": "select",
            "id": "db_replica",
            "label": "数据库副本数量",
            "options": ["1 副本", "3 副本 (推荐)", "5 副本"],
            "default_value": "3 副本 (推荐)"
        }"#;
        let sel_node: Result<UiNode, _> = serde_json::from_str(sel_json);
        assert!(sel_node.is_ok());

        let chk_json = r#"{
            "type": "checkbox_group",
            "id": "plugins",
            "label": "选择安装插件",
            "items": [
                { "label": "Redis 缓存", "value": "redis", "checked": true },
                { "label": "Prometheus 监控", "value": "prom", "checked": false }
            ]
        }"#;
        let chk_node: Result<UiNode, _> = serde_json::from_str(chk_json);
        assert!(chk_node.is_ok());
    }

    #[test]
    fn parses_data_visualization_and_metrics() {
        let metric_json = r#"{
            "type": "metric",
            "label": "当前 CPU 使用率",
            "value": "78.4%",
            "change": "+5.2%",
            "change_tone": "warning",
            "subtext": "最近 5 分钟均值"
        }"#;
        let metric: Result<UiNode, _> = serde_json::from_str(metric_json);
        assert!(metric.is_ok());

        let chart_json = r#"{
            "type": "bar_chart",
            "title": "服务响应时间",
            "direction": "row",
            "items": [
                { "label": "API 网关", "value": 24.5, "display_value": "24.5ms", "tone": "success" },
                { "label": "认证中心", "value": 112.0, "display_value": "112ms", "tone": "danger" }
            ]
        }"#;
        let chart: Result<UiNode, _> = serde_json::from_str(chart_json);
        assert!(chart.is_ok());
        if let Ok(UiNode::BarChart {
            direction, items, ..
        }) = chart
        {
            assert_eq!(direction, LayoutDirection::Row);
            assert_eq!(items.len(), 2);
        } else {
            panic!("Expected BarChart");
        }

        let line_json = r#"{
            "type": "line_chart",
            "title": "QPS 趋势图",
            "points": [120.0, 145.0, 190.0, 310.0, 280.0, 420.0],
            "tone": "info",
            "summary": "最高 420 QPS"
        }"#;
        let line_node: Result<UiNode, _> = serde_json::from_str(line_json);
        assert!(line_node.is_ok());
    }

    #[test]
    fn parses_callout_accordion_and_artifact() {
        let callout_json = r#"{
            "type": "callout",
            "tone": "success",
            "title": "部署成功",
            "message": "所有节点已通过就绪探针检查。"
        }"#;
        let callout: Result<UiNode, _> = serde_json::from_str(callout_json);
        assert!(callout.is_ok());

        let artifact_json = r#"{
            "type": "artifact",
            "id": "nginx_conf",
            "title": "反向代理配置",
            "kind": "nginx",
            "content": "server {\n  listen 80;\n  server_name volt.local;\n}"
        }"#;
        let artifact: Result<UiNode, _> = serde_json::from_str(artifact_json);
        assert!(artifact.is_ok());
    }
}
