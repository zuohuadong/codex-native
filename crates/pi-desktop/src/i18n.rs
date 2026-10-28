#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
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
    pub theme_label: &'static str,
    pub theme_auto: &'static str,
    pub theme_light: &'static str,
    pub theme_dark: &'static str,
    pub mode_label: &'static str,
    pub mode_auto: &'static str,
    pub mode_manual: &'static str,
    pub save_btn: &'static str,
    pub save_success: &'static str,

    // Quick Actions
    pub quick_test: &'static str,
    pub quick_clippy: &'static str,
    pub quick_build: &'static str,
    pub quick_office: &'static str,
    pub quick_calc: &'static str,
    pub nav_automations_desc: &'static str,
    pub nav_skills_desc: &'static str,
    pub skills_cat_all: &'static str,
    pub skills_cat_office: &'static str,
    pub skills_cat_scientific: &'static str,
    pub skills_cat_financial: &'static str,
    pub skills_cat_core: &'static str,
}

pub const ZH_CN: I18nText = I18nText {
    app_title: "Pi Desktop",
    new_conversation: "新建任务",
    conversation_history: "会话记录",
    automations: "自动化工作流",
    skills_tools: "技能与工具",
    projects_header: "工作空间",
    conversations_header: "近期会话",
    settings: "偏好设置",
    agent_online: "在线 · 120 FPS",

    branch_label: "分支",
    path_label: "路径",
    status_ready: "就绪 · 原生核心",
    status_running: "正在执行微秒级调度...",

    you: "用户",
    agent_name: "Pi Agent",
    tool_result: "工具执行输出",
    tool_duration: "耗时",
    welcome_msg: "你好！我是 Pi，基于 Rust 原生构建的高性能自主编程智能体。\n\n· ⚡ 120 FPS GPU 硬件加速渲染，极速流式响应\n· 🛠️ 纯 Rust 自主内核，内置系统级代码重构与工具调用链\n· 🚀 极致轻量：MB 级内存开销，毫秒级冷启动\n\n今天有什么需要我协助分析、构建或重构的代码任务？",

    context_tokens: "上下文",
    input_placeholder: "向 ZCode 提问，使用 @ 添加上下文，使用 / 选择命令或能力",
    send_btn: "发送",

    settings_title: "系统偏好与配置",
    settings_subtitle: "配置模型接口、智能体运行策略与系统显示",
    back_to_chat: "← 返回对话",
    settings_model_title: "模型与 API 接口",
    api_base_label: "Base URL",
    api_key_label: "API Key",
    model_name_label: "模型",
    language_label: "界面语言 (Language)",
    theme_label: "界面外观",
    theme_auto: "跟随系统",
    theme_light: "明亮",
    theme_dark: "暗黑",
    mode_label: "执行策略",
    mode_auto: "全自动执行 (Autonomous)",
    mode_manual: "逐步确认 (Review Steps)",
    save_btn: "保存配置",
    save_success: "配置已更新",

    quick_test: "运行测试",
    quick_clippy: "规范检查",
    quick_build: "编译构建",
    quick_office: "表格与报告",
    quick_calc: "科学计算",
    nav_automations_desc: "已启用 2 个自动化守护进程：Git 分支监听、CI 自动化测试监控",
    nav_skills_desc: "内置 Rust 原生办公套件 (office-cli)、科学计算 (calc-cli) 及全套智能体工具",
    skills_cat_all: "全部技能",
    skills_cat_office: "办公套件",
    skills_cat_scientific: "科学计算",
    skills_cat_financial: "金融与财务",
    skills_cat_core: "开发基座",
};

pub const EN_US: I18nText = I18nText {
    app_title: "Pi Desktop",
    new_conversation: "New Task",
    conversation_history: "Chat History",
    automations: "Automations",
    skills_tools: "Skills & Tools",
    projects_header: "WORKSPACE",
    conversations_header: "RECENT SESSIONS",
    settings: "Settings",
    agent_online: "Online · 120 FPS",

    branch_label: "branch",
    path_label: "path",
    status_ready: "Ready · Native Core",
    status_running: "Executing tasks...",

    you: "User",
    agent_name: "Pi Agent",
    tool_result: "Tool Execution Output",
    tool_duration: "duration",
    welcome_msg: "Hello! I am Pi, your native high-performance autonomous AI coding agent.\n\n· ⚡ 120 FPS GPU hardware acceleration & ultra-responsive streaming\n· 🛠️ Pure Rust autonomous runtime with native toolchain\n· 🚀 Ultra-lightweight: Minimal resident memory & sub-millisecond cold start\n\nWhat would you like to inspect, build, or refactor today?",

    context_tokens: "Context",
    input_placeholder: "Ask ZCode, use @ for context, / for commands",
    send_btn: "Send",

    settings_title: "Preferences & Configuration",
    settings_subtitle: "Configure LLM provider endpoints, agent autonomy, and interface",
    back_to_chat: "← Back to Chat",
    settings_model_title: "Model & API Provider",
    api_base_label: "Base URL",
    api_key_label: "API key",
    model_name_label: "Model",
    language_label: "Interface Language",
    theme_label: "Appearance",
    theme_auto: "System",
    theme_light: "Light",
    theme_dark: "Dark",
    mode_label: "Execution Policy",
    mode_auto: "Autonomous (Full Auto)",
    mode_manual: "Review Steps",
    save_btn: "Save Preferences",
    save_success: "Settings Updated",

    quick_test: "Run Tests",
    quick_clippy: "Lint & Clippy",
    quick_build: "Build Desktop",
    quick_office: "Office & Tables",
    quick_calc: "Scientific Calc",
    nav_automations_desc: "2 Automations active: Git watcher daemon, CI auto-test trigger",
    nav_skills_desc: "Built-in pure Rust office suite (office-cli), scientific computing (calc-cli), and agent tools",
    skills_cat_all: "All Skills",
    skills_cat_office: "Office Suite",
    skills_cat_scientific: "Scientific",
    skills_cat_financial: "Financial",
    skills_cat_core: "Core Agent",
};

impl Language {
    pub fn text(&self) -> &'static I18nText {
        match self {
            Language::Chinese => &ZH_CN,
            Language::English => &EN_US,
        }
    }
}
