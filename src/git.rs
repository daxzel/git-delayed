use anyhow::{Context, Result};
use git2::Repository;
use std::env;
use std::path::{Path, PathBuf};

// find the git repo we're currently in
// walks up the directory tree looking for .git
pub fn get_repository_path() -> Result<PathBuf> {
    let current_dir = env::current_dir().context("couldn't get current dir")?;

    let repo = Repository::discover(&current_dir)
        .context("not in a git repo. run this from inside a git repository.")?;

    let workdir = repo
        .workdir()
        .context("repo has no working directory")?;

    Ok(workdir.to_path_buf())
}

// get the current branch name
pub fn get_current_branch(repo_path: &Path) -> Result<String> {
    let repo = Repository::open(repo_path)?;
    let head = repo.head()?;
    let branch = head
        .shorthand()
        .ok_or_else(|| anyhow::anyhow!("couldn't get branch name"))?;
    Ok(branch.to_string())
}

// check if there are unstaged changes
pub fn has_unstaged_changes(repo_path: &Path) -> Result<bool> {
    let repo = Repository::open(repo_path)?;
    let statuses = repo.statuses(None)?;
    Ok(!statuses.is_empty())
}

// check if branch needs push (has unpushed commits)
pub fn needs_push(repo_path: &Path, branch: &str) -> Result<bool> {
    let repo = Repository::open(repo_path)?;
    
    // get local branch
    let local_branch = repo.find_branch(branch, git2::BranchType::Local)?;
    let local_oid = local_branch
        .get()
        .target()
        .ok_or_else(|| anyhow::anyhow!("no local commit"))?;
    
    // try to get remote branch
    let remote_name = format!("origin/{}", branch);
    let remote_branch = repo.find_branch(&remote_name, git2::BranchType::Remote);
    
    match remote_branch {
        Ok(branch) => {
            let remote_oid = branch
                .get()
                .target()
                .ok_or_else(|| anyhow::anyhow!("no remote commit"))?;
            Ok(local_oid != remote_oid)
        }
        Err(_) => Ok(true), // no remote branch means we need to push
    }
}
