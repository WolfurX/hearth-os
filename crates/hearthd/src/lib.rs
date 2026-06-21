//! # hearthd — the agent runtime ("the mind")
//!
//! The seat of the steward. It runs the core loop — **assemble context → plan → gate →
//! act → audit** — over the Brain (memory), the neutral model router (the swappable mind),
//! and the capability fabric (its hands). The deliberate split: the *reasoning* lives in
//! the model; the *judgement, permissions, and memory* live here, in code you can read.

pub mod capability;
pub mod mcp;
pub mod plan;
pub mod server;
pub mod planner;
pub mod prompt;
pub mod surface;

use anyhow::Result;
use capability::{Decision, Registry};
use hearth_brain::Brain;
use hearth_model::Model;
use hearth_substrate::Substrate;
use mcp::McpHost;
use planner::{HeuristicPlanner, ModelPlanner, Planner};
use prompt::Tier;
use std::path::PathBuf;
use surface::Surface;

pub struct Hearthd {
    pub brain: Brain,
    pub prompt_dir: PathBuf,
    pub tier: Tier,
    pub registry: Registry,
    pub substrate: Substrate,
    pub host: McpHost,
}

impl Hearthd {
    /// Open the steward's world: its memory (the Brain) and its constitution (the prompt).
    pub fn open(tier: Tier) -> Result<Self> {
        let brain_dir = hearth_brain::default_brain_dir()?;
        let brain = Brain::init(&brain_dir)?; // idempotent — ensures the Brain exists
        let prompt_dir = brain
            .root
            .parent()
            .map(|p| p.join("prompt"))
            .unwrap_or_else(|| PathBuf::from("prompt"));
        // The steward's whole home (Brain + workspace + constitution) is one undoable unit:
        // the substrate snapshots it before any mutation. The store lives inside the home but is
        // skipped by the snapshot, so nothing ever nests. `hearthd undo` reverts the last action.
        let home = brain.root.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| brain.root.clone());
        let store = home.join(".substrate");
        let substrate = Substrate::new(home, store);

