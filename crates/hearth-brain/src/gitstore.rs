//! Versioned memory. The Brain's data dir is its own git repo so that `forget` is
//! auditable and any consolidation can be rolled back — the glass-box tenet applied
//! to memory. If git isn't available we degrade quietly: the Brain still works, just
//! without history.

use anyhow::Result;
use std::path::Path;
use std::process::{Command, Stdio};

/// Initialize a git repo in `dir` if there isn't one already.
pub fn init_if_needed(dir: &Path) -> Result<()> {
    if dir.join(".git").exists() {
        return Ok(());
    }
    run(dir, &["init", "-q"]);
    run(dir, &["config", "user.name", "hearth-brain"]);
    run(dir, &["config", "user.email", "brain@hearth.local"]);
    Ok(())
}

/// Stage everything and commit. No-op if nothing changed or git is missing.
pub fn commit_all(dir: &Path, message: &str) -> Result<()> {
    if !dir.join(".git").exists() {
        return Ok(());
    }
    run(dir, &["add", "-A"]);
    run(dir, &["commit", "-q", "-m", message]);
    Ok(())
}

fn run(dir: &Path, args: &[&str]) {
    let _ = Command::new("git")
        .current_dir(dir)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}
