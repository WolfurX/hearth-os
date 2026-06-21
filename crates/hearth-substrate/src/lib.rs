//! # hearth-substrate — the sovereignty substrate
//!
//! The most important subsystem, because the steward has root-equivalent reach: **every
//! mutating action is a transaction that snapshots first**, so one gesture undoes anything.
//!
//! On a real Hearth this is a **btrfs snapshot** of the subvolume (cheap, copy-on-write).
//! Here it is a portable, faithful stand-in: a recursive copy of the protected state into a
//! snapshot store, an append-only transaction log, and `undo` that restores a snapshot. The
//! interface is what matters; the mechanism swaps for btrfs on the real target.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// One recorded transaction: a snapshot taken before a mutating action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Txn {
    pub id: u64,
    pub ts: u64,
    pub summary: String,
    pub snapshot: PathBuf,
    #[serde(default)]
    pub undone: bool,
}

/// Guards a `protected` directory (the steward's mutable state — the Brain). Snapshots and
/// the transaction log live in `store`, a sibling that is never inside `protected`.
pub struct Substrate {
    protected: PathBuf,
    store: PathBuf,
}

impl Substrate {
    pub fn new(protected: impl Into<PathBuf>, store: impl Into<PathBuf>) -> Self {
        Self { protected: protected.into(), store: store.into() }
    }

    fn log_path(&self) -> PathBuf {
        self.store.join("transactions.jsonl")
    }
    fn snaps_dir(&self) -> PathBuf {
        self.store.join("snapshots")
    }

    /// The transaction timeline, oldest first.
    pub fn timeline(&self) -> Result<Vec<Txn>> {
        let p = self.log_path();
        if !p.exists() {
            return Ok(vec![]);
        }
        let mut out = vec![];
        for line in std::fs::read_to_string(&p)?.lines() {
            if line.trim().is_empty() {
                continue;
            }
            out.push(serde_json::from_str(line).with_context(|| format!("parsing txn: {line}"))?);
        }
        Ok(out)
    }

    /// Snapshot the protected state, run the action, then record the transaction. If the
    /// action fails, no transaction is recorded (the snapshot is harmless and reusable).
    pub fn transact<T>(&self, summary: &str, action: impl FnOnce() -> Result<T>) -> Result<(u64, T)> {
        std::fs::create_dir_all(self.snaps_dir())?;
        let id = self.timeline()?.iter().map(|t| t.id).max().unwrap_or(0) + 1;
        let snap = self.snaps_dir().join(id.to_string());
        copy_tree(&self.protected, &snap).context("snapshotting before the action")?;

        let result = action()?;

        let txn = Txn { id, ts: now(), summary: summary.to_string(), snapshot: snap, undone: false };
        append_line(&self.log_path(), &serde_json::to_string(&txn)?)?;
        Ok((id, result))
    }

    /// Undo a transaction — the most recent not-yet-undone, or a specific id — by restoring
    /// its snapshot over the protected state.
    pub fn undo(&self, which: Option<u64>) -> Result<Txn> {
        let mut txns = self.timeline()?;
        let idx = match which {
            Some(id) => txns.iter().rposition(|t| t.id == id),
            None => txns.iter().rposition(|t| !t.undone),
        }
        .context("nothing to undo")?;

        let txn = txns[idx].clone();
        restore_tree(&txn.snapshot, &self.protected).context("restoring the snapshot")?;
        txns[idx].undone = true;

        let body: String = txns
            .iter()
            .map(|t| serde_json::to_string(t).unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(self.log_path(), body + "\n")?;
        Ok(txn)
    }
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn append_line(path: &Path, line: &str) -> Result<()> {
    use std::io::Write;
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p)?;
    }
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(f, "{line}")?;
    Ok(())
}

/// Entries never copied into / cleared from a snapshot: git internals, and the snapshot store
/// itself — so a protected dir that *contains* the store never recurses or self-deletes.
fn skip_in_snapshot(name: &std::ffi::OsStr) -> bool {
    name == ".git" || name == ".substrate"
}

/// Recursively copy `src` → `dst`, skipping `.git` and the snapshot store.
fn copy_tree(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        if skip_in_snapshot(&entry.file_name()) {
            continue;
        }
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_tree(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// Restore `snap` → `dst`: clear `dst` (except `.git`), then copy the snapshot back.
fn restore_tree(snap: &Path, dst: &Path) -> Result<()> {
    if dst.exists() {
        for entry in std::fs::read_dir(dst)? {
            let entry = entry?;
            if skip_in_snapshot(&entry.file_name()) {
                continue;
            }
            let p = entry.path();
            if p.is_dir() {
                std::fs::remove_dir_all(&p)?;
            } else {
                std::fs::remove_file(&p)?;
            }
        }
    }
    copy_tree(snap, dst)
}
