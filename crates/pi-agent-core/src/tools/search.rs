use anyhow::Result;
use async_trait::async_trait;
use regex::Regex;
use serde_json::Value;
use std::path::{Path, PathBuf};
use tokio::fs::{self, ReadDir};

use super::Tool;

/// Depth-first traversal whose filesystem operations and directory boundaries
/// yield to the runtime. Dropping a search drops this traversal as well.
struct SearchWalk {
    root: Option<PathBuf>,
    pending_dir: Option<PathBuf>,
    directories: Vec<ReadDir>,
}
impl SearchWalk {
    fn new(root: PathBuf) -> Self {
        Self {
            root: Some(root),
            pending_dir: None,
            directories: Vec::new(),
        }
    }

    async fn next(&mut self) -> Option<(PathBuf, bool)> {
        if let Some(root) = self.root.take() {
            let metadata = fs::metadata(&root).await.ok()?;
            if metadata.is_dir() {
                self.pending_dir = Some(root.clone());
            }
            return Some((root, metadata.is_file()));
        }
        loop {
            tokio::task::yield_now().await;
            if let Some(path) = self.pending_dir.take() {
                if let Ok(directory) = fs::read_dir(path).await {
                    self.directories.push(directory);
                }
            }
            let directory = self.directories.last_mut()?;
            let entry = match directory.next_entry().await {
                Ok(Some(entry)) => entry,
                Ok(None) | Err(_) => {
                    self.directories.pop();
                    continue;
                }
            };
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }
            let Ok(kind) = entry.file_type().await else {
                continue;
            };
            let path = entry.path();
            if kind.is_dir() {
                self.pending_dir = Some(path.clone());
            }
            return Some((path, kind.is_file()));
        }
    }
}

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
        let sub_path = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");
        let max_results = arguments
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(100) as usize;

        let root = if Path::new(sub_path).is_absolute() {
            PathBuf::from(sub_path)
        } else {
            self.workspace_dir.join(sub_path)
        };

        let re = Regex::new(pattern_str)?;
        let mut matches = Vec::new();

        let mut entries = SearchWalk::new(root);
        while let Some((path, is_file)) = entries.next().await {
            if !is_file {
                continue;
            }

            if let Ok(content) = fs::read_to_string(&path).await {
                let rel = path.strip_prefix(&self.workspace_dir).unwrap_or(&path);
                for (idx, line) in content.lines().enumerate() {
                    if idx % 256 == 0 {
                        tokio::task::yield_now().await;
                    }
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
        let max_results = arguments
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(100) as usize;

        let mut matches = Vec::new();
        let mut entries = SearchWalk::new(self.workspace_dir.clone());
        while let Some((path, _)) = entries.next().await {
            let rel = path.strip_prefix(&self.workspace_dir).unwrap_or(&path);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "current_thread")]
    async fn traversal_yields_and_can_be_cancelled_after_starting() {
        let root = tempfile::tempdir().unwrap();
        for index in 0..512 {
            std::fs::write(root.path().join(format!("file-{index}")), "evidence").unwrap();
        }
        let mut entries = SearchWalk::new(root.path().to_owned());
        let (started, waiting) = tokio::sync::oneshot::channel();
        let scan = tokio::spawn(async move {
            let mut started = Some(started);
            let mut count = 0;
            while entries.next().await.is_some() {
                count += 1;
                if count == 64 {
                    let _ = started.take().unwrap().send(());
                }
            }
            count
        });
        waiting.await.unwrap();
        // On a single-thread runtime this requires the traversal to yield while
        // there is still work, not merely before or after the complete scan.
        assert!(!scan.is_finished());
        scan.abort();
        assert!(scan.await.unwrap_err().is_cancelled());
    }

    #[tokio::test]
    async fn traversal_preserves_exclusions_and_nested_matches() {
        let root = tempfile::tempdir().unwrap();
        for directory in ["src", ".hidden", "target", "node_modules"] {
            std::fs::create_dir(root.path().join(directory)).unwrap();
            std::fs::write(root.path().join(directory).join("hit.txt"), "needle").unwrap();
        }
        let find = FindTool::new(root.path().to_owned())
            .execute(serde_json::json!({"pattern":"hit"}))
            .await
            .unwrap();
        assert_eq!(find, "src/hit.txt");
        let grep = GrepTool::new(root.path().to_owned())
            .execute(serde_json::json!({"pattern":"needle"}))
            .await
            .unwrap();
        assert_eq!(grep, "src/hit.txt:1: needle");
    }
}
