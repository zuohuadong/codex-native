# Volt Desktop (Native AI Coding Agent)

A blazing-fast, lightweight native desktop AI coding assistant built with **Volt** native high-performance runtime.

---

## ⚡ Tech Stack Architecture

* **Desktop UI & Rendering**: Native GPU rendering engine (120 FPS hardware acceleration).
* **Agent Core**: **Volt Autonomous Runtime** (`pi-agent-core`):
  * Full autonomous tool execution loop (`read_file`, `write_file`, `edit_file`, `bash`, `grep`, `find`, `office`, `calc`).
  * Built-in **10+ Native High-Performance Skills**: Office spreadsheet/table analytics, Word (.docx) unpacker & text extractor, presentation slide decks (Marp/Slidev), ASCII charting, scientific math evaluator, descriptive statistics, linear regression, linear algebra/matrix math, loan PMT / financial modeling, unit conversion, and calendar business days.
  * Streaming model provider interface (OpenAI-compatible endpoints, Anthropic, Ollama, etc.).
  * Session history management and serialization.
* **Built-in Standalone CLIs (Pure Rust)**:
  * `office-cli`: High-speed tabular filtering, statistics, CSV/TSV <-> Markdown table conversion, docx text extraction, slides generation, and ASCII charts.
  * `calc-cli`: Scientific expression solver, sample statistics, bivariate linear regression, matrix operations, mortgage PMT & compound interest, unit converter, and business days calculation.
* **Ultra-Low Overhead**: Zero Electron, zero Node/V8 runtime overhead, native process startup without a bundled browser runtime. Performance depends on the workload and platform; no benchmark claims are made here.
* **No External Codex CLI**: Standalone native agent runtime without dependencies on npm or proprietary wrappers.

---

## 📁 Workspace Layout

```text
.
├── Cargo.toml                  # Cargo workspace definition
├── crates/
│   ├── pi-agent-core/          # Core agent runtime, tools & skills (headless)
│   │   ├── src/
│   │   │   ├── agent.rs        # Autonomous agent turn loop & event streaming
│   │   │   ├── bin/
│   │   │   │   ├── office_cli.rs # Standalone office-cli binary
│   │   │   │   └── calc_cli.rs   # Standalone calc-cli binary
│   │   │   ├── provider.rs     # Streaming LLM client
│   │   │   ├── session.rs      # Session state & persistence
│   │   │   ├── skills/         # Native skills registry & system prompt injection
│   │   │   ├── tools/          # Native tools: bash, fs, search, office, calc
│   │   │   └── types.rs        # Message, ToolCall, AgentEvent, ModelConfig
│   │   └── tests/              # End-to-end tool & skill integration tests
│   └── pi-desktop/             # Volt desktop GUI application
│       └── src/
│           ├── i18n.rs         # Bilingual text (ZH/EN) & skill categories
│           └── main.rs         # Native desktop window, chat interface & skills dashboard
└── .github/workflows/
    └── build.yml               # Multi-platform CI pipeline (macOS + Windows)
```

---

## 🚀 Quick Start

### Prerequisites
* Rust toolchain (1.80+ or latest stable)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Run Tests
```bash
cargo test --workspace
```

### Run Desktop App
```bash
cargo run --bin volt
# 或者直接运行编译出的原生二进制文件：
./volt
```

---

## 🪟 Windows 自安装与系统集成 (Self-Installation)

Volt 提供了现代跨平台桌面工具标准的**免提权用户级自安装系统**（参考 VS Code User Setup、Rustup 与 Ollama 方案）：

### 1. 单文件自安装（二进制原生支持）
直接下载或分发单文件 `volt.exe`，无需外部安装包即可实现自部署：
```powershell
# 一键自安装至 %LOCALAPPDATA%\Programs\Volt，配置桌面/开始菜单快捷方式与全局环境变量 PATH
.\volt.exe --install

# 纯便携运行模式（不写注册表与系统路径）
.\volt.exe --portable

# 查看当前安装状态
.\volt.exe --install-status

# 干净卸载（清理快捷方式、环境变量与文件）
.\volt.exe --uninstall
```

### 2. PowerShell 一键远程/本地快速安装
通过一行 PowerShell 命令自动安装并启动：
```powershell
# 本地脚本安装
.\scripts\install.ps1

# 远程一键部署
irm https://raw.githubusercontent.com/zuohuadong/codex-native/main/scripts/install.ps1 | iex
```

### 3. 图形界面内一键修复与集成
打开 Volt 桌面端后，点击左下角 **设置 (Settings) -> Windows 系统集成与自安装**，即可随时一键执行自安装或修复快捷方式。

---

## 📜 License
Apache-2.0

## Reproducible builds and releases

`Cargo.lock` is part of this application source. CI runs `cargo fmt --all -- --check`,
`cargo clippy --locked --workspace --all-targets -- -D warnings`, and
`cargo test --locked --workspace`. Use `--locked` when building release artifacts.

Pushing a `v*` tag runs the checks and builds, then publishes the Windows executable,
Windows bundle and macOS arm64 archive as GitHub Release assets. The PowerShell installer
resolves `latest` to one tag and checks the executable SHA256 before installation.
`-Version 0.1.0` selects tag `v0.1.0`. A remote installation requires a successfully
published release; local builds remain usable via `-LocalExePath`.

## Reports require source data

`office-cli report gen --file report.json --out report.md` renders supplied data without
inventing measurements, dates, target assessments or recommendations. The Agent's
`office` tool accepts the same JSON as its `report` property with `action: "report_gen"`.
Required input shape (illustrative data, replace with your actual data):

```json
{
  "title": "Delivery report",
  "author": "Engineering",
  "date": "2026-10-24",
  "summary": "Two completed deliveries recorded in the supplied dataset.",
  "kpis": [["Completed deliveries", "2"]],
  "table": {"headers": ["Project", "Deliveries"], "rows": [["Example", "2"]]}
}
```

Missing report fields, unknown query operators/columns, and invalid numeric array
members return errors. Statistical CLI input such as `1,invalid,3` is rejected in full.