        // Connect MCP capability servers shipped next to this binary.
        let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf()));
        let server_bin = if cfg!(windows) { "hearth-mcp-fs.exe" } else { "hearth-mcp-fs" };
        let servers: Vec<PathBuf> = exe_dir.map(|d| d.join(server_bin)).into_iter().collect();
        let host = McpHost::connect(servers);

        Ok(Self { brain, prompt_dir, tier, registry: Registry::default(), substrate, host })
    }

    /// The assembled system prompt for the active tier — the glass-box "show me what you
    /// were told."
    pub fn system_prompt(&self) -> Result<String> {
        prompt::assemble(&self.prompt_dir, self.tier, &self.tool_list())
    }

    /// The full tool list — in-process plus the federated MCP tools — for the prompt and
    /// the glass box.
    fn tool_list(&self) -> String {
        let mut s = self.registry.tool_list();
        for (_, t) in self.host.tools() {
            s.push_str(&format!("\n- {}  — {}  (mcp)", t.name, t.description));
        }
        s
    }

    /// Permission for a tool, whether in-process or MCP (deny-by-default).
    fn decide(&self, cap: &str, tool: &str) -> Decision {
        if self.registry.knows(cap, tool) {
            return self.registry.policy(cap, tool);
        }
        let name = format!("{cap}.{tool}");
        if self.host.has(&name) {
            mcp::policy(&name)
        } else {
            Decision::Forbid
        }
    }

    /// Does the tool change state (so the runtime snapshots first)?
    fn mutates(&self, cap: &str, tool: &str) -> bool {
        if self.registry.knows(cap, tool) {
            return self.registry.is_mutating(cap, tool);
        }
        let name = format!("{cap}.{tool}");
        self.host.has(&name) && mcp::mutating(&name)
    }

    /// Run a tool — in-process if we serve it, otherwise over MCP.
    fn execute_tool(&self, cap: &str, tool: &str, args: &serde_json::Value) -> Result<String> {
        if self.registry.knows(cap, tool) {
            self.registry.execute(&self.brain, cap, tool, args)
        } else {
            self.host.call(&format!("{cap}.{tool}"), args)
        }
    }

    /// Is `full` (a dotted `cap.tool` id) a real tool — in-process or MCP?
    fn tool_exists(&self, full: &str) -> bool {
        self.host.has(full)
            || full.rsplit_once('.').map_or(false, |(c, t)| self.registry.knows(c, t))
    }

    /// Normalise the model's `(capability, tool)` to a real tool. Models often mis-split a
    /// dotted name — e.g. emitting `cap="fs.read", tool="read"` for the tool `fs.read` — so we
    /// try the obvious recombinations and, on a match, return the canonical split (everything
    /// before the last dot is the capability). A truly-unknown tool passes through unchanged,
    /// so the gate's deny-by-default still refuses it.
    fn resolve(&self, cap: &str, tool: &str) -> (String, String) {
        for full in [format!("{cap}.{tool}"), cap.to_string(), tool.to_string()] {
            if self.tool_exists(&full) {
                if let Some((c, t)) = full.rsplit_once('.') {
                    return (c.to_string(), t.to_string());
                }
            }
        }
        (cap.to_string(), tool.to_string())
    }

    /// One turn of the loop, as structured data — the single source of truth used by the
    /// CLI (`run`). Plans, gates, acts, audits. A thin wrapper over [`Self::turn_streaming`].
    pub fn turn(&self, intent: &str, approve: bool) -> Result<TurnResult> {
        self.turn_streaming(intent, approve, |_: &StreamEvent| {})
    }

    /// One turn, emitting [`StreamEvent`]s as it happens — the live tool-trail behind the
    /// streaming server (`/api/intent`). `emit` is called in order: `recalled` → `plan` →
    /// one `step` per action → `done`. The buffered [`Self::turn`] passes a no-op sink, so
    /// there is exactly one copy of the loop.
    pub fn turn_streaming(
        &self,
        intent: &str,
        approve: bool,
        mut emit: impl FnMut(&StreamEvent),
    ) -> Result<TurnResult> {
        // context: the constitution + what we already know about the owner
        let system = self.system_prompt()?;
        let mem = hearth_brain::recall::recall(&self.brain.wiki_dir(), intent, 2)?;
        let recalled: Vec<String> = mem.iter().map(|h| h.page.clone()).collect();
        let known = if mem.is_empty() {
            "- (nothing relevant yet)".to_string()
        } else {
            mem.iter().map(|h| format!("- {}", h.snippet)).collect::<Vec<_>>().join("\n")
        };
        emit(&StreamEvent::Recalled { recalled: &recalled });
        let full_system =
            format!("{system}\n\nWhat you already know about the owner (from memory):\n{known}");

        // plan → act → observe → plan again: a bounded agentic loop. With a model the steward
        // chains steps, *seeing each result before deciding the next*; the heuristic floor stays
        // one-shot. The loop ends when the steward gives its final answer (respond.say), needs
        // the owner's approval, has nothing left to do, or hits the step ceiling.
        let model = hearth_model::HttpModel::from_env().ok();
        let max_iters = if model.is_some() { 5 } else { 1 };
        let mut planner = "heuristic".to_string();
        let mut summary = String::new();
        let mut steps: Vec<StepResult> = vec![];
        let mut transcript = String::new();
        let mut paused = false;

        for iter in 0..max_iters {
            // reason over the intent plus what's already happened this turn
            let ask = if transcript.is_empty() {
                intent.to_string()
            } else {
                format!(
                    "{intent}\n\nWhat you've already done this turn (results included):\n{transcript}\n\
                     Plan the next step(s). Do NOT repeat a tool call that already appears above — its \
                     result is there to use. Use respond.say only for your final answer to the owner; \
                     return an empty steps array if nothing more is needed."
                )
            };
            let plan = match &model {
                Some(m) => match (ModelPlanner { model: m }).plan(&ask, &full_system) {
                    Ok(p) => {
                        planner = m.id().to_string();
                        p
                    }
                    Err(e) => {
                        eprintln!("· model planner failed ({e}); falling back to the heuristic");
                        planner = "heuristic".to_string();
                        HeuristicPlanner.plan(intent, &full_system)?
                    }
                },
                None => HeuristicPlanner.plan(intent, &full_system)?,
            };
            if iter == 0 {
                summary = plan.summary.clone();
                emit(&StreamEvent::Plan { planner: &planner, summary: &summary });
            }
            if plan.steps.is_empty() {
                break;
            }

            // act — gated, snapshotted, audited; each completed step streams out at once
            let mut terminal = false;
            for st in &plan.steps {
                let (rcap, rtool) = self.resolve(&st.capability, &st.tool);
                let (cap, tool, args) = (rcap.as_str(), rtool.as_str(), &st.args);
                let decision = self.decide(cap, tool);
                let mut sr = StepResult {
                    capability: cap.to_string(),
                    tool: tool.to_string(),
                    why: st.why.clone(),
                    decision: format!("{decision:?}").to_lowercase(),
                    ran: false,
                    snapshot: None,
                    result: None,
                };
                let run_it = !matches!(decision, Decision::Forbid)
                    && !(matches!(decision, Decision::Ask) && !approve);
                if run_it {
                    if self.mutates(cap, tool) {
                        let (txid, r) = self
                            .substrate
                            .transact(&format!("{cap}.{tool}"), || self.execute_tool(cap, tool, args))?;
                        sr.snapshot = Some(txid);
                        sr.result = Some(r);
                    } else {
                        sr.result = Some(self.execute_tool(cap, tool, args)?);
                    }
                    sr.ran = true;
                    let _ = hearth_brain::log::append(
                        &self.brain.log_path(),
                        hearth_brain::log::Kind::Action,
                        &format!("hearthd ran {cap}.{tool} {args}"),
                        vec!["hearthd".into()],
                    );
                    transcript.push_str(&format!("- {cap}.{tool} → {}\n", sr.result.as_deref().unwrap_or("")));
                    if cap == "respond" && tool == "say" {
                        terminal = true; // the steward has given its final answer
                    }
                } else if matches!(decision, Decision::Ask) {
                    paused = true; // held for the owner's ok — the loop can't continue past this
                }
                emit(&StreamEvent::Step { step: &sr });
                steps.push(sr);
                if paused {
                    break;
                }
            }
            if terminal || paused {
                break;
            }
        }

        // manifestation out — a bespoke surface composed (by the model, from the DSL) for this intent
        let context = format!(
            "recalled: {}\nsummary: {}\nwhat happened:\n{}",
            if recalled.is_empty() { "(nothing)".to_string() } else { recalled.join(", ") },
            summary,
            if transcript.trim().is_empty() { "(nothing executed)".to_string() } else { transcript.clone() },
        );
        let surface = self.manifest(model.as_ref(), intent, &context);
        if let Some(s) = &surface {
            emit(&StreamEvent::Surface { surface: s });
        }

        // the Brain evolves on its own: fold any new activity from this turn into the legible
        // wiki (offline heuristic — free + deterministic), so what the steward learned this turn
        // becomes recallable next turn. The underlying raw events were already snapshotted.
        if let Ok(report) =
            hearth_brain::compile::consolidate(&self.brain, &hearth_brain::compile::HeuristicCompiler)
        {
            if report.events_folded > 0 {
                let _ = hearth_brain::gitstore::commit_all(
                    &self.brain.root,
                    &format!("brain: {}", report.summary()),
                );
            }
        }

        let result = TurnResult { recalled, planner, summary, steps };
        emit(&StreamEvent::Done { result: &result });
        Ok(result)
    }

    /// Compose the generated surface for an intent — "intent in, manifestation out". With a
    /// model wired, the steward composes a **bespoke** surface from the DSL (`surface::compose`);
    /// the model returns an empty surface when none would help. Without a model, or if compose
    /// fails, the floor manifests the canonical reference surface only when explicitly asked.
    fn manifest(
        &self,
        model: Option<&hearth_model::HttpModel>,
        intent: &str,
        context: &str,
    ) -> Option<Surface> {
        if let Some(m) = model {
            match surface::compose(m, intent, context) {
                Ok(s) if !s.nodes.is_empty() => return Some(s),
                Ok(_) => return None, // the model judged that no surface would help here
                Err(e) => eprintln!("· surface compose failed ({e}); falling back to the floor"),
            }
        }
        if intent.to_lowercase().contains("surface") {
            Some(Surface::reference())
        } else {
            None
        }
    }

    /// One turn, printed for the CLI (the glass box, on the terminal).
    pub fn run(&self, intent: &str, approve: bool) -> Result<()> {
        let r = self.turn(intent, approve)?;
        if !r.recalled.is_empty() {
            println!("· recalled from memory: {}", r.recalled.join(", "));
        }
        if r.planner != "heuristic" {
            println!("· planning with {}", r.planner);
        }
        println!("  plan · {}", r.summary);
        for (i, s) in r.steps.iter().enumerate() {
            let why = if s.why.is_empty() { "(no reason given)" } else { s.why.as_str() };
            println!("   {}. {}.{} — {}", i + 1, s.capability, s.tool, why);
            if !s.ran {
                if s.decision == "forbid" {
                    println!("   ✗ {}.{} is not permitted — skipped", s.capability, s.tool);
                } else {
                    println!("   ⏸ {}.{} needs your approval — re-run with --yes to proceed", s.capability, s.tool);
                }
            } else {
                if let Some(tx) = s.snapshot {
                    println!("   ✓ snapshot tx-{tx} taken first — `hearthd undo` reverts this");
                }
                if let Some(res) = &s.result {
                    println!("   → {res}");
                }
            }
        }
        Ok(())
    }

    /// The Brain's curated pages, for the UI's "what do you know about me?" view.
    pub fn brain_pages(&self) -> Result<Vec<BrainPage>> {
        let mut out = vec![];
        for name in hearth_brain::wiki::list(&self.brain.wiki_dir())? {
            let path = hearth_brain::wiki::Page::path_for(&self.brain.wiki_dir(), &name);
            let text = std::fs::read_to_string(&path).unwrap_or_default();
            let bullets: Vec<String> = text
                .lines()
                .filter_map(|l| l.trim().strip_prefix("- ").map(|s| s.to_string()))
                .collect();
            out.push(BrainPage { name, bullets });
        }
        Ok(out)
    }

    /// Forget a curated page — the glass box, made reversible. Snapshots the Brain first
    /// (so `hearthd undo` reverts it), removes the page, and redacts the raw-log entries
    /// that fed it. The wiki's own git history still retains the prior state for audit.
    pub fn forget(&self, page: &str) -> Result<ForgetResult> {
        let page = page.to_string();
        let wiki_dir = self.brain.wiki_dir();
        let log_path = self.brain.log_path();
        let root = self.brain.root.clone();
        let (snapshot, (removed, redacted)) =
            self.substrate.transact(&format!("forget {page}"), || {
                let removed = hearth_brain::wiki::remove(&wiki_dir, &page)?;
                let subject = page.rsplit('/').next().unwrap_or(&page).replace('-', " ");
                let redacted = hearth_brain::log::redact(&log_path, &subject)?;
                hearth_brain::gitstore::commit_all(&root, &format!("brain: forget {page}"))?;
                Ok((removed, redacted))
            })?;
        Ok(ForgetResult { page, removed, redacted, snapshot })
    }

    /// Record a structured edit the owner made to a live surface — the bidirectional half of
    /// generative surfaces (VISION §2.4). The edit becomes ground truth in the Brain (so the
    /// steward can learn from it), through the same **privacy floor** as everything else: a
    /// value that looks like a secret is refused, never written. v0 floor: we record + audit;
    /// a model will later *interpret* the edit ("you removed 4 photos") from this same signal.
    pub fn surface_event(&self, node: &str, kind: &str, value: &str) -> Result<SurfaceEventResult> {
        if let Some(term) = self.brain.schema.forbidden_term(value) {
            return Ok(SurfaceEventResult {
                recorded: None,
                refused: true,
                note: format!("refused — that looks like a secret (matched \"{term}\")"),
            });
        }
        let ev = hearth_brain::log::append(
            &self.brain.log_path(),
            hearth_brain::log::Kind::Note,
            &format!("surface {kind} · {node}: {value}"),
            vec!["surface".into()],
        )?;
        Ok(SurfaceEventResult { recorded: Some(ev.id), refused: false, note: String::new() })
    }
}

