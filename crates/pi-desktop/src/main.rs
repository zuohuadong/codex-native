use gpui_kit::component::input::{Input, InputEvent, InputState, Textarea, TextareaState};
use gpui_kit::component::select::{SelectEvent, SelectState};
use gpui_kit::component::*;
use gpui_kit::*;
use pi_agent_core::skills::{SkillCategory, SkillRegistry};
use pi_agent_core::{AgentEvent, Message, ModelConfig, PiAgent, Session};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
mod desktop_state;
mod execution;
mod file_summary;
mod generative_ui;
mod i18n;
mod installer;
mod interactions;
mod panels;
mod review;
mod settings;
mod task_state;
mod theme;
mod transcript_scroll;
mod workbench;
mod workflow_pane;
mod workflow_timeline;
mod workflow_view;
use i18n::{I18nText, Language};
use theme::{ThemeColors, ThemePreference};

const MODELS: &[(&str, &str)] = &[
    ("SenseNova 6.8 Flash Lite", "sensenova-6.8-flash-lite"),
    ("GLM 5.2", "glm-5.2"),
    ("Kimi K3", "kimi-k3"),
    ("DeepSeek V4 Flash", "deepseek-v4-flash"),
    ("DeepSeek V4.1 Flash", "deepseek-v4.1-flash"),
    ("Claude 3.7 Sonnet", "claude-3-7-sonnet-20250219"),
    ("GPT-4o", "gpt-4o"),
];
#[derive(Clone)]
pub struct ConversationItem {
    pub id: String,
    pub title: String,
    pub is_active: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageFeedback {
    Like,
    Dislike,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArtifactPaneState {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub content: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ArtifactViewMode {
    #[default]
    Preview,
    Code,
}

pub struct PiDesktopApp {
    pub artifact_pane: Option<ArtifactPaneState>,
    pub artifact_visible: bool,
    pub artifact_view_mode: ArtifactViewMode,
    workspace_dir: PathBuf,
    session: Session,
    session_history: HashMap<String, Session>,
    task_states: HashMap<String, task_state::TaskState>,
    review: review::ReviewState,
    review_visible: bool,
    workflow_pane: Option<workflow_pane::WorkflowPane>,
    terminal_visible: bool,
    terminal_input: Entity<InputState>,
    review_search: Entity<InputState>,
    review_files_scroll: ScrollHandle,
    review_diff_scroll: ScrollHandle,
    terminal_output_scroll: ScrollHandle,
    conversations: Vec<ConversationItem>,
    drafts: HashMap<String, String>,
    expanded_tools: HashSet<String>,
    hovered_message: Option<String>,
    hovered_reasoning: Option<String>,
    hovered_tool: Option<String>,
    copied_message: Option<String>,
    message_feedback: HashMap<String, MessageFeedback>,
    agent: Arc<PiAgent>,
    workflow_controls: pi_agent_core::workflow_control::WorkflowControls,
    input_state: Entity<TextareaState>,
    settings_general_language: Entity<SelectState<Vec<SharedString>>>,
    settings_appearance_theme: Entity<SelectState<Vec<SharedString>>>,
    settings_footer_language: Entity<SelectState<Vec<SharedString>>>,
    settings_footer_theme: Entity<SelectState<Vec<SharedString>>>,
    settings_base_input: Entity<InputState>,
    settings_key_input: Entity<InputState>,
    settings_model_input: Entity<InputState>,
    settings_font_input: Entity<InputState>,
    search_input: Entity<InputState>,
    skill_search_input: Entity<InputState>,
    selected_nav: &'static str,
    show_settings: bool,
    settings_section: settings::Section,
    sidebar_visible: bool,
    search_open: bool,
    show_onboard_banner: bool,
    show_quick_actions: bool,
    language: Language,
    theme_preference: ThemePreference,
    preferences: desktop_state::Preferences,
    api_base: String,
    api_key: String,
    model_identifier: String,
    settings_saved: bool,
    settings_error: String,
    status_message: String,
    tokens_used: usize,
    is_executing: bool,
    run_error: Option<String>,
    run_stopped: bool,
    run_generation: u64,
    running: Option<tokio::task::AbortHandle>,
    runtime: tokio::runtime::Runtime,
    skill_registry: SkillRegistry,
    skill_category: Option<SkillCategory>,
    store: desktop_state::Store,
    storage_error: String,
    storage_read_failed: bool,
    save_task: Option<Task<()>>,
    root_focus: FocusHandle,
    transcript_focus: FocusHandle,
    scroll: ScrollHandle,
    scroll_pinned: bool,
    scroll_memory: HashMap<String, (ScrollHandle, bool)>,
    tool_output_scroll: std::cell::RefCell<HashMap<String, ScrollHandle>>,
    reasoning_scroll_follow:
        std::cell::RefCell<HashMap<String, std::rc::Rc<std::cell::Cell<bool>>>>,
    workflow_scroll: std::cell::RefCell<HashMap<String, workflow_timeline::Viewport>>,
    workflow_card_focus: std::cell::RefCell<HashMap<String, FocusHandle>>,
    message_action_focus: std::cell::RefCell<HashMap<String, FocusHandle>>,
    reasoning_summary_rolls: std::cell::RefCell<HashMap<String, workbench::ReasoningSummaryRoll>>,
    /// Deadline of the detached hold timer. A later render must not schedule a second wake for the same slot.
    reasoning_roll_wake: std::cell::RefCell<Option<std::time::Instant>>,
    _subscriptions: Vec<Subscription>,
    automations: Vec<desktop_state::Automation>,
    automation_name: Entity<InputState>,
    automation_prompt: Entity<TextareaState>,
    automation_minutes: Entity<InputState>,
    automation_form: bool,
    automation_editing: Option<String>,
    automation_error: String,
    timer: Option<Task<()>>,
    /// Wakes the empty draft when the local greeting hour changes.
    greeting_refresh: std::cell::RefCell<Option<Task<()>>>,
    copy_reset: std::cell::RefCell<Option<Task<()>>>,
}

fn mask_api_key(key: &str, language: Language) -> String {
    if key.trim().is_empty() {
        match language {
            Language::Chinese => "未配置",
            Language::English => "Not configured",
        }
        .into()
    } else {
        "••••••••".into()
    }
}
impl PiDesktopApp {
    fn ui_text(&self, base: f32) -> Pixels {
        px(desktop_state::ui_text_px(
            base,
            self.preferences.ui_font_size_px,
        ))
    }

    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let workspace_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let store = desktop_state::Store::for_workspace(&workspace_dir);
        let (saved, storage_error) = match store.load() {
            Ok(saved) => (saved, String::new()),
            Err(error) => (
                desktop_state::SavedDesktop::default(),
                format!("History could not be read: {error}"),
            ),
        };
        let mut saved = saved;
        workflow_view::migrate_disclosures(&mut saved);
        saved.skip_missed_automations(desktop_state::now());
        for state in saved.task_states.values_mut() {
            state.queue_paused = true;
            for workflow in &mut state.workflows {
                workflow.interrupt();
            }
        }
        for session in saved.sessions.values_mut() {
            execution::reconcile_interrupted_tools(&mut session.messages);
        }
        let preferences = saved.preferences.clone();
        let language = preferences.language;
        let api_base = std::env::var("OPENAI_BASE_URL")
            .ok()
            .or(preferences.api_base.clone())
            .unwrap_or_else(|| "https://token.sensenova.cn/v1".into());
        let api_key = std::env::var("SENSENOVA_API_KEY")
            .or_else(|_| std::env::var("OPENAI_API_KEY"))
            .unwrap_or_default();
        let model_identifier = preferences.model.clone();
        let agent = Arc::new(PiAgent::new(workspace_dir.clone(), ModelConfig::default()));
        let input_state = cx.new(|cx| {
            TextareaState::new(window, cx)
                .auto_grow(2, 8)
                .submit_on_enter(true)
                .placeholder(language.text().input_placeholder)
        });
        let terminal_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("输入命令 / Enter command"));
        let review_search =
            cx.new(|cx| InputState::new(window, cx).placeholder("搜索文件 / Search files"));
        let settings_general_language = cx.new(|cx| {
            SelectState::new(
                vec![SharedString::from("中文"), SharedString::from("English")],
                Some(IndexPath::new(if language == Language::Chinese {
                    0
                } else {
                    1
                })),
                window,
                cx,
            )
        });
        let settings_appearance_theme = cx.new(|cx| {
            SelectState::new(
                settings::theme_labels(language),
                Some(IndexPath::new(settings::theme_index(preferences.theme))),
                window,
                cx,
            )
        });
        let settings_footer_language = cx.new(|cx| {
            SelectState::new(
                vec![SharedString::from("中文"), SharedString::from("English")],
                Some(IndexPath::new(if language == Language::Chinese {
                    0
                } else {
                    1
                })),
                window,
                cx,
            )
        });
        let settings_footer_theme = cx.new(|cx| {
            SelectState::new(
                settings::theme_labels(language),
                Some(IndexPath::new(settings::theme_index(preferences.theme))),
                window,
                cx,
            )
        });
        let settings_base_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("https://api.example.com/v1"));
        let settings_key_input = cx.new(|cx| {
            InputState::new(window, cx)
                .masked(true)
                .placeholder(if language == Language::Chinese {
                    "输入 API Key"
                } else {
                    "Enter API key"
                })
        });
        let settings_model_input = cx.new(|cx| InputState::new(window, cx));
        let settings_font_input = cx.new(|cx| InputState::new(window, cx));
        settings_font_input.update(cx, |input, cx| {
            input.set_value(
                desktop_state::commit_ui_font_size(&preferences.ui_font_size_px.to_string(), 14)
                    .to_string(),
                window,
                cx,
            )
        });
        settings_base_input.update(cx, |input, cx| {
            input.set_value(api_base.clone(), window, cx)
        });
        settings_model_input.update(cx, |input, cx| {
            input.set_value(model_identifier.clone(), window, cx)
        });
        let search_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder(if language == Language::Chinese {
                "搜索任务..."
            } else {
                "Search tasks..."
            })
        });
        let skill_search_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("搜索技能 / Search skills"));
        let mut session_history = saved.sessions;
        let session = session_history
            .get(&saved.active_id)
            .cloned()
            .unwrap_or_else(|| {
                Session::new(desktop_state::new_id(), language.text().new_conversation)
            });
        session_history.insert(session.id.clone(), session.clone());
        let mut restored: Vec<_> = session_history.values().cloned().collect();
        restored.sort_by_key(|session| std::cmp::Reverse(session.updated_at));
        let conversations = restored
            .into_iter()
            .map(|entry| ConversationItem {
                is_active: entry.id == session.id,
                id: entry.id,
                title: entry.title,
            })
            .collect();
        input_state.update(cx, |input, cx| {
            input.set_value(
                saved.drafts.get(&session.id).cloned().unwrap_or_default(),
                window,
                cx,
            );
            input.focus(window, cx);
        });
        let mut subscriptions =
            vec![
                cx.subscribe_in(&input_state, window, |this, _, event, window, cx| {
                    match event {
                        InputEvent::PressEnter { shift: false, .. } => {
                            this.trigger_send(window, cx)
                        }
                        InputEvent::Change => {
                            this.drafts.insert(
                                this.session.id.clone(),
                                this.input_state.read(cx).value().to_string(),
                            );
                            this.schedule_save(cx);
                        }
                        _ => {}
                    }
                    cx.notify();
                }),
            ];
        subscriptions.push(cx.subscribe_in(
            &settings_general_language,
            window,
            |this, _, event: &SelectEvent<Vec<SharedString>>, window, cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    this.change_language(
                        if value.as_ref() == "中文" {
                            Language::Chinese
                        } else {
                            Language::English
                        },
                        window,
                        cx,
                    );
                }
            },
        ));
        subscriptions.push(cx.subscribe_in(
            &settings_footer_language,
            window,
            |this, _, event: &SelectEvent<Vec<SharedString>>, window, cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    this.change_language(
                        if value.as_ref() == "中文" {
                            Language::Chinese
                        } else {
                            Language::English
                        },
                        window,
                        cx,
                    );
                }
            },
        ));
        subscriptions.push(cx.subscribe_in(
            &settings_appearance_theme,
            window,
            |this, _, event: &SelectEvent<Vec<SharedString>>, window, cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    this.change_theme(value, window, cx);
                }
            },
        ));
        subscriptions.push(cx.subscribe_in(
            &settings_footer_theme,
            window,
            |this, _, event: &SelectEvent<Vec<SharedString>>, window, cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    this.change_theme(value, window, cx);
                }
            },
        ));
        subscriptions.push(cx.subscribe_in(
            &search_input,
            window,
            |this, _, event: &InputEvent, window, cx| {
                if let InputEvent::PressEnter { .. } = event {
                    if this.search_open {
                        let query = this.search_input.read(cx).value().trim().to_lowercase();
                        let target_id = this
                            .conversations
                            .iter()
                            .find(|entry| {
                                if query.is_empty() {
                                    true
                                } else {
                                    entry.title.to_lowercase().contains(&query)
                                }
                            })
                            .map(|entry| entry.id.clone());
                        if let Some(id) = target_id {
                            this.activate_conversation(&id, window, cx);
                            this.search_open = false;
                            this.focus_composer(window, cx);
                            cx.notify();
                        }
                    }
                } else {
                    cx.notify();
                }
            },
        ));
        subscriptions.push(
            cx.subscribe(&skill_search_input, |_, _, _event: &InputEvent, cx| {
                cx.notify()
            }),
        );
        subscriptions.push(cx.subscribe_in(
            &settings_font_input,
            window,
            |this, input, event: &InputEvent, window, cx| {
                if !matches!(event, InputEvent::Blur | InputEvent::PressEnter { .. }) {
                    return;
                }
                let next = desktop_state::commit_ui_font_size(
                    &input.read(cx).value(),
                    this.preferences.ui_font_size_px,
                );
                this.preferences.ui_font_size_px = next;
                this.settings_saved = false;
                input.update(cx, |field, cx| {
                    field.set_value(next.to_string(), window, cx)
                });
                cx.notify();
            },
        ));
        for field in [
            &settings_base_input,
            &settings_key_input,
            &settings_model_input,
        ] {
            subscriptions.push(cx.subscribe(field, |this, _, event, cx| {
                if matches!(event, InputEvent::Change) {
                    this.settings_saved = false;
                }
                cx.notify();
            }));
        }
        subscriptions.push(cx.on_app_quit(|this, cx| {
            this.drafts.insert(
                this.session.id.clone(),
                this.input_state.read(cx).value().to_string(),
            );
            this.persist_active_session();
            async {}
        }));
        subscriptions.push(cx.on_release(|this, cx| {
            this.drafts.insert(
                this.session.id.clone(),
                this.input_state.read(cx).value().to_string(),
            );
            this.persist_active_session();
        }));
        subscriptions.push(cx.on_focus_lost(window, |this, window, cx| {
            this.root_focus.focus(window, cx);
        }));
        subscriptions.push(cx.subscribe_in(
            &terminal_input,
            window,
            |this, _, event: &InputEvent, window, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    this.run_terminal(window, cx);
                }
            },
        ));
        subscriptions
            .push(cx.subscribe(&review_search, |_, _, _event: &InputEvent, cx| cx.notify()));
        let mut app = Self {
            workspace_dir,
            session,
            session_history,
            task_states: saved.task_states,
            review: review::ReviewState::default(),
            review_visible: false,
            workflow_pane: None,
            terminal_visible: false,
            terminal_input,
            review_search,
            review_files_scroll: ScrollHandle::new(),
            review_diff_scroll: ScrollHandle::new(),
            terminal_output_scroll: ScrollHandle::new(),
            conversations,
            drafts: saved.drafts,
            expanded_tools: saved.expanded_tools,
            hovered_message: None,
            hovered_reasoning: None,
            hovered_tool: None,
            copied_message: None,
            message_feedback: HashMap::new(),
            artifact_pane: None,
            artifact_visible: false,
            artifact_view_mode: ArtifactViewMode::Preview,
            agent,
            workflow_controls: Default::default(),
            input_state,
            settings_general_language,
            settings_appearance_theme,
            settings_footer_language,
            settings_footer_theme,
            settings_base_input,
            settings_key_input,
            settings_model_input,
            settings_font_input,
            search_input,
            skill_search_input,
            selected_nav: "chat",
            show_settings: false,
            settings_section: settings::section_from_key(preferences.settings_section.as_deref()),
            sidebar_visible: true,
            search_open: false,
            show_onboard_banner: true,
            show_quick_actions: true,
            language,
            theme_preference: preferences.theme,
            preferences,
            api_base,
            api_key,
            model_identifier,
            settings_saved: false,
            settings_error: String::new(),
            status_message: String::new(),
            tokens_used: 0,
            is_executing: false,
            run_error: None,
            run_stopped: false,
            run_generation: 0,
            running: None,
            runtime: tokio::runtime::Runtime::new().expect("desktop runtime"),
            skill_registry: SkillRegistry::with_builtin_skills(),
            skill_category: None,
            store,
            storage_read_failed: !storage_error.is_empty(),
            storage_error,
            save_task: None,
            root_focus: cx.focus_handle(),
            transcript_focus: cx.focus_handle().tab_stop(true),
            scroll: ScrollHandle::new(),
            scroll_pinned: true,
            scroll_memory: HashMap::new(),
            tool_output_scroll: std::cell::RefCell::new(HashMap::new()),
            reasoning_scroll_follow: std::cell::RefCell::new(HashMap::new()),
            workflow_scroll: std::cell::RefCell::new(HashMap::new()),
            workflow_card_focus: std::cell::RefCell::new(HashMap::new()),
            message_action_focus: std::cell::RefCell::new(HashMap::new()),
            reasoning_summary_rolls: std::cell::RefCell::new(HashMap::new()),
            reasoning_roll_wake: std::cell::RefCell::new(None),
            _subscriptions: subscriptions,
            automations: saved.automations,
            automation_name: cx.new(|cx| InputState::new(window, cx)),
            automation_prompt: cx.new(|cx| {
                TextareaState::new(window, cx)
                    .auto_grow(5, 7)
                    .submit_on_enter(false)
            }),
            automation_minutes: cx.new(|cx| InputState::new(window, cx).placeholder("60")),
            automation_form: false,
            automation_editing: None,
            automation_error: String::new(),
            timer: None,
            greeting_refresh: std::cell::RefCell::new(None),
            copy_reset: std::cell::RefCell::new(None),
        };
        app.restore_task_state();
        app.start_timer(window, cx);
        window.activate_window();
        cx.activate(true);
        app
    }
    pub fn t(&self) -> &'static I18nText {
        self.language.text()
    }
    fn tr(&self, zh: &'static str, en: &'static str) -> &'static str {
        if self.language == Language::Chinese {
            zh
        } else {
            en
        }
    }
    pub fn current_model(&self) -> String {
        MODELS
            .iter()
            .find(|(_, id)| *id == self.model_identifier)
            .map(|(name, _)| name.to_string())
            .unwrap_or_else(|| self.model_identifier.clone())
    }
    pub fn toggle_tool_expansion(&mut self, id: &str) {
        if !self.expanded_tools.remove(id) {
            self.expanded_tools.insert(id.to_string());
        }
        self.persist_active_session();
    }
    pub fn trigger_quick_task(&mut self, kind: &str, cx: &mut Context<Self>) {
        let command = match kind {
            "test" => "cargo test -p pi-agent-core",
            "clippy" => "cargo clippy -p pi-agent-core",
            "build" => "cargo check -p pi-desktop",
            _ => return,
        };
        self.submit_prompt(format!("$ {command}"), cx);
    }
}
fn main() {
    if let Ok(true) = installer::handle_cli_args() {
        return;
    }
    tracing_subscriber::fmt::init();
    gpui_kit::application()
        .with_assets(gpui_kit::assets::AllAssets)
        .run(|cx| {
            gpui_kit::init(cx);
            interactions::bind_keys(cx);
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1200.), px(800.)),
                    cx,
                ))),
                window_min_size: Some(size(px(480.), px(640.))),
                titlebar: Some(TitlebarOptions {
                    title: Some("ZCode".into()),
                    appears_transparent: true,
                    traffic_light_position: Some(point(px(22.), px(23.))),
                }),
                ..Default::default()
            };
            gpui_kit::open_window(options, cx, |window, cx| {
                cx.new(|cx| PiDesktopApp::new(window, cx))
            })
            .expect("open workbench");
        });
}
