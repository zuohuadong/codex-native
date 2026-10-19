use std::path::PathBuf;
use std::sync::Arc;
use gpui_kit::gpui::{
    div, px, rgb, App, AppContext, Context, Entity, FocusHandle, Focusable,
    InteractiveElement, IntoElement, KeyBinding, KeyDownEvent, ParentElement,
    Render, Styled, Subscription, View, ViewContext, VisualContext, WindowOptions,
};
use pi_agent_core::{AgentEvent, Message, ModelConfig, PiAgent, Session};
use tokio::sync::mpsc;

struct PiDesktopApp {
    workspace_dir: PathBuf,
    session: Session,
    agent: Arc<PiAgent>,
    input_text: String,
    is_running: bool,
    status_message: String,
    focus_handle: FocusHandle,
}

impl PiDesktopApp {
    fn new(cx: &mut ViewContext<Self>) -> Self {
        let workspace_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let model_config = ModelConfig::default();
        let agent = Arc::new(PiAgent::new(workspace_dir.clone(), model_config));

        Self {
            workspace_dir,
            session: Session::new("default", "New Chat Session"),
            agent,
            input_text: String::new(),
            is_running: false,
            status_message: "Ready".to_string(),
            focus_handle: cx.focus_handle(),
        }
    }

    fn submit_prompt(&mut self, prompt: String, cx: &mut ViewContext<Self>) {
        let prompt = prompt.trim().to_string();
        if prompt.is_empty() || self.is_running {
            return;
        }

        self.input_text.clear();
        self.is_running = true;
        self.status_message = "Thinking...".to_string();
        self.session.messages.push(Message::user(&prompt));
        cx.notify();

        let agent = self.agent.clone();
        let mut messages = self.session.messages.clone();
        let (tx, mut rx) = mpsc::channel::<AgentEvent>(100);

        // Spawn agent worker on tokio thread pool
        cx.spawn(|this, mut cx| async move {
            let agent_task = tokio::spawn(async move {
                agent.run_turn(&mut messages, tx).await
            });

            // Listen to streaming events
            while let Some(event) = rx.recv().await {
                let _ = this.update(&mut cx, |app, cx| {
                    match event {
                        AgentEvent::TextDelta(delta) => {
                            if let Some(last) = app.session.messages.last_mut() {
                                if last.role == pi_agent_core::Role::Assistant {
                                    last.content.push_str(&delta);
                                } else {
                                    app.session.messages.push(Message::assistant(&delta));
                                }
                            } else {
                                app.session.messages.push(Message::assistant(&delta));
                            }
                            app.status_message = "Generating response...".to_string();
                        }
                        AgentEvent::ToolCallStarted { name, arguments, .. } => {
                            app.status_message = format!("Running tool: {}...", name);
                            app.session.messages.push(Message::assistant(format!(
                                "> [Tool Execution: `{}`]\nArgs: `{}`",
                                name, arguments
                            )));
                        }
                        AgentEvent::ToolCallFinished { name, output, is_error, .. } => {
                            let prefix = if is_error { "Error" } else { "Result" };
                            app.status_message = format!("Tool {} finished ({})", name, prefix);
                        }
                        AgentEvent::Error(err) => {
                            app.status_message = format!("Error: {}", err);
                        }
                        _ => {}
                    }
                    cx.notify();
                });
            }

            let _ = agent_task.await;

            let _ = this.update(&mut cx, |app, cx| {
                app.is_running = false;
                app.status_message = "Idle".to_string();
                cx.notify();
            });
        }).detach();
    }
}

impl Focusable for PiDesktopApp {
    fn focus_handle(&self, _: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PiDesktopApp {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let ws_str = self.workspace_dir.display().to_string();
        let status = self.status_message.clone();
        let is_running = self.is_running;

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x18181b)) // zinc-900
            .text_color(rgb(0xf4f4f5)) // zinc-100
            // Header bar
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(px(16.0))
                    .py(px(10.0))
                    .bg(rgb(0x27272a)) // zinc-800
                    .border_b_1()
                    .border_color(rgb(0x3f3f46)) // zinc-700
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(div().font_weight(gpui_kit::gpui::FontWeight::BOLD).child("⚡ Pi Desktop"))
                            .child(div().text_color(rgb(0xa1a1aa)).text_xs().child(format!("({})", ws_str))),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .px(px(8.0))
                                    .py(px(2.0))
                                    .rounded(px(4.0))
                                    .bg(if is_running { rgb(0x0284c7) } else { rgb(0x3f3f46) })
                                    .text_xs()
                                    .child(status),
                            ),
                    ),
            )
            // Main content area: Message list
            .child(
                div()
                    .flex_1()
                    .overflow_y_scroll()
                    .p(px(16.0))
                    .flex()
                    .flex_col()
                    .gap(px(12.0))
                    .children(self.session.messages.iter().map(|msg| {
                        let is_user = msg.role == pi_agent_core::Role::User;
                        div()
                            .flex()
                            .flex_col()
                            .p(px(10.0))
                            .rounded(px(6.0))
                            .bg(if is_user { rgb(0x27272a) } else { rgb(0x1e1e24) })
                            .border_1()
                            .border_color(if is_user { rgb(0x3f3f46) } else { rgb(0x2e2e38) })
                            .child(
                                div()
                                    .font_weight(gpui_kit::gpui::FontWeight::SEMIBOLD)
                                    .text_xs()
                                    .text_color(if is_user { rgb(0x60a5fa) } else { rgb(0x34d399) })
                                    .child(if is_user { "You" } else { "Pi Agent" }),
                            )
                            .child(
                                div()
                                    .pt(px(4.0))
                                    .text_sm()
                                    .child(msg.content.clone()),
                            )
                    })),
            )
            // Input prompt footer
            .child(
                div()
                    .p(px(16.0))
                    .border_t_1()
                    .border_color(rgb(0x3f3f46))
                    .bg(rgb(0x202024))
                    .flex()
                    .gap(px(10.0))
                    .child(
                        div()
                            .flex_1()
                            .p(px(8.0))
                            .rounded(px(6.0))
                            .bg(rgb(0x27272a))
                            .border_1()
                            .border_color(rgb(0x3f3f46))
                            .text_sm()
                            .text_color(rgb(0xa1a1aa))
                            .child("Type your instruction and press Enter... (Pi Rust Engine)"),
                    ),
            )
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    gpui_kit::application().run(|cx| {
        gpui_kit::init(cx);

        let window_options = WindowOptions {
            titlebar: Some(gpui_kit::gpui::TitlebarOptions {
                title: Some("Pi Desktop - Native AI Coding Agent".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            ..Default::default()
        };

        gpui_kit::open_window(window_options, cx, |_, cx| {
            cx.new(PiDesktopApp::new)
        })
        .expect("Failed to open main window");
    });
}
