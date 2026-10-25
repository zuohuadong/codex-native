pub mod bash;
pub mod calc;
pub mod fs;
pub mod office;
pub mod search;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn parameters_schema(&self) -> Value;
    async fn execute(&self, arguments: Value) -> Result<String>;
}

#[derive(Default, Clone)]
pub struct ToolRegistry {
    tools: Vec<Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    pub fn with_standard_tools(workspace_dir: impl Into<std::path::PathBuf>) -> Self {
        let ws = workspace_dir.into();
        let mut registry = Self::new();
        registry.register(Arc::new(bash::BashTool::new(ws.clone())));
        registry.register(Arc::new(fs::ReadFileTool::new(ws.clone())));
        registry.register(Arc::new(fs::WriteFileTool::new(ws.clone())));
        registry.register(Arc::new(fs::EditFileTool::new(ws.clone())));
        registry.register(Arc::new(search::GrepTool::new(ws.clone())));
        registry.register(Arc::new(search::FindTool::new(ws.clone())));
        registry.register(Arc::new(calc::CalcTool::new()));
        registry.register(Arc::new(office::OfficeTool::new(ws)));
        registry
    }

    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.push(tool);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.iter().find(|t| t.name() == name).cloned()
    }

    pub fn to_schema_json(&self) -> Value {
        let tools: Vec<Value> = self
            .tools
            .iter()
            .map(|t| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": t.name(),
                        "description": t.description(),
                        "parameters": t.parameters_schema(),
                    }
                })
            })
            .collect();
        Value::Array(tools)
    }
}