/// The result of one turn — what the steward recalled, planned, and did.
#[derive(serde::Serialize)]
pub struct TurnResult {
    pub recalled: Vec<String>,
    pub planner: String,
    pub summary: String,
    pub steps: Vec<StepResult>,
}

#[derive(serde::Serialize)]
pub struct StepResult {
    pub capability: String,
    pub tool: String,
    pub why: String,
    pub decision: String,
    pub ran: bool,
    pub snapshot: Option<u64>,
    pub result: Option<String>,
}

#[derive(serde::Serialize)]
pub struct BrainPage {
    pub name: String,
    pub bullets: Vec<String>,
}

/// The outcome of forgetting a page: whether it existed, how many raw entries were
/// redacted, and the snapshot id that makes it undoable.
#[derive(serde::Serialize)]
pub struct ForgetResult {
    pub page: String,
    pub removed: bool,
    pub redacted: usize,
    pub snapshot: u64,
}

/// The outcome of a surface edit: the raw-log id it became, or a privacy-floor refusal.
#[derive(serde::Serialize)]
pub struct SurfaceEventResult {
    pub recorded: Option<u64>,
    pub refused: bool,
    pub note: String,
}

/// A turn's progress, streamed live so the shell can show the tool-trail as it happens.
/// Internally tagged (`{"event":"step", ...}`) so the UI can dispatch on one field.
#[derive(serde::Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum StreamEvent<'a> {
    /// What the steward pulled from memory for this intent.
    Recalled { recalled: &'a [String] },
    /// The plan headline, as soon as it's decided (before acting).
    Plan { planner: &'a str, summary: &'a str },
    /// One action, just completed (gated, maybe snapshotted, maybe run).
    Step { step: &'a StepResult },
    /// The manifestation — a generated surface the shell renders natively.
    Surface { surface: &'a Surface },
    /// The turn is finished; carries the full result for good measure.
    Done { result: &'a TurnResult },
}
