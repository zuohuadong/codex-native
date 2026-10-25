use anyhow::{bail, Result};
use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

use super::Tool;

#[cfg(unix)]
struct ProcessGroup {
    id: u32,
    armed: bool,
}

#[cfg(unix)]
impl Drop for ProcessGroup {
    fn drop(&mut self) {
        // The child creates its own group; cancellation must also stop its descendants.
        if self.armed {
            unsafe {
                libc::kill(-(self.id as i32), libc::SIGKILL);
            }
        }
    }
}

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
            .current_dir(&self.workspace_dir)
            .kill_on_drop(true)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        #[cfg(unix)]
        cmd.process_group(0);

        let child = cmd.spawn()?;
        #[cfg(unix)]
        let mut group = ProcessGroup {
            id: child.id().expect("just spawned child"),
            armed: true,
        };
        let run_future = child.wait_with_output();
        let output = match timeout(Duration::from_secs(timeout_secs), run_future).await {
            Ok(res) => {
                #[cfg(unix)]
                {
                    group.armed = false;
                }
                res?
            }
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
            bail!("{}\n[process exited with code {}]", res, code);
        }

        if res.trim().is_empty() {
            Ok("(command completed with no output)".to_string())
        } else {
            Ok(res)
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn nonzero_exit_is_a_tool_failure() {
        let directory = tempfile::tempdir().unwrap();
        let tool = BashTool::new(directory.path().into());
        let error = tool
            .execute(serde_json::json!({"command":"printf failure >&2; exit 7"}))
            .await
            .unwrap_err();
        assert!(error.to_string().contains("failure"));
        assert!(error.to_string().contains("code 7"));
    }

    #[tokio::test]
    async fn cancelling_kills_descendants_before_their_next_write() {
        let directory = tempfile::tempdir().unwrap();
        let tool = BashTool::new(directory.path().into());
        let worker = tokio::spawn(async move {
            tool.execute(
                serde_json::json!({"command":"(touch ready; sleep 1; touch escaped) & wait"}),
            )
            .await
        });
        tokio::time::timeout(Duration::from_secs(5), async {
            while !directory.path().join("ready").exists() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        worker.abort();
        assert!(worker.await.unwrap_err().is_cancelled());
        tokio::time::sleep(Duration::from_millis(1200)).await;
        assert!(!directory.path().join("escaped").exists());
    }
}
