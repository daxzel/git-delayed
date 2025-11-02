use anyhow::{Context, Result};
use git2::Repository;
use std::env;
use std::path::PathBuf;

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
