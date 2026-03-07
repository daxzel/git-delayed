use anyhow::Result;
use std::path::Path;
use std::process::Command;

#[derive(Debug)]
pub enum PushResult {
    Success(()),
    NothingToPush,
}

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

// run git push in the specified repo, optionally switching to a specific branch
pub fn execute_push_with_branch(repo_path: &Path, branch: Option<&str>) -> Result<PushResult> {
    let current_branch = crate::git::get_current_branch(repo_path)?;
    let target_branch = branch.unwrap_or(&current_branch);
    
    // check if we need to push
    if !crate::git::needs_push(repo_path, target_branch)? {
        return Ok(PushResult::NothingToPush);
    }
    
    // stash any changes if present
    let has_changes = crate::git::has_unstaged_changes(repo_path)?;
    let mut stashed = false;
    
    if has_changes {
        let stash = Command::new("git")
            .args(["stash", "push", "-u", "-m", "git-delayed auto-stash"])
            .current_dir(repo_path)
            .output()?;
        
        if stash.status.success() {
            stashed = true;
        }
    }
    
    // switch to target branch if needed
    let mut switched = false;
    if current_branch != target_branch {
        let checkout = Command::new("git")
            .args(["checkout", target_branch])
            .current_dir(repo_path)
            .output()?;
        
        if !checkout.status.success() {
            // unstash before returning error
            if stashed {
                let _ = Command::new("git")
                    .args(["stash", "pop"])
                    .current_dir(repo_path)
                    .output();
            }
            return Err(anyhow::anyhow!(
                "couldn't switch to branch {}: {}",
                target_branch,
                String::from_utf8_lossy(&checkout.stderr)
            ));
        }
        switched = true;
    }
    
    // do the push
    let output = Command::new("git")
        .arg("push")
        .current_dir(repo_path)
        .output()?;
    
    let push_result = if output.status.success() {
        Ok(PushResult::Success(()))
    } else {
        Err(anyhow::anyhow!(
            "push failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    };
    
    // switch back to original branch if we changed it
    if switched {
        let _ = Command::new("git")
            .args(["checkout", &current_branch])
            .current_dir(repo_path)
            .output();
    }
    
    // unstash changes if we stashed them
    if stashed {
        let _ = Command::new("git")
            .args(["stash", "pop"])
            .current_dir(repo_path)
            .output();
    }

    push_result
}




// run git merge: checkout target branch, merge source branch, then switch back
pub fn execute_merge(repo_path: &Path, source_branch: &str, target_branch: &str) -> Result<String> {
    let current_branch = crate::git::get_current_branch(repo_path)?;

    let has_changes = crate::git::has_unstaged_changes(repo_path)?;
    let mut stashed = false;
    if has_changes {
        let stash = Command::new("git")
            .args(["stash", "push", "-u", "-m", "git-delayed auto-stash"])
            .current_dir(repo_path)
            .output()?;
        if stash.status.success() {
            stashed = true;
        }
    }

    let cleanup = |stashed: bool, switched: bool, current: &str, repo: &Path| {
        if switched {
            let _ = Command::new("git").args(["checkout", current]).current_dir(repo).output();
        }
        if stashed {
            let _ = Command::new("git").args(["stash", "pop"]).current_dir(repo).output();
        }
    };

    // checkout target branch
    let checkout = Command::new("git")
        .args(["checkout", target_branch])
        .current_dir(repo_path)
        .output()?;
    if !checkout.status.success() {
        cleanup(stashed, false, &current_branch, repo_path);
        return Err(anyhow::anyhow!(
            "couldn't switch to branch {}: {}",
            target_branch,
            String::from_utf8_lossy(&checkout.stderr)
        ));
    }

    // merge source branch with squash
    let output = Command::new("git")
        .args(["merge", "--squash", source_branch])
        .current_dir(repo_path)
        .output()?;

    let result = if output.status.success() {
        // get the last commit message from the source branch
        let log_output = Command::new("git")
            .args(["log", "-1", "--format=%B", source_branch])
            .current_dir(repo_path)
            .output()?;
        let msg = String::from_utf8_lossy(&log_output.stdout).trim().to_string();
        let commit_msg = if msg.is_empty() {
            format!("merge {} into {}", source_branch, target_branch)
        } else {
            msg
        };

        let commit = Command::new("git")
            .args(["commit", "-m", &commit_msg])
            .current_dir(repo_path)
            .output()?;
        if commit.status.success() {
            Ok(String::from_utf8_lossy(&commit.stdout).to_string())
        } else {
            Err(anyhow::anyhow!(
                "squash commit failed: {}",
                String::from_utf8_lossy(&commit.stderr)
            ))
        }
    } else {
        let _ = Command::new("git").args(["merge", "--abort"]).current_dir(repo_path).output();
        Err(anyhow::anyhow!(
            "merge failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    };

    cleanup(stashed, current_branch != target_branch, &current_branch, repo_path);
    result
}
