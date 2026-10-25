use anyhow::{bail, Context, Result};
use std::{path::Path, time::Duration};
use tokio::process::Command;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum ReviewMode {
    #[default]
    Unstaged,
    Staged,
    Untracked,
}
#[derive(Default)]
pub struct ReviewState {
    pub mode: ReviewMode,
    pub files: Vec<String>,
    pub selected: Option<String>,
    pub diff: String,
    pub loading: bool,
    pub error: Option<String>,
    pub generation: u64,
}
pub struct ReviewSnapshot {
    pub files: Vec<String>,
    pub selected: Option<String>,
    pub diff: String,
}

async fn git(workspace: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = tokio::time::timeout(
        Duration::from_secs(15),
        Command::new("git")
            .args(["--no-pager", "-c", "color.ui=false"])
            .args(args)
            .current_dir(workspace)
            .env("GIT_OPTIONAL_LOCKS", "0")
            .kill_on_drop(true)
            .output(),
    )
    .await
    .context("Git timed out; refresh to retry")??;
    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(output.stdout)
}
pub async fn snapshot(
    workspace: &Path,
    mode: ReviewMode,
    selected: Option<String>,
) -> Result<ReviewSnapshot> {
    let args = match mode {
        ReviewMode::Unstaged => vec![
            "diff",
            "--no-ext-diff",
            "--no-textconv",
            "--name-only",
            "-z",
        ],
        ReviewMode::Staged => vec![
            "diff",
            "--cached",
            "--no-ext-diff",
            "--no-textconv",
            "--name-only",
            "-z",
        ],
        ReviewMode::Untracked => vec!["ls-files", "--others", "--exclude-standard", "-z"],
    };
    let bytes = git(workspace, &args).await?;
    let files: Vec<_> = bytes
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
        .map(|name| String::from_utf8_lossy(name).into_owned())
        .collect();
    let selected = selected
        .filter(|path| files.contains(path))
        .or_else(|| files.first().cloned());
    let diff = if let Some(path) = &selected {
        if mode == ReviewMode::Untracked {
            let full = workspace.join(path);
            match untracked_preview(workspace, &full).await {
                Ok(text) => text,
                Err(error) => format!("Preview unavailable: {error}"),
            }
        } else {
            let mut args = vec!["diff", "--no-ext-diff", "--no-textconv", "--no-color"];
            if mode == ReviewMode::Staged {
                args.push("--cached");
            }
            args.extend(["--", path]);
            preview(git(workspace, &args).await?)
        }
    } else {
        String::new()
    };
    Ok(ReviewSnapshot {
        files,
        selected,
        diff,
    })
}
async fn untracked_preview(workspace: &Path, full: &Path) -> Result<String> {
    let canonical = tokio::fs::canonicalize(full)
        .await
        .context("file cannot be read")?;
    let root = tokio::fs::canonicalize(workspace)
        .await
        .context("workspace cannot be read")?;
    if !canonical.starts_with(root) {
        bail!("file resolves outside this workspace");
    }
    let metadata = tokio::fs::metadata(&canonical)
        .await
        .context("file metadata unavailable")?;
    if !metadata.is_file() || metadata.len() > 1_048_576 {
        bail!("not a regular file or larger than 1 MiB");
    }
    Ok(preview(
        tokio::fs::read(canonical)
            .await
            .context("file cannot be read")?,
    ))
}

fn preview(mut bytes: Vec<u8>) -> String {
    if bytes.contains(&0) {
        return "Binary file; text preview unavailable.".into();
    }
    let truncated = bytes.len() > 1_048_576;
    bytes.truncate(1_048_576);
    let mut text = String::from_utf8_lossy(&bytes).into_owned();
    if truncated {
        text.push_str("\n[Preview truncated at 1 MiB]");
    }
    text
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn reads_staged_unstaged_and_preserves_list_on_preview_failure() {
        let directory = tempfile::tempdir().unwrap();
        let workspace = directory.path();
        git(workspace, &["init", "-q"]).await.unwrap();
        tokio::fs::write(workspace.join("file with space.txt"), "before\n")
            .await
            .unwrap();
        git(workspace, &["add", "--", "file with space.txt"])
            .await
            .unwrap();
        git(
            workspace,
            &[
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
                "commit",
                "-qm",
                "fixture",
            ],
        )
        .await
        .unwrap();
        tokio::fs::write(workspace.join("file with space.txt"), "after\n")
            .await
            .unwrap();
        let unstaged = snapshot(workspace, ReviewMode::Unstaged, None)
            .await
            .unwrap();
        assert_eq!(unstaged.files, vec!["file with space.txt"]);
        assert!(unstaged.diff.contains("+after"));
        assert!(snapshot(workspace, ReviewMode::Staged, None)
            .await
            .unwrap()
            .files
            .is_empty());
        git(workspace, &["add", "--", "file with space.txt"])
            .await
            .unwrap();
        assert!(snapshot(workspace, ReviewMode::Staged, None)
            .await
            .unwrap()
            .diff
            .contains("+after"));
        tokio::fs::write(workspace.join("readable.txt"), "readable")
            .await
            .unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(workspace.join("missing"), workspace.join("broken"))
                .unwrap();
            let broken = snapshot(workspace, ReviewMode::Untracked, Some("broken".into()))
                .await
                .unwrap();
            assert!(broken.files.contains(&"readable.txt".to_owned()));
            assert!(broken.diff.contains("Preview unavailable"));
        }
        let readable = snapshot(
            workspace,
            ReviewMode::Untracked,
            Some("readable.txt".into()),
        )
        .await
        .unwrap();
        assert_eq!(readable.diff, "readable");
    }
    #[tokio::test]
    async fn non_repository_is_an_error_not_a_clean_tree() {
        let directory = tempfile::tempdir().unwrap();
        assert!(snapshot(directory.path(), ReviewMode::Unstaged, None)
            .await
            .is_err());
    }
    #[test]
    fn binary_and_large_previews_are_explicit() {
        assert!(preview(vec![0]).contains("Binary"));
        assert!(preview(vec![b'a'; 1_048_577]).ends_with("[Preview truncated at 1 MiB]"));
    }
}
