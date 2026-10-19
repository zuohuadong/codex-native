use std::path::PathBuf;
use std::time::Duration;
use anyhow::{bail, Result};
use async_trait::async_trait;
use serde_json::Value;
use tokio::process::Command;
use tokio::time::timeout;

use super::Tool;

pub struct BashTool {
    workspace_dir: PathBuf,
}

impl BashTool {
    pub fn new(workspace_dir: PathBuf) -> Self {
        Self { workspace_dir }
    }
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &'static str {
        "bash"
    }

    fn description(&self) -> &'static str {
        "Run a shell command inside the workspace directory. Output stdout and stderr will be returned."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The exact shell command line string to execute"
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Optional timeout in seconds (default 60)",
                    "default": 60
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let command_str = arguments
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required 'command' argument"))?;

        let timeout_secs = arguments
            .get("timeout_secs")
            .and_then(|v| v.as_u64())
            .unwrap_or(60);

        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg(command_str)
            .current_dir(&self.workspace_dir);

        let run_future = cmd.output();
        let output = match timeout(Duration::from_secs(timeout_secs), run_future).await {
            Ok(res) => res?,
            Err(_) => bail!("Command timed out after {} seconds", timeout_secs),
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut res = String::new();
        if !stdout.is_empty() {
            res.push_str(&stdout);
        }
        if !stderr.is_empty() {
            if !res.is_empty() {
                res.push('\n');
            }
            res.push_str("[stderr]: ");
            res.push_str(&stderr);
        }

        if !output.status.success() {
            let code = output.status.code().unwrap_or(-1);
            res.push_str(&format!("\n[process exited with code {}]", code));
        }

        if res.trim().is_empty() {
            Ok("(command completed with no output)".to_string())
        } else {
            Ok(res)
        }
    }
}
