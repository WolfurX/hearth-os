//! # hearth-brain — the Brain
//!
//! Hearth OS's long-term memory, and the reason it gets smarter with use. It is an
//! **LLM-wiki** (Karpathy): raw activity is *compiled* into a curated, legible wiki,
//! governed by a schema. Three layers live on disk:
//!
//! ```text
//! <brain>/raw/activity.jsonl   ground truth — append-only, never rewritten
//! <brain>/wiki/*.md            compiled, curated markdown — what the steward knows
//! <brain>/schema.toml          the rules: how raw activity becomes knowledge
//! ```
//!
//! It is **local-first, legible, and model-portable**: plain markdown you can read,
//! edit, and forget; externalized text that survives swapping the model. The wiki is
//! its own git repo, so `forget` is auditable and any compile can be rolled back.

pub mod clock;
pub mod compile;
pub mod gitstore;
pub mod log;
pub mod recall;
pub mod schema;
pub mod wiki;

use anyhow::{Context, Result};
use schema::Schema;
use std::path::{Path, PathBuf};

/// Default location of the Brain's data: `$HEARTH_HOME/brain`, else `~/.hearth/brain`.
/// This is user memory, deliberately kept OUT of the source tree and given its own
/// git repo so history, `forget`, and rollback work.
pub fn default_brain_dir() -> Result<PathBuf> {
    if let Ok(h) = std::env::var("HEARTH_HOME") {
        return Ok(PathBuf::from(h).join("brain"));
    }
    let home = home_dir().context("could not determine home directory (set HEARTH_HOME)")?;
    Ok(home.join(".hearth").join("brain"))
}

/// Minimal, dependency-free home-directory resolution (keeps the build lean): `HOME`
/// on Unix, `USERPROFILE` (or `HOMEDRIVE`+`HOMEPATH`) on Windows.
fn home_dir() -> Option<PathBuf> {
    if let Some(h) = std::env::var_os("HOME") {
        return Some(PathBuf::from(h));
    }
    if let Some(h) = std::env::var_os("USERPROFILE") {
        return Some(PathBuf::from(h));
    }
    match (std::env::var_os("HOMEDRIVE"), std::env::var_os("HOMEPATH")) {
        (Some(d), Some(p)) => {
            let mut s = PathBuf::from(d);
            s.push(p);
            Some(s)
        }
        _ => None,
    }
}

/// A handle to a Brain on disk.
pub struct Brain {
    pub root: PathBuf,
    pub schema: Schema,
}

impl Brain {
    pub fn raw_dir(&self) -> PathBuf {
        self.root.join("raw")
    }
    pub fn wiki_dir(&self) -> PathBuf {
        self.root.join("wiki")
    }
    pub fn log_path(&self) -> PathBuf {
        self.raw_dir().join("activity.jsonl")
    }
    pub fn cursor_path(&self) -> PathBuf {
        self.raw_dir().join(".compiled")
    }
    pub fn schema_path(&self) -> PathBuf {
        self.root.join("schema.toml")
    }

    /// Open an existing Brain.
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let schema_path = root.join("schema.toml");
        anyhow::ensure!(
            schema_path.exists(),
            "no Brain at {} — run `hearth-brain init` first",
            root.display()
        );
        let schema = Schema::load(&schema_path)?;
        Ok(Self { root, schema })
    }

    /// Create a new Brain (idempotent): directories, default schema, seed pages, and
    /// its own git repo so memory has revision history from day one.
    pub fn init(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        std::fs::create_dir_all(root.join("raw"))?;
        for sub in ["people", "projects", "lessons"] {
            std::fs::create_dir_all(root.join("wiki").join(sub))?;
        }

        let schema_path = root.join("schema.toml");
        if !schema_path.exists() {
            std::fs::write(&schema_path, schema::DEFAULT_SCHEMA_TOML)?;
        }
        let log_path = root.join("raw").join("activity.jsonl");
        if !log_path.exists() {
            std::fs::write(&log_path, "")?;
        }
        let cursor = root.join("raw").join(".compiled");
        if !cursor.exists() {
            std::fs::write(&cursor, "0")?;
        }

        let schema = Schema::load(&schema_path)?;
        let brain = Self { root: root.clone(), schema };
        wiki::seed_pages(&brain.wiki_dir())?;

        gitstore::init_if_needed(&root)?;
        gitstore::commit_all(&root, "brain: initialize")?;
        Ok(brain)
    }
}
