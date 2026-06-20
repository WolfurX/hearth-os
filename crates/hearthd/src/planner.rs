//! Turning an intent into a [`Plan`]. The [`Planner`] trait is the seam that keeps the
//! *judgement* (which tools, gated how) in our code while the *reasoning* lives in a
//! swappable model — so a model swap never moves the safety logic.

use crate::plan::{Plan, Step};
use anyhow::Result;
use hearth_model::{Completion, Model};
use serde_json::json;

/// Does a token look like a file path or a `name.ext`? (for the offline planner's
/// best-effort path extraction — a real model would do this properly).
fn looks_like_path(t: &str) -> bool {
    t.contains('/')
        || t.contains('\\')
        || (t.contains('.')
            && !t.ends_with('.')
            && t.rsplit('.').next().map_or(false, |e| !e.is_empty() && e.len() <= 5 && e.chars().all(|c| c.is_alphanumeric())))
}

fn extract_path(s: &str) -> Option<String> {
    s.split_whitespace()
        .map(|t| t.trim_matches(|c: char| c == '"' || c == '\'' || c == ',' || c == '?'))
        .find(|t| looks_like_path(t))
        .map(|t| t.to_string())
}

pub trait Planner {
    fn plan(&self, intent: &str, system: &str) -> Result<Plan>;
}

/// Deterministic, no-model planner — the offline / weak-model floor. It never improvises
/// a side effect: it maps an intent to exactly one safe step.
pub struct HeuristicPlanner;

impl Planner for HeuristicPlanner {
    fn plan(&self, intent: &str, _system: &str) -> Result<Plan> {
        let v = intent.to_lowercase();
        let step = if v.contains("remember")
            || v.contains("i like")
            || v.contains("i prefer")
            || v.starts_with("note ")
        {
            Step {
                capability: "brain".into(),
                tool: "remember".into(),
                args: json!({ "text": intent.trim() }),
                why: "the owner stated something to keep".into(),
            }
        } else if v.contains("know about me")
            || v.contains("recall")
            || v.contains("what do you know")
            || v.contains("do you know")
        {
            Step {
                capability: "brain".into(),
                tool: "recall".into(),
                args: json!({ "query": intent.trim() }),
                why: "the owner asked what is known".into(),
            }
        } else if (v.contains("read ") || v.contains("open the file")) && extract_path(intent).is_some() {
            Step {
                capability: "fs".into(),
                tool: "read".into(),
                args: json!({ "path": extract_path(intent).unwrap() }),
                why: "the owner asked to read a file".into(),
            }
        } else if v.contains("list") || v.contains("files") || v.contains("folder") || v.contains("directory") || v.contains("what's here") {
            Step {
                capability: "fs".into(),
                tool: "list".into(),
                args: json!({ "path": extract_path(intent).unwrap_or_else(|| ".".into()) }),
                why: "the owner asked what's in a folder".into(),
            }
        } else {
            Step {
                capability: "respond".into(),
                tool: "say".into(),
                args: json!({ "text": format!("I can note things, recall what I know, or answer plainly. You said: {}", intent.trim()) }),
                why: "no capability fits — answer plainly".into(),
            }
        };
        Ok(Plan {
            summary: "Handle the owner's request.".into(),
            steps: vec![step],
        })
    }
}

/// Model-backed planner — uses the assembled constitution as the system prompt and asks
/// any model (via the neutral router) for a structured JSON plan. Used when a real model
/// is configured; otherwise `hearthd` degrades to the heuristic planner.
pub struct ModelPlanner<'a> {
    pub model: &'a dyn Model,
}

impl Planner for ModelPlanner<'_> {
    fn plan(&self, intent: &str, system: &str) -> Result<Plan> {
        let raw = self
            .model
            .complete(&Completion::new(system, format!("Owner says: {intent}\n\nJSON plan:")))?;
        let slice = match (raw.find('{'), raw.rfind('}')) {
            (Some(a), Some(b)) if b > a => &raw[a..=b],
            _ => anyhow::bail!("model did not return a JSON object:\n{raw}"),
        };
        Ok(serde_json::from_str(slice)?)
    }
}
