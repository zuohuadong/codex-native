# Pi Desktop (Native AI Coding Agent)

A blazing-fast, lightweight native desktop AI coding assistant built entirely in **Rust** using **GPUI Kit** and the **Pi Agent** autonomous runtime.

---

## ⚡ Tech Stack Architecture

* **Desktop UI & Rendering**: [GPUI Kit](https://github.com/longbridge/gpui-kit) (Built upon Zed's 120 FPS GPU rendering engine GPUI, featuring 75+ production-grade native components).
* **Agent Core**: **Pi Rust Runtime** (`pi-agent-core`), inspired by the Pi agent architecture:
  * Full autonomous tool execution loop (`read_file`, `write_file`, `edit_file`, `bash`, `grep`, `find`).
  * Streaming model provider interface (OpenAI-compatible endpoints, DeepSeek, Ollama, Claude, etc.).
  * Session history management and serialization.
* **Pure Rust**: Zero Electron, zero Node/V8 runtime overhead, sub-millisecond cold starts, and minimal memory consumption.
* **No External Codex CLI**: Standalone native agent runtime without dependencies on npm or proprietary wrappers.

---

## 📁 Workspace Layout

```text
.
├── Cargo.toml                  # Cargo workspace definition
├── crates/
│   ├── pi-agent-core/          # Core Pi agent runtime & tools (headless)
│   │   ├── src/
│   │   │   ├── agent.rs        # Autonomous agent turn loop & event streaming
│   │   │   ├── provider.rs     # Streaming LLM client
│   │   │   ├── session.rs      # Session state & persistence
│   │   │   ├── tools/          # Native tools: bash, fs (read/write/edit), search (find/grep)
│   │   │   └── types.rs        # Message, ToolCall, AgentEvent, ModelConfig
│   │   └── tests/              # End-to-end tool integration tests
│   └── pi-desktop/             # GPUI Kit desktop GUI application
│       └── src/
│           └── main.rs         # Native desktop window, chat interface & event dispatch
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
cargo run -p pi-desktop
```

---

## 📜 License
Apache-2.0
