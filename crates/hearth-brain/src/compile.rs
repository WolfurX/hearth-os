//! Consolidation — the "sleep" pass that folds new raw activity into the wiki.
//!
//! The [`Compiler`] trait is the point: the *same* pipeline runs whether knowledge is
//! distilled by deterministic local rules (offline) or by a real model behind the
//! neutral router. The intelligence is swappable without touching the Brain's plumbing.

use crate::log::{self, Event};
use crate::schema::Schema;
use crate::wiki::Page;
use crate::Brain;
use anyhow::Result;
use hearth_model::{Completion, Model};
use std::collections::BTreeMap;

/// The outcome of a consolidation pass — surfaced in the evening review.
#[derive(Debug, Default)]
pub struct CompileReport {
    pub events_folded: usize,
    pub pages_touched: Vec<String>,
    pub new_bullets: usize,
    pub compiler: String,
    pub last_id: u64,
}

impl CompileReport {
    pub fn summary(&self) -> String {
        if self.events_folded == 0 {
            return "Nothing new to consolidate — the wiki is up to date.".to_string();
        }
        format!(
            "Consolidated {} new event(s) into {} page(s) (+{} note(s)) via the {} compiler.",
            self.events_folded,
            self.pages_touched.len(),
            self.new_bullets,
            self.compiler
        )
    }
}

/// Fold all raw events newer than the cursor into the wiki, then advance the cursor.
/// Batch and idle-time by design (calm, and cheap enough for a local model).
pub fn consolidate(brain: &Brain, compiler: &dyn Compiler) -> Result<CompileReport> {
    let cursor = log::read_cursor(&brain.cursor_path())?;
    let tail = log::since(&brain.log_path(), cursor)?;
    let last_id = tail.iter().map(|e| e.id).max().unwrap_or(cursor);
    // Consolidate real knowledge only: operational audit (Action events) and forgotten
    // entries stay in the raw log — never folded into the curated wiki.
    let new: Vec<Event> = tail
        .into_iter()
        .filter(|e| e.text != log::TOMBSTONE && e.kind != log::Kind::Action)
        .collect();

    if new.is_empty() {
        log::write_cursor(&brain.cursor_path(), last_id)?;
        return Ok(CompileReport {
            compiler: compiler.name().to_string(),
            last_id,
            ..Default::default()
        });
    }

    // group the compiler's proposed edits by page
    let mut by_page: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for edit in compiler.compile(&brain.schema, &new)? {
        by_page.entry(edit.page).or_default().push(edit.bullet);
    }

    let mut touched = vec![];
    let mut new_bullets = 0;
    for (page_name, bullets) in by_page {
        let mut page = Page::load_or_new(&brain.wiki_dir(), &page_name)?;
        let mut added = 0;
        for b in &bullets {
            if page.add_bullet(b) {
                added += 1;
            }
        }
        page.save()?;
        if added > 0 {
            touched.push(page_name);
            new_bullets += added;
        }
    }

    log::write_cursor(&brain.cursor_path(), last_id)?;
    Ok(CompileReport {
        events_folded: new.len(),
        pages_touched: touched,
        new_bullets,
        compiler: compiler.name().to_string(),
        last_id,
    })
}

/// One unit of change a compiler proposes: a bullet to merge into a page.
pub struct Edit {
    pub page: String,
    pub bullet: String,
}

/// A compilation strategy.
pub trait Compiler {
    fn name(&self) -> &str;
    fn compile(&self, schema: &Schema, events: &[Event]) -> Result<Vec<Edit>>;
}

/// Deterministic, no-model, fully-offline distiller: routes each event into a page
/// via the schema. Honest about its limits — it sorts and dedups; it does not reason
/// or rephrase. This is the graceful-degradation path (and the zero-setup demo path).
pub struct HeuristicCompiler;

impl Compiler for HeuristicCompiler {
    fn name(&self) -> &str {
        "heuristic"
    }
    fn compile(&self, schema: &Schema, events: &[Event]) -> Result<Vec<Edit>> {
        Ok(events
            .iter()
            .map(|ev| {
                let r = schema.route(ev);
                Edit { page: r.page, bullet: r.bullet }
            })
            .collect())
    }
}

/// Model-backed distiller: hands the schema + new events to any [`Model`] and asks
/// for structured page edits. This is the "real" consolidation — semantic merging and
/// rephrasing — and it works with ANY backend behind the neutral router.
pub struct ModelCompiler<'a> {
    pub model: &'a dyn Model,
}

impl Compiler for ModelCompiler<'_> {
    fn name(&self) -> &str {
        "model"
    }
    fn compile(&self, schema: &Schema, events: &[Event]) -> Result<Vec<Edit>> {
        let system = "You are the consolidation pass of Hearth OS's Brain — a legible \
            markdown wiki about the user. Distil the new events into concise, third-person wiki \
            bullets, each filed on the RIGHT page. The pages and what each holds:\n\
            - you: the owner's preferences, traits, and how they like to be helped\n\
            - rhythms: the owner's routines and the timing of their days\n\
            - people/<name>: a person in the owner's life\n\
            - projects/<name>: a project the owner is working on\n\
            - lessons/<name>: a machine-specific strategy the steward learned\n\
            - observations: ONLY if nothing above fits (use sparingly)\n\
            Reply with ONLY a JSON array of objects {\"page\": <page>, \"bullet\": <short \
            third-person fact>}. Merge duplicates; omit ephemera.";
        let mut prompt = String::from("New events:\n");
        for e in events {
            prompt.push_str(&format!("- ({:?}) {}\n", e.kind, e.text));
        }
        prompt.push_str("\nNever record anything matching: ");
        prompt.push_str(&schema.redact.never.join(", "));
        prompt.push_str("\n\nJSON:");

        let raw = self.model.complete(&Completion::new(system, prompt))?;
        parse_edits(&raw)
    }
}

/// Parse a model's reply, tolerating prose or code fences around the JSON array.
fn parse_edits(raw: &str) -> Result<Vec<Edit>> {
    let slice = match (raw.find('['), raw.rfind(']')) {
        (Some(s), Some(e)) if e > s => &raw[s..=e],
        _ => anyhow::bail!("model did not return a JSON array:\n{raw}"),
    };
    #[derive(serde::Deserialize)]
    struct E {
        page: String,
        bullet: String,
    }
    let parsed: Vec<E> = serde_json::from_str(slice)?;
    Ok(parsed
        .into_iter()
        .map(|e| Edit { page: e.page, bullet: e.bullet })
        .collect())
}
