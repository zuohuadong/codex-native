use std::path::{Path, PathBuf};
use anyhow::Result;
use async_trait::async_trait;
use regex::Regex;
use serde_json::Value;
use walkdir::WalkDir;

use super::Tool;

pub struct GrepTool {
    workspace_dir: PathBuf,
}

impl GrepTool {
    pub fn new(workspace_dir: PathBuf) -> Self {
        Self { workspace_dir }
    }
}

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &'static str {
        "grep"
    }

    fn description(&self) -> &'static str {
        "Search file contents for a regex pattern within the workspace."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Regular expression pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Subdirectory or file path to search in (relative to workspace)",
                    "default": "."
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum matching lines to return",
                    "default": 100
                }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let pattern_str = arguments["pattern"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'pattern'"))?;
        let sub_path = arguments.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        let max_results = arguments.get("max_results").and_then(|v| v.as_u64()).unwrap_or(100) as usize;

        let root = if Path::new(sub_path).is_absolute() {
            PathBuf::from(sub_path)
        } else {
            self.workspace_dir.join(sub_path)
        };

        let re = Regex::new(pattern_str)?;
        let mut matches = Vec::new();

        for entry in WalkDir::new(&root).into_iter().filter_entry(|e| {
            if e.depth() == 0 {
                return true;
            }
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.') && name != "target" && name != "node_modules"
        }).filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            if let Ok(content) = std::fs::read_to_string(path) {
                let rel = path.strip_prefix(&self.workspace_dir).unwrap_or(path);
                for (idx, line) in content.lines().enumerate() {
                    if re.is_match(line) {
                        matches.push(format!("{}:{}: {}", rel.display(), idx + 1, line.trim()));
                        if matches.len() >= max_results {
                            break;
                        }
                    }
                }
            }
            if matches.len() >= max_results {
                break;
            }
        }

        if matches.is_empty() {
            Ok(format!("No matches found for regex: {:?}", pattern_str))
        } else {
            Ok(matches.join("\n"))
        }
    }
}

pub struct FindTool {
    workspace_dir: PathBuf,
}

impl FindTool {
    pub fn new(workspace_dir: PathBuf) -> Self {
        Self { workspace_dir }
    }
}

#[async_trait]
impl Tool for FindTool {
    fn name(&self) -> &'static str {
        "find"
    }

    fn description(&self) -> &'static str {
        "Find file and directory paths matching a substring or pattern in the workspace."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Substring to search for in file paths"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of results to return",
                    "default": 100
                }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let pattern_str = arguments["pattern"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'pattern'"))?
            .to_lowercase();
        let max_results = arguments.get("max_results").and_then(|v| v.as_u64()).unwrap_or(100) as usize;

        let mut matches = Vec::new();
        for entry in WalkDir::new(&self.workspace_dir).into_iter().filter_entry(|e| {
            if e.depth() == 0 {
                return true;
            }
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.') && name != "target" && name != "node_modules"
        }).filter_map(|e| e.ok()) {
            let rel = entry.path().strip_prefix(&self.workspace_dir).unwrap_or(entry.path());
            let rel_str = rel.to_string_lossy();
            if rel_str.to_lowercase().contains(&pattern_str) {
                matches.push(rel_str.to_string());
                if matches.len() >= max_results {
                    break;
                }
            }
        }

        if matches.is_empty() {
            Ok(format!("No files found matching {:?}", pattern_str))
        } else {
            Ok(matches.join("\n"))
        }
    }
}
