use std::path::{Path, PathBuf};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use serde_json::Value;

use super::Tool;

pub struct ReadFileTool {
    workspace_dir: PathBuf,
}

impl ReadFileTool {
    pub fn new(workspace_dir: PathBuf) -> Self {
        Self { workspace_dir }
    }

    fn resolve_path(&self, p: &str) -> PathBuf {
        let path = Path::new(p);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.workspace_dir.join(path)
        }
    }
}

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &'static str {
        "read_file"
    }

    fn description(&self) -> &'static str {
        "Read contents of a file with optional line range (start_line, end_line, 1-indexed)."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to read"
                },
                "start_line": {
                    "type": "integer",
                    "description": "Optional start line number (1-indexed, inclusive)"
                },
                "end_line": {
                    "type": "integer",
                    "description": "Optional end line number (1-indexed, inclusive)"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let path_str = arguments["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'path'"))?;
        let full_path = self.resolve_path(path_str);

        let content = tokio::fs::read_to_string(&full_path)
            .await
            .with_context(|| format!("Failed to read file: {:?}", full_path))?;

        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        let start = arguments
            .get("start_line")
            .and_then(|v| v.as_u64())
            .map(|v| v.max(1) as usize)
            .unwrap_or(1);

        let end = arguments
            .get("end_line")
            .and_then(|v| v.as_u64())
            .map(|v| (v as usize).min(total_lines))
            .unwrap_or(total_lines);

        if start > total_lines {
            return Ok(format!("File has only {} lines (requested start at line {})", total_lines, start));
        }

        let slice = &lines[(start - 1)..end];
        let mut out = String::new();
        for (i, line) in slice.iter().enumerate() {
            out.push_str(&format!("{:4} | {}\n", start + i, line));
        }
        Ok(out)
    }
}

pub struct WriteFileTool {
    workspace_dir: PathBuf,
}

impl WriteFileTool {
    pub fn new(workspace_dir: PathBuf) -> Self {
        Self { workspace_dir }
    }

    fn resolve_path(&self, p: &str) -> PathBuf {
        let path = Path::new(p);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.workspace_dir.join(path)
        }
    }
}

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &'static str {
        "write_file"
    }

    fn description(&self) -> &'static str {
        "Write full content to a file (creates directories if they do not exist)."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to create or overwrite"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write into the file"
                }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let path_str = arguments["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'path'"))?;
        let content = arguments["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'content'"))?;

        let full_path = self.resolve_path(path_str);
        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&full_path, content).await?;
        Ok(format!("Successfully wrote {} bytes to {:?}", content.len(), full_path))
    }
}

pub struct EditFileTool {
    workspace_dir: PathBuf,
}

impl EditFileTool {
    pub fn new(workspace_dir: PathBuf) -> Self {
        Self { workspace_dir }
    }

    fn resolve_path(&self, p: &str) -> PathBuf {
        let path = Path::new(p);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.workspace_dir.join(path)
        }
    }
}

#[async_trait]
impl Tool for EditFileTool {
    fn name(&self) -> &'static str {
        "edit_file"
    }

    fn description(&self) -> &'static str {
        "Edit a file by precisely replacing an existing target string with new content."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to edit"
                },
                "target_content": {
                    "type": "string",
                    "description": "The exact string within the file to replace"
                },
                "replacement_content": {
                    "type": "string",
                    "description": "The replacement string"
                }
            },
            "required": ["path", "target_content", "replacement_content"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let path_str = arguments["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'path'"))?;
        let target = arguments["target_content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'target_content'"))?;
        let replacement = arguments["replacement_content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'replacement_content'"))?;

        let full_path = self.resolve_path(path_str);
        let content = tokio::fs::read_to_string(&full_path).await?;

        let count = content.matches(target).count();
        if count == 0 {
            bail!("target_content not found in {:?}", full_path);
        }
        if count > 1 {
            bail!("target_content found {} times in {:?}; must be unique", count, full_path);
        }

        let new_content = content.replacen(target, replacement, 1);
        tokio::fs::write(&full_path, new_content).await?;
        Ok(format!("Successfully replaced 1 occurrence in {:?}", full_path))
    }
}
