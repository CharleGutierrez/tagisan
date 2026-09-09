use crate::error::{Result, TagisanError};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

/// Git worktree sandbox providing isolated execution environments for multi-agent tasks
#[derive(Debug)]
pub struct WorktreeSandbox {
    repo_root: PathBuf,
    worktree_path: PathBuf,
    branch_name: String,
    base_commit: String,
    cleaned_up: bool,
}

impl WorktreeSandbox {
    /// Create a new isolated Git worktree sandbox with an auto-generated unique branch name
    pub fn new(repo_root: impl AsRef<Path>) -> Result<Self> {
        let branch_name = format!(
            "tagisan/sandbox-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        );
        Self::create(repo_root, branch_name)
    }

    /// Create a new isolated Git worktree sandbox branched from HEAD or `base_branch`
    pub fn create(repo_root: impl AsRef<Path>, branch_name: impl Into<String>) -> Result<Self> {
        let repo_root = repo_root.as_ref().to_path_buf();
        let branch_name = branch_name.into();

        // Capture base commit before adding worktree
        let base_commit = Command::new("git")
            .current_dir(&repo_root)
            .args(["rev-parse", "HEAD"])
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
            .unwrap_or_else(|_| "HEAD".to_string());

        let unique_id = format!(
            "tagisan_worktree_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        );
        let worktree_path = std::env::temp_dir().join(unique_id);

        info!(
            "Provisioning Git worktree sandbox on branch '{}' at '{}' (base: {})...",
            branch_name,
            worktree_path.display(),
            base_commit
        );

        let output = Command::new("git")
            .current_dir(&repo_root)
            .args([
                "worktree",
                "add",
                "-b",
                &branch_name,
                worktree_path.to_str().unwrap_or_default(),
                "HEAD",
            ])
            .output()
            .map_err(|e| TagisanError::Execution(format!("Failed to execute 'git worktree add': {e}")))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(TagisanError::Execution(format!(
                "Failed to provision git worktree sandbox: {err}"
            )));
        }

        Ok(Self {
            repo_root,
            worktree_path,
            branch_name,
            base_commit,
            cleaned_up: false,
        })
    }

    /// Access path to the isolated worktree directory
    pub fn path(&self) -> &Path {
        &self.worktree_path
    }

    /// Access the branch name of this sandbox
    pub fn branch(&self) -> &str {
        &self.branch_name
    }

    /// Run a command inside the isolated worktree
    pub fn run_command(&self, cmd: &str) -> Result<(i32, String, String)> {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", cmd])
                .current_dir(&self.worktree_path)
                .output()
        } else {
            Command::new("sh")
                .args(["-c", cmd])
                .current_dir(&self.worktree_path)
                .output()
        }
        .map_err(|e| TagisanError::Execution(format!("Failed to execute command '{cmd}': {e}")))?;

        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok((exit_code, stdout, stderr))
    }

    /// Stage and commit all changes in the isolated worktree
    pub fn commit_all(&self, message: &str) -> Result<String> {
        let add_out = Command::new("git")
            .current_dir(&self.worktree_path)
            .args(["add", "."])
            .output()
            .map_err(|e| TagisanError::Execution(format!("git add failed: {e}")))?;

        if !add_out.status.success() {
            return Err(TagisanError::Execution(
                String::from_utf8_lossy(&add_out.stderr).to_string(),
            ));
        }

        let commit_out = Command::new("git")
            .current_dir(&self.worktree_path)
            .args(["commit", "-m", message])
            .output()
            .map_err(|e| TagisanError::Execution(format!("git commit failed: {e}")))?;

        if !commit_out.status.success() {
            return Err(TagisanError::Execution(
                String::from_utf8_lossy(&commit_out.stderr).to_string(),
            ));
        }

        // Return new commit SHA
        let rev_out = Command::new("git")
            .current_dir(&self.worktree_path)
            .args(["rev-parse", "HEAD"])
            .output()
            .map_err(|e| TagisanError::Execution(format!("git rev-parse failed: {e}")))?;

        Ok(String::from_utf8_lossy(&rev_out.stdout).trim().to_string())
    }

    /// Get diff of changes against base commit
    pub fn diff(&self) -> Result<String> {
        let out = Command::new("git")
            .current_dir(&self.worktree_path)
            .args(["diff", &self.base_commit])
            .output()
            .map_err(|e| TagisanError::Execution(format!("git diff failed: {e}")))?;

        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }

    /// Access the base commit SHA this sandbox was branched from
    pub fn base_commit(&self) -> &str {
        &self.base_commit
    }

    /// Clean up the worktree and remove the branch if desired
    pub fn cleanup(&mut self) -> Result<()> {
        if self.cleaned_up {
            return Ok(());
        }

        info!(
            "Removing Git worktree at '{}'...",
            self.worktree_path.display()
        );

        let rm_out = Command::new("git")
            .current_dir(&self.repo_root)
            .args([
                "worktree",
                "remove",
                "--force",
                self.worktree_path.to_str().unwrap_or_default(),
            ])
            .output();

        if let Ok(out) = rm_out {
            if !out.status.success() {
                warn!(
                    "git worktree remove reported: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
        }

        // Delete temporary branch
        let _ = Command::new("git")
            .current_dir(&self.repo_root)
            .args(["branch", "-D", &self.branch_name])
            .output();

        self.cleaned_up = true;
        Ok(())
    }
}

impl Drop for WorktreeSandbox {
    fn drop(&mut self) {
        if !self.cleaned_up {
            let _ = self.cleanup();
        }
    }
}
