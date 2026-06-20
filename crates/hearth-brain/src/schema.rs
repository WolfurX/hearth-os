//! The schema — procedural memory: the rules by which the steward turns raw activity
//! into curated, legible knowledge. Editing `schema.toml` changes *how the OS learns*.

use crate::log::{Event, Kind};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

/// Written on `init`. Plain, commented TOML the user is meant to read and edit.
pub const DEFAULT_SCHEMA_TOML: &str = r#"# The Brain's schema — procedural memory: the rules by which the steward turns raw
# activity into curated, legible knowledge. This file is yours to edit; editing it
# changes *how the OS learns*.

# Keyword routing for free-form observations (kind = interaction / note / action).
# Explicitly-kinded events (preference / person / project / lesson) route by kind.
[promote]
preferences = ["prefer", "like", "love", "dislike", "hate", "always", "never", "concise", "verbose", "want"]
rhythms     = ["morning", "evening", "night", "daily", "weekly", "every", "routine", "schedule", "usually"]
lessons     = ["lesson", "after", "rebuild", "route", "failed", "instead", "remember to", "when "]

# The privacy floor: anything matching these is NEVER written to the log or the wiki.
# The steward refuses to remember it. (Secrets belong in the vault, not in memory.)
[redact]
never = ["password", "passwd", "secret", "api key", "apikey", "token", "ssn", "social security", "credit card", "private key", "seed phrase"]

# Retention. 0 = keep everything (no auto-retire yet).
[retire]
stale_days = 0
"#;

#[derive(Debug, Clone, Deserialize)]
pub struct Schema {
    #[serde(default)]
    pub promote: Promote,
    #[serde(default)]
    pub redact: Redact,
    #[serde(default)]
    pub retire: Retire,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Promote {
    #[serde(default)]
    pub preferences: Vec<String>,
    #[serde(default)]
    pub rhythms: Vec<String>,
    #[serde(default)]
    pub lessons: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Redact {
    #[serde(default)]
    pub never: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Retire {
    #[serde(default)]
    pub stale_days: u32,
}

impl Schema {
    pub fn load(path: &Path) -> Result<Self> {
        let s = std::fs::read_to_string(path)
            .with_context(|| format!("reading schema {}", path.display()))?;
        toml::from_str(&s).context("parsing schema.toml")
    }

    /// The privacy floor. Returns the matched term if the text must never be stored.
    pub fn forbidden_term(&self, text: &str) -> Option<String> {
        let t = text.to_lowercase();
        self.redact
            .never
            .iter()
            .find(|term| t.contains(&term.to_lowercase()))
            .cloned()
    }

    /// Where a raw event belongs in the wiki, and the bullet to record there.
    pub fn route(&self, ev: &Event) -> Route {
        let text = ev.text.trim().to_string();
        let subject = ev.tags.first().map(|s| s.as_str());
        match ev.kind {
            Kind::Preference => Route::new("you", text),
            Kind::Person => Route::new(&format!("people/{}", slug(subject.unwrap_or("misc"))), text),
            Kind::Project => {
                Route::new(&format!("projects/{}", slug(subject.unwrap_or("misc"))), text)
            }
            Kind::Lesson => {
                Route::new(&format!("lessons/{}", slug(subject.unwrap_or("general"))), text)
            }
            Kind::Interaction | Kind::Note | Kind::Action => {
                let low = text.to_lowercase();
                if any_match(&self.promote.rhythms, &low) {
                    Route::new("rhythms", text)
                } else if any_match(&self.promote.lessons, &low) {
                    Route::new("lessons/general", text)
                } else if any_match(&self.promote.preferences, &low) {
                    Route::new("you", text)
                } else {
                    Route::new("observations", text)
                }
            }
        }
    }
}

/// A routing decision: which wiki page, and the bullet to merge into it.
pub struct Route {
    pub page: String,
    pub bullet: String,
}

impl Route {
    fn new(page: &str, bullet: String) -> Self {
        Self { page: page.to_string(), bullet }
    }
}

fn any_match(keys: &[String], low_text: &str) -> bool {
    keys.iter().any(|k| low_text.contains(&k.to_lowercase()))
}

/// Turn an arbitrary subject into a safe, lowercase, dash-separated page slug.
pub fn slug(s: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for ch in s.trim().to_lowercase().chars() {
        if ch.is_alphanumeric() {
            out.push(ch);
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    let out = out.trim_end_matches('-').to_string();
    if out.is_empty() {
        "misc".to_string()
    } else {
        out
    }
}
