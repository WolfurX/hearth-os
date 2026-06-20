//! The raw activity log — the Brain's ground-truth layer (§3.5/§3.8).
//!
//! Append-only: text enters here and is never rewritten, with one explicit
//! exception — [`redact`], the git-tracked erasure invoked by `forget`. In the full
//! system this log is the sovereignty substrate's audit trail; here the Brain owns a
//! standalone version of it so Phase 1 can stand on its own.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// The kind of a raw event. Explicitly-kinded events route to the wiki by kind;
/// free-form ones (`interaction` / `note` / `action`) route by schema keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// Something the user said — a turn of dialogue.
    Interaction,
    /// An explicit, stated preference ("I prefer concise replies").
    Preference,
    /// A fact about a person in the user's life.
    Person,
    /// A fact about a project the user is working on.
    Project,
    /// A machine-specific strategy / "note to self" the steward learned.
    Lesson,
    /// An action the steward took on the user's behalf.
    Action,
    /// A generic observation.
    Note,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: u64,
    /// Seconds since the Unix epoch (UTC).
    pub ts: u64,
    pub kind: Kind,
    pub text: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// The tombstone left in place of forgotten content.
pub const TOMBSTONE: &str = "[forgotten]";

/// Append an event to the ground-truth log. Returns the stored event (with its id).
pub fn append(log_path: &Path, kind: Kind, text: &str, tags: Vec<String>) -> Result<Event> {
    let ev = Event {
        id: next_id(log_path)?,
        ts: crate::clock::now_unix(),
        kind,
        text: text.to_string(),
        tags,
    };
    let line = serde_json::to_string(&ev)?;
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .with_context(|| format!("opening {}", log_path.display()))?;
    writeln!(f, "{line}")?;
    Ok(ev)
}

/// Every event, in order.
pub fn all(log_path: &Path) -> Result<Vec<Event>> {
    if !log_path.exists() {
        return Ok(vec![]);
    }
    let f = std::fs::File::open(log_path)?;
    let mut out = vec![];
    for line in BufReader::new(f).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let ev: Event =
            serde_json::from_str(&line).with_context(|| format!("parsing log line: {line}"))?;
        out.push(ev);
    }
    Ok(out)
}

/// Events with id greater than `after_id` (the not-yet-consolidated tail).
pub fn since(log_path: &Path, after_id: u64) -> Result<Vec<Event>> {
    Ok(all(log_path)?
        .into_iter()
        .filter(|e| e.id > after_id)
        .collect())
}

fn next_id(log_path: &Path) -> Result<u64> {
    Ok(all(log_path)?.last().map(|e| e.id + 1).unwrap_or(1))
}

/// Redact every event whose text contains `needle` (case-insensitive): the text is
/// replaced with a tombstone, but the id and timestamp remain, so the log stays a
/// consistent record while the *content* is truly gone. The Brain's git history holds
/// the prior state for audit. Returns how many entries were hit.
pub fn redact(log_path: &Path, needle: &str) -> Result<usize> {
    let needle_l = needle.to_lowercase();
    let mut n = 0;
    let mut buf = String::new();
    for mut e in all(log_path)? {
        if e.text != TOMBSTONE && e.text.to_lowercase().contains(&needle_l) {
            e.text = TOMBSTONE.to_string();
            e.tags = vec!["forgotten".to_string()];
            n += 1;
        }
        buf.push_str(&serde_json::to_string(&e)?);
        buf.push('\n');
    }
    std::fs::write(log_path, buf)?;
    Ok(n)
}

/// The cursor: id of the last event folded into the wiki (incremental compile).
pub fn read_cursor(cursor_path: &Path) -> Result<u64> {
    if !cursor_path.exists() {
        return Ok(0);
    }
    Ok(std::fs::read_to_string(cursor_path)?
        .trim()
        .parse()
        .unwrap_or(0))
}

pub fn write_cursor(cursor_path: &Path, id: u64) -> Result<()> {
    std::fs::write(cursor_path, id.to_string())?;
    Ok(())
}
