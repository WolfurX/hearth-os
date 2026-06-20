//! `hearth-brain` — the CLI for the Brain.
//!
//! In the full system these are the `brain` capability's tools (recall / remember /
//! edit / forget / compile), driven by `hearthd`. As a standalone binary they let you
//! converse with the Brain directly and *read what it has learned about you* — the
//! Phase 1 promise, made real.

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use hearth_brain::log::Kind;
use hearth_brain::{compile, default_brain_dir, gitstore, log, recall, wiki, Brain};

/// The Brain — Hearth OS's legible, model-portable long-term memory.
#[derive(Parser)]
#[command(name = "hearth-brain", version, about)]
struct Cli {
    /// Brain data directory (default: $HEARTH_HOME/brain or ~/.hearth/brain).
    #[arg(long, global = true)]
    brain: Option<PathBuf>,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Create a new Brain (idempotent): folders, default schema, seed pages, git.
    Init,
    /// Append a raw event to the ground-truth activity log.
    Log {
        /// What happened (free text).
        text: Vec<String>,
        /// Event kind.
        #[arg(long, value_enum, default_value = "note")]
        kind: Kind,
        /// Subject tag(s) — e.g. a person/project name. The first names the page.
        #[arg(long = "tag")]
        tags: Vec<String>,
    },
    /// Shortcut: record an explicit preference about how you like to be helped.
    Remember {
        /// The preference (free text).
        text: Vec<String>,
    },
    /// Run the consolidation pass: fold new raw events into the wiki ("sleep").
    Compile,
    /// Show the wiki — "what do you know about me?" (optionally a single page).
    Whoami {
        /// Limit to one page, e.g. `you` or `people/alice`.
        page: Option<String>,
    },
    /// Select the wiki pages most relevant to a query (context-assembly preview).
    Recall {
        /// What to look up.
        query: Vec<String>,
        #[arg(long, default_value = "3")]
        k: usize,
    },
    /// Print the raw activity log (the immutable ground truth).
    Raw,
    /// Forget a page: delete it AND redact matching entries from the raw log.
    Forget {
        /// Page name, e.g. `projects/hearth-os` or `observations`.
        page: String,
    },
    /// Show counts, the brain location, and what's pending consolidation.
    Status,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let dir = match &cli.brain {
        Some(p) => p.clone(),
        None => default_brain_dir()?,
    };

    match cli.cmd {
        Cmd::Init => {
            let brain = Brain::init(&dir)?;
            println!("Brain ready at {}", brain.root.display());
            println!("  raw log : {}", brain.log_path().display());
            println!("  wiki    : {}", brain.wiki_dir().display());
            println!("  schema  : {}", brain.schema_path().display());
            println!("\nTry:  hearth-brain remember \"I prefer concise replies\"");
            println!("      hearth-brain compile && hearth-brain whoami");
        }
        Cmd::Log { text, kind, tags } => {
            let brain = Brain::open(&dir)?;
            append_guarded(&brain, kind, &text.join(" "), tags)?;
        }
        Cmd::Remember { text } => {
            let brain = Brain::open(&dir)?;
            append_guarded(&brain, Kind::Preference, &text.join(" "), vec![])?;
        }
        Cmd::Compile => {
            let brain = Brain::open(&dir)?;
            let report = run_compile(&brain)?;
            gitstore::commit_all(&brain.root, &format!("brain: {}", report.summary()))?;
            println!("{}", report.summary());
            if !report.pages_touched.is_empty() {
                println!("  pages: {}", report.pages_touched.join(", "));
            }
        }
        Cmd::Whoami { page } => {
            let brain = Brain::open(&dir)?;
            show_wiki(&brain, page.as_deref())?;
        }
        Cmd::Recall { query, k } => {
            let brain = Brain::open(&dir)?;
            let q = query.join(" ");
            let hits = recall::recall(&brain.wiki_dir(), &q, k)?;
            if hits.is_empty() {
                println!("(nothing relevant in memory for \"{q}\")");
            } else {
                println!("Relevant memory for \"{q}\":");
                for h in hits {
                    println!("  • {} (score {})", h.page, h.score);
                    if !h.snippet.is_empty() {
                        println!("      {}", h.snippet);
                    }
                }
            }
        }
        Cmd::Raw => {
            let brain = Brain::open(&dir)?;
            for e in log::all(&brain.log_path())? {
                let tags = if e.tags.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", e.tags.join(","))
                };
                println!(
                    "#{:<3} {}  [{:?}]{}  {}",
                    e.id,
                    hearth_brain::clock::format_utc(e.ts),
                    e.kind,
                    tags,
                    e.text
                );
            }
        }
        Cmd::Forget { page } => {
            let brain = Brain::open(&dir)?;
            forget(&brain, &page)?;
        }
        Cmd::Status => {
            let brain = Brain::open(&dir)?;
            let events = log::all(&brain.log_path())?;
            let cursor = log::read_cursor(&brain.cursor_path())?;
            let pending = events.iter().filter(|e| e.id > cursor).count();
            let pages = wiki::list(&brain.wiki_dir())?;
            println!("Brain at {}", brain.root.display());
            println!("  raw events   : {}", events.len());
            println!("  wiki pages   : {}", pages.len());
            println!("  consolidated : through #{cursor}");
            println!("  pending      : {pending} event(s) awaiting `compile`");
            println!("  model        : neutral (offline heuristic unless a backend is configured)");
        }
    }
    Ok(())
}

