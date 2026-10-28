use crate::{Language, ThemePreference};
use pi_agent_core::Session;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub api_base: Option<String>,
    pub model: String,
    pub language: Language,
    pub theme: ThemePreference,
    pub reduced_motion: bool,
    pub settings_section: Option<String>,
    /// Interface text size in px. Icons, spacing, and fixed workflow geometry stay unscaled.
    #[serde(default = "default_ui_font_size")]
    pub ui_font_size_px: u8,
}
fn default_ui_font_size() -> u8 {
    14
}
pub fn commit_ui_font_size(draft: &str, current: u8) -> u8 {
    let current = current.clamp(12, 20);
    let Some(parsed) = draft.trim().parse::<f32>().ok() else {
        return current;
    };
    if !parsed.is_finite() {
        return current;
    }
    parsed.round().clamp(12., 20.) as u8
}
pub fn ui_text_px(base: f32, font_size: u8) -> f32 {
    base * f32::from(font_size.clamp(12, 20)) / 14.
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            api_base: None,
            model: crate::MODELS[0].1.into(),
            language: Language::Chinese,
            theme: ThemePreference::default(),
            reduced_motion: false,
            settings_section: None,
            ui_font_size_px: default_ui_font_size(),
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Automation {
    pub id: String,
    pub name: String,
    pub prompt: String,
    pub minutes: u32,
    pub enabled: bool,
    pub next_run: u64,
}
impl Automation {
    pub fn validate(name: &str, prompt: &str, minutes: &str) -> Result<u32, &'static str> {
        if name.trim().is_empty() || prompt.trim().is_empty() {
            return Err("名称和任务不能为空 / Name and task are required");
        }
        let minutes = minutes
            .parse::<u32>()
            .map_err(|_| "请输入分钟数 / Enter minutes")?;
        if !(1..=10080).contains(&minutes) {
            return Err("间隔须为 1–10080 分钟 / Interval: 1–10080 minutes");
        }
        Ok(minutes)
    }
}
#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SavedDesktop {
    pub preferences: Preferences,
    pub sessions: HashMap<String, Session>,
    pub task_states: HashMap<String, crate::task_state::TaskState>,
    pub active_id: String,
    pub drafts: HashMap<String, String>,
    pub expanded_tools: HashSet<String>,
    pub automations: Vec<Automation>,
}
impl SavedDesktop {
    pub fn skip_missed_automations(&mut self, now: u64) {
        for job in &mut self.automations {
            if job.enabled && job.next_run <= now {
                job.next_run = now + u64::from(job.minutes.max(1)) * 60;
            }
        }
    }
}
pub struct Store {
    path: PathBuf,
}
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
pub fn new_id() -> String {
    format!(
        "task-{:x}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}
impl Store {
    pub fn for_workspace(workspace: &Path) -> Self {
        let root = std::env::var_os("VOLT_STATE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(
                    std::env::var_os("HOME")
                        .or_else(|| std::env::var_os("USERPROFILE"))
                        .unwrap_or_default(),
                )
                .join(".local/share/volt-desktop")
            });
        // Fixed FNV-1a isolates workspaces without storing chat history inside Git repositories.
        let hash = workspace
            .to_string_lossy()
            .bytes()
            .fold(0xcbf29ce484222325u64, |hash, byte| {
                (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
            });
        Self {
            path: root.join(format!("{hash:016x}/desktop.json")),
        }
    }
    pub fn load(&self) -> anyhow::Result<SavedDesktop> {
        match fs::read(&self.path) {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(SavedDesktop::default())
            }
            Err(error) => Err(error.into()),
        }
    }
    pub fn save(&self, state: &SavedDesktop) -> anyhow::Result<()> {
        let parent = self.path.parent().expect("state parent");
        fs::create_dir_all(parent)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
        }
        let temp = parent.join(format!("{}.tmp", new_id()));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temp)?;
        let written = (|| -> anyhow::Result<()> {
            file.write_all(&serde_json::to_vec_pretty(state)?)?;
            file.sync_all()?;
            drop(file);
            fs::rename(&temp, &self.path)?;
            Ok(())
        })();
        if written.is_err() {
            let _ = fs::remove_file(temp);
        }
        written
    }
}
pub fn validate_endpoint(endpoint: &str) -> bool {
    reqwest::Url::parse(endpoint).is_ok_and(|url| {
        matches!(url.scheme(), "https" | "http")
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none()
    })
}
#[cfg(test)]
mod tests {
    #[test]
    fn theme_defaults_to_light_and_preserves_saved_choices() {
        use super::{Preferences, SavedDesktop, ThemePreference};
        assert_eq!(Preferences::default().theme, ThemePreference::Light);
        assert_eq!(Preferences::default().ui_font_size_px, 14);
        for json in ["{}", r#"{"preferences":{}}"#] {
            let saved: SavedDesktop = serde_json::from_str(json).unwrap();
            assert_eq!(saved.preferences.theme, ThemePreference::Light);
            assert_eq!(saved.preferences.ui_font_size_px, 14);
        }
        assert_eq!(super::commit_ui_font_size("", 16), 16);
        assert_eq!(super::commit_ui_font_size("11", 14), 12);
        assert_eq!(super::commit_ui_font_size("20.4", 14), 20);
        assert_eq!(super::commit_ui_font_size("99", 14), 20);
        assert_eq!(super::ui_text_px(14., 14), 14.);
        assert_eq!(super::ui_text_px(14., 20), 20.);
        for theme in [
            ThemePreference::Light,
            ThemePreference::Dark,
            ThemePreference::Auto,
        ] {
            let mut saved = SavedDesktop::default();
            saved.preferences.theme = theme;
            let restored: SavedDesktop =
                serde_json::from_str(&serde_json::to_string(&saved).unwrap()).unwrap();
            assert_eq!(restored.preferences.theme, theme);
        }
    }
    use super::*;
    #[test]
    fn round_trip_history_preferences_drafts() {
        let root = std::env::temp_dir().join(new_id());
        let store = Store {
            path: root.join("desktop.json"),
        };
        let mut state = SavedDesktop::default();
        let mut session = Session::new("a", "中文会话");
        session
            .messages
            .push(pi_agent_core::Message::user("draft survives"));
        state.sessions.insert("a".into(), session);
        state.active_id = "a".into();
        state.drafts.insert("a".into(), "未发送".into());
        store.save(&state).unwrap();
        store.save(&state).unwrap();
        let restored = store.load().unwrap();
        assert_eq!(restored.sessions["a"].messages[0].content, "draft survives");
        assert_eq!(restored.drafts["a"], "未发送");
        assert!(!fs::read_to_string(&store.path).unwrap().contains("api_key"));
        fs::write(&store.path, "broken").unwrap();
        assert!(store.load().is_err());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn endpoints_reject_credentials_and_non_http() {
        for value in [
            "file:///tmp",
            "https://user:secret@example.com",
            "https://a?key=x",
            "broken",
        ] {
            assert!(!validate_endpoint(value));
        }
        assert!(validate_endpoint("http://localhost:11434/v1"));
    }
    #[test]
    fn reopening_skips_missed_runs_and_preserves_future_runs() {
        let mut state = SavedDesktop::default();
        for (id, next_run) in [("past", 50), ("future", 500)] {
            state.automations.push(Automation {
                id: id.into(),
                name: id.into(),
                prompt: "test".into(),
                minutes: 1,
                enabled: true,
                next_run,
            });
        }
        state.skip_missed_automations(100);
        assert_eq!(state.automations[0].next_run, 160);
        assert_eq!(state.automations[1].next_run, 500);
    }

    #[test]
    fn automation_validation() {
        assert!(Automation::validate("", "hello", "60").is_err());
        assert!(Automation::validate("Test", "hello", "0").is_err());
        assert_eq!(Automation::validate("Test", "hello", "60").unwrap(), 60);
    }
}
