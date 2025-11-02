use anyhow::Result;
use std::path::Path;
use std::process::Command;

// run git commit in the specified repo
pub fn execute_commit(repo_path: &Path, message: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(repo_path)
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(anyhow::anyhow!(
            "commit failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

// run git push in the specified repo
pub fn execute_push(repo_path: &Path) -> Result<String> {
    let output = Command::new("git")
        .arg("push")
        .current_dir(repo_path)
        .output()?;

    if output.status.success() {
        // git push writes to stderr even on success, so include both
        Ok(format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    } else {
        Err(anyhow::anyhow!(
            "push failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}


