use super::*;
use desktop_state::{Automation, SavedDesktop};
use std::time::Duration;
extern crate gpui_kit as gpui;
gpui_kit::actions!(
    volt,
    [
        NewTask,
        ToggleSidebar,
        ToggleSearch,
        Settings,
        Dismiss,
        FocusComposer,
        ToggleReview,
        ToggleTerminal
    ]
);
pub fn bind_keys(cx: &mut App) {
    let primary = if cfg!(target_os = "macos") {
        "cmd"
    } else {
        "ctrl"
    };
    cx.bind_keys([
        KeyBinding::new(&format!("{primary}-n"), NewTask, Some("ZCode")),
        KeyBinding::new(&format!("{primary}-b"), ToggleSidebar, Some("ZCode")),
        KeyBinding::new(&format!("{primary}-k"), ToggleSearch, Some("ZCode")),
        KeyBinding::new(&format!("{primary}-,"), Settings, Some("ZCode")),
        KeyBinding::new(&format!("{primary}-l"), FocusComposer, Some("ZCode")),
        KeyBinding::new("escape", Dismiss, Some("ZCode")),
        KeyBinding::new(&format!("{primary}-shift-d"), ToggleReview, Some("ZCode")),
        KeyBinding::new(&format!("{primary}-j"), ToggleTerminal, Some("ZCode")),
    ]);
}
impl PiDesktopApp {
    pub fn persist_active_session(&mut self) {
        if self.storage_read_failed {
            return;
        } // Never overwrite an unreadable history with a default document.
        self.capture_task_state();
        self.session_history
            .insert(self.session.id.clone(), self.session.clone());
        let saved = SavedDesktop {
            sessions: self.session_history.clone(),
            task_states: self.task_states.clone(),
            active_id: self.session.id.clone(),
            drafts: self.drafts.clone(),
            expanded_tools: self.expanded_tools.clone(),
            preferences: self.preferences.clone(),
            automations: self.automations.clone(),
        };
        self.storage_error = match self.store.save(&saved) {
            Ok(()) => String::new(),
            Err(error) => format!("保存失败 / Save failed: {error}"),
        };
    }
    pub fn schedule_save(&mut self, cx: &mut Context<Self>) {
        self.save_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(400))
                .await;
            let _ = this.update(cx, |this, cx| {
                this.persist_active_session();
                cx.notify();
            });
        }));
    }
    pub fn new_conversation(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_executing {
            return;
        }
        self.persist_active_session();
        self.remember_scroll();
        self.workflow_pane = None;
        self.session = Session::new(desktop_state::new_id(), self.t().new_conversation);
        self.scroll = ScrollHandle::new();
        self.scroll_pinned = true;
        for entry in &mut self.conversations {
            entry.is_active = false;
        }
        self.conversations.insert(
            0,
            ConversationItem {
                id: self.session.id.clone(),
                title: self.session.title.clone(),
                is_active: true,
            },
        );
        self.restore_task_state();
        self.input_state.update(cx, |input, cx| {
            input.clean(window, cx);
            input.focus(window, cx);
        });
        self.show_settings = false;
        self.selected_nav = "chat";
        self.persist_active_session();
        cx.notify();
    }
    pub fn activate_conversation(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_executing {
            return;
        }
        let Some(session) = self.session_history.get(id).cloned() else {
            return;
        };
        self.persist_active_session();
        self.remember_scroll();
        self.workflow_pane = None;
        self.session = session;
        (self.scroll, self.scroll_pinned) = self
            .scroll_memory
            .remove(id)
            .unwrap_or_else(|| (ScrollHandle::new(), true));
        for entry in &mut self.conversations {
            entry.is_active = entry.id == id;
        }
        let draft = self.drafts.get(id).cloned().unwrap_or_default();
        self.input_state.update(cx, |input, cx| {
            input.set_value(draft, window, cx);
            input.focus(window, cx);
        });
        self.show_settings = false;
        self.selected_nav = "chat";
        self.restore_task_state();
        self.persist_active_session();
        cx.notify();
    }
    pub fn focus_composer(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.show_settings = false;
        self.selected_nav = "chat";
        if panels::review_uses_single_column(window.viewport_size().width) {
            self.workflow_pane = None;
        }
        self.input_state
            .update(cx, |input, cx| input.focus(window, cx));
        cx.notify();
    }
    fn remember_scroll(&mut self) {
        // Renderer-local only: never serialize reading position into chat data.
        if self.scroll_memory.len() >= 200 {
            if let Some(key) = self.scroll_memory.keys().next().cloned() {
                self.scroll_memory.remove(&key);
            }
        }
        self.scroll_memory.insert(
            self.session.id.clone(),
            (self.scroll.clone(), self.scroll_pinned),
        );
    }
    pub fn save_preferences(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let editing_provider = self.settings_section == settings::Section::Providers;
        let base = self.settings_base_input.read(cx).value().trim().to_string();
        let model = self
            .settings_model_input
            .read(cx)
            .value()
            .trim()
            .to_string();
        self.settings_saved = false;
        if editing_provider && (!desktop_state::validate_endpoint(&base) || model.is_empty()) {
            self.settings_error = self
                .tr(
                    "请输入有效 HTTP(S) 地址和模型名称",
                    "Enter a valid HTTP(S) URL and model name",
                )
                .into();
            cx.notify();
            return;
        }
        if editing_provider {
            self.api_base = base.clone();
            self.model_identifier = model.clone();
            let key = self.settings_key_input.read(cx).unmask_value().to_string();
            if !key.trim().is_empty() {
                self.api_key = key;
            }
            self.settings_key_input
                .update(cx, |input, cx| input.clean(window, cx));
        }
        self.preferences = desktop_state::Preferences {
            api_base: Some(self.api_base.clone()),
            model: self.model_identifier.clone(),
            language: self.language,
            theme: self.theme_preference,
            reduced_motion: self.preferences.reduced_motion,
            settings_section: Some(self.settings_section.key().into()),
            ui_font_size_px: self.preferences.ui_font_size_px,
        };
        self.persist_active_session();
        self.settings_error = self.storage_error.clone();
        self.settings_saved = !self.storage_read_failed && self.storage_error.is_empty();
        cx.notify();
    }
    pub fn change_language(
        &mut self,
        language: Language,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.language = language;
        let theme = self.theme_preference;
        self.settings_theme.update(cx, |state, cx| {
            state.set_items(settings::theme_labels(language), window, cx);
            state.set_selected_index(
                Some(IndexPath::new(settings::theme_index(theme))),
                window,
                cx,
            );
        });
        self.settings_saved = false;
        let placeholder = self.t().input_placeholder;
        self.input_state.update(cx, |input, cx| {
            input.set_placeholder(placeholder, window, cx)
        });
        self.search_input.update(cx, |input, cx| {
            input.set_placeholder(
                if language == Language::Chinese {
                    "搜索任务..."
                } else {
                    "Search tasks..."
                },
                window,
                cx,
            );
        });
        cx.notify();
    }
    pub fn edit_automation(
        &mut self,
        id: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let job = id
            .as_ref()
            .and_then(|id| self.automations.iter().find(|job| &job.id == id))
            .cloned();
        self.automation_name.update(cx, |input, cx| {
            input.set_value(
                job.as_ref().map(|j| j.name.clone()).unwrap_or_default(),
                window,
                cx,
            )
        });
        self.automation_prompt.update(cx, |input, cx| {
            input.set_value(
                job.as_ref().map(|j| j.prompt.clone()).unwrap_or_default(),
                window,
                cx,
            )
        });
        self.automation_minutes.update(cx, |input, cx| {
            input.set_value(
                job.as_ref().map(|j| j.minutes).unwrap_or(60).to_string(),
                window,
                cx,
            )
        });
        self.automation_editing = id;
        self.automation_form = true;
        self.automation_error.clear();
        cx.notify();
    }
    pub fn save_automation(&mut self, cx: &mut Context<Self>) {
        let name = self.automation_name.read(cx).value().to_string();
        let prompt = self.automation_prompt.read(cx).value().to_string();
        let minutes =
            match Automation::validate(&name, &prompt, &self.automation_minutes.read(cx).value()) {
                Ok(minutes) => minutes,
                Err(error) => {
                    self.automation_error = error.into();
                    cx.notify();
                    return;
                }
            };
        let id = self
            .automation_editing
            .clone()
            .unwrap_or_else(desktop_state::new_id);
        let job = Automation {
            id: id.clone(),
            name,
            prompt,
            minutes,
            enabled: false,
            next_run: desktop_state::now() + u64::from(minutes) * 60,
        };
        if let Some(index) = self.automations.iter().position(|j| j.id == id) {
            self.automations[index] = job;
        } else {
            self.automations.push(job);
        }
        self.persist_active_session();
        if self.storage_read_failed || !self.storage_error.is_empty() {
            self.automation_error = self.storage_error.clone();
        } else {
            self.automation_form = false;
        }
        cx.notify();
    }
    pub fn start_timer(&mut self, window: &Window, cx: &mut Context<Self>) {
        self.timer = Some(cx.spawn_in(window, async move |this, cx| loop {
            cx.background_executor().timer(Duration::from_secs(1)).await;
            if this
                .update_in(cx, |this, window, cx| {
                    if this.is_executing || this.storage_read_failed {
                        return;
                    }
                    let now = desktop_state::now();
                    if let Some(index) = this
                        .automations
                        .iter()
                        .position(|job| job.enabled && job.next_run <= now)
                    {
                        let prompt = this.automations[index].prompt.clone();
                        this.automations[index].next_run =
                            now + u64::from(this.automations[index].minutes) * 60;
                        // A scheduled run gets its own task; the unsent composer draft stays with its original task.
                        this.persist_active_session();
                        this.session = Session::new(
                            desktop_state::new_id(),
                            this.automations[index].name.clone(),
                        );
                        for entry in &mut this.conversations {
                            entry.is_active = false;
                        }
                        this.conversations.insert(
                            0,
                            ConversationItem {
                                id: this.session.id.clone(),
                                title: this.session.title.clone(),
                                is_active: true,
                            },
                        );
                        this.restore_task_state();
                        this.input_state
                            .update(cx, |input, cx| input.clean(window, cx));
                        this.submit_prompt(prompt, cx);
                    }
                })
                .is_err()
            {
                break;
            }
        }));
    }
}