/// Enforce the privacy floor before anything enters the log.
fn append_guarded(brain: &Brain, kind: Kind, text: &str, tags: Vec<String>) -> Result<()> {
    if text.trim().is_empty() {
        anyhow::bail!("nothing to record");
    }
    if let Some(term) = brain.schema.forbidden_term(text) {
        println!("Refused: that looks like a secret (matched \"{term}\").");
        println!("The Brain never stores secrets — those belong in the vault, not in memory.");
        return Ok(());
    }
    let ev = log::append(&brain.log_path(), kind, text, tags)?;
    gitstore::commit_all(&brain.root, &format!("brain: log #{} ({:?})", ev.id, ev.kind))?;
    println!(
        "Logged #{} [{:?}]. Run `hearth-brain compile` to fold it into the wiki.",
        ev.id, ev.kind
    );
    Ok(())
}

/// Neutral by construction: use a real model if one is configured, else degrade to
/// the offline heuristic compiler. Same pipeline either way.
fn run_compile(brain: &Brain) -> Result<compile::CompileReport> {
    #[cfg(feature = "online")]
    {
        if std::env::var("HEARTH_MODEL_URL").is_ok() {
            match hearth_model::http::OpenAiCompatModel::from_env() {
                Ok(model) => {
                    return compile::consolidate(brain, &compile::ModelCompiler { model: &model });
                }
                Err(e) => eprintln!("(model backend unavailable: {e}; using heuristic)"),
            }
        }
    }
    compile::consolidate(brain, &compile::HeuristicCompiler)
}

fn show_wiki(brain: &Brain, only: Option<&str>) -> Result<()> {
    let names = match only {
        Some(p) => vec![p.to_string()],
        None => wiki::list(&brain.wiki_dir())?,
    };
    if names.is_empty() {
        println!("The wiki is empty. Log something and run `hearth-brain compile`.");
        return Ok(());
    }
    for (i, name) in names.iter().enumerate() {
        let path = wiki::Page::path_for(&brain.wiki_dir(), name);
        if !path.exists() {
            println!("(no page \"{name}\")");
            continue;
        }
        if i > 0 {
            println!();
        }
        let text = std::fs::read_to_string(&path)?;
        print!("{text}");
        if !text.ends_with('\n') {
            println!();
        }
    }
    Ok(())
}

fn forget(brain: &Brain, page: &str) -> Result<()> {
    let removed = wiki::remove(&brain.wiki_dir(), page)?;
    // Redact raw-log entries that fed this page, matched on the page's subject token.
    let subject = page.rsplit('/').next().unwrap_or(page).replace('-', " ");
    let redacted = log::redact(&brain.log_path(), &subject)?;
    gitstore::commit_all(&brain.root, &format!("brain: forget {page}"))?;
    if removed {
        println!("Forgot page \"{page}\".");
    } else {
        println!("No page \"{page}\" (nothing removed).");
    }
    println!(
        "Redacted {redacted} matching raw-log entr(ies). Git history retains the prior state for audit."
    );
    Ok(())
}
