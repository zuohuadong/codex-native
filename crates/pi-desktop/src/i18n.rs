#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    Chinese,
    English,
}

pub struct I18nText {
    // Navigation & Sidebar
    pub app_title: &'static str,
    pub new_conversation: &'static str,
    pub conversation_history: &'static str,
    pub automations: &'static str,
    pub skills_tools: &'static str,
    pub projects_header: &'static str,
    pub conversations_header: &'static str,
    pub settings: &'static str,
    pub agent_online: &'static str,

    // Header & Workspace
    pub branch_label: &'static str,
    pub path_label: &'static str,
    pub status_ready: &'static str,
    pub status_running: &'static str,

    // Chat
    pub you: &'static str,
    pub agent_name: &'static str,
    pub tool_result: &'static str,
    pub tool_duration: &'static str,
    pub welcome_msg: &'static str,

    // Input Bar
    pub context_tokens: &'static str,
    pub input_placeholder: &'static str,
    pub send_btn: &'static str,

    // Settings Panel
    pub settings_title: &'static str,
    pub settings_subtitle: &'static str,
    pub back_to_chat: &'static str,
    pub settings_model_title: &'static str,
    pub api_base_label: &'static str,
    pub api_key_label: &'static str,
    pub model_name_label: &'static str,
    pub language_label: &'static str,
    pub mode_label: &'static str,
    pub mode_auto: &'static str,
    pub mode_manual: &'static str,
    pub save_btn: &'static str,
    pub save_success: &'static str,

    // Quick Actions
    pub quick_test: &'static str,
    pub quick_clippy: &'static str,
    pub quick_build: &'static str,
    pub nav_automations_desc: &'static str,
    pub nav_skills_desc: &'static str,
}

pub const ZH_CN: I18nText = I18nText {
    app_title: "⚡ Volt 智能编程桌面",
    new_conversation: "+ 新建会话",
    conversation_history: "💬 对话历史",
    automations: "⏱️ 自动化工作流",
    skills_tools: "🧩 技能与工具",
    projects_header: "项目空间",
    conversations_header: "近期任务",
    settings: "设置",
    agent_online: "● 运行中 (v0.1.0)",

    branch_label: "分支",
    path_label: "路径",
    status_ready: "就绪 (Volt 核心在线)",
    status_running: "智能体执行中...",

    you: "你",
    agent_name: "Volt 原生智能体",
    tool_result: "⚡ 工具执行结果",
    tool_duration: "耗时",
    welcome_msg: "你好！我是 Volt，高性能原生自主编程智能体。\n⚡ 由 Volt 极速引擎 (120 FPS 硬件加速) 驱动。\n🛠️ 零 Electron、零 Node/V8 运行时开销，毫秒级冷启动。\n\n今天有什么代码任务需要我协助分析、编写或重构？",

    context_tokens: "📊 上下文",
    input_placeholder: "输入任务需求、代码修改目标或调用工具... (↵ 发送)",
    send_btn: "发送 ↵",

    settings_title: "系统偏好与配置",
    settings_subtitle: "配置模型接口、智能体运行模式与界面显示",
    back_to_chat: "← 返回对话",
    settings_model_title: "模型与 API 接口",
    api_base_label: "API 接口地址 (Base URL)",
    api_key_label: "API 密钥 (API Key)",
    model_name_label: "模型标识 (Model Identifier)",
    language_label: "界面语言 (Language)",
    mode_label: "执行策略",
    mode_auto: "全自动执行 (Autonomous)",
    mode_manual: "逐步确认 (Review Steps)",
    save_btn: "保存设置",
    save_success: "配置已更新",

    quick_test: "⚡ 运行测试 (cargo test)",
    quick_clippy: "🔍 代码检查 (cargo clippy)",
    quick_build: "📦 构建发布 (cargo build)",
    nav_automations_desc: "已启用 2 个自动化工作流：Git 状态守护、CI 自动测试监控",
    nav_skills_desc: "原生工具就绪：bash, read_file, write_file, edit_file, grep, find",
};

pub const EN_US: I18nText = I18nText {
    app_title: "⚡ Volt Desktop (ADE)",
    new_conversation: "+ New Conversation",
    conversation_history: "💬 Chat History",
    automations: "⏱️ Automations",
    skills_tools: "🧩 Skills & Tools",
    projects_header: "PROJECTS",
    conversations_header: "RECENT TASKS",
    settings: "Settings",
    agent_online: "● Online (v0.1.0)",

    branch_label: "branch",
    path_label: "path",
    status_ready: "Ready (Volt Core)",
    status_running: "Agent Executing...",

    you: "You",
    agent_name: "Volt Agent",
    tool_result: "⚡ Tool Result",
    tool_duration: "duration",
    welcome_msg: "Hello! I am Volt, your native high-performance autonomous AI coding agent.\n⚡ Powered by Volt High-Performance Engine (120 FPS hardware accelerated).\n🛠️ Zero Electron, zero Node/V8 overhead, sub-millisecond cold start.\n\nWhat would you like to inspect, build, or edit today?",

    context_tokens: "📊 Context",
    input_placeholder: "Ask anything, describe a task, or run tools... (↵ Send)",
    send_btn: "Send ↵",

    settings_title: "Preferences & Configuration",
    settings_subtitle: "Configure LLM provider endpoints, agent autonomy, and interface",
    back_to_chat: "← Back to Chat",
    settings_model_title: "Model & API Provider",
    api_base_label: "API Base URL",
    api_key_label: "API Secret Key",
    model_name_label: "Model Identifier",
    language_label: "Interface Language",
    mode_label: "Execution Policy",
    mode_auto: "Autonomous (Full Auto)",
    mode_manual: "Review Steps",
    save_btn: "Save Preferences",
    save_success: "Settings Updated",

    quick_test: "⚡ Run Tests (cargo test)",
    quick_clippy: "🔍 Linter (cargo clippy)",
    quick_build: "📦 Build Release (cargo build)",
    nav_automations_desc: "2 Automations active: Git watcher daemon, CI auto-test trigger",
    nav_skills_desc: "Native tools loaded: bash, read_file, write_file, edit_file, grep, find",
};

impl Language {
    pub fn text(&self) -> &'static I18nText {
        match self {
            Language::Chinese => &ZH_CN,
            Language::English => &EN_US,
        }
    }
}
