//! # hearthd — the agent runtime ("the mind")
//!
//! The seat of the steward. It runs the core loop — **assemble context → plan → gate →
//! act → audit** — over the Brain (memory), the neutral model router (the swappable mind),
//! and the capability fabric (its hands). The deliberate split: the *reasoning* lives in
//! the model; the *judgement, permissions, and memory* live here, in code you can read.

pub mod capability;
pub mod mcp;
pub mod plan;
pub mod planner;
pub mod prompt;

use anyhow::Result;
use capability::{Decision, Registry};
use hearth_brain::Brain;
use hearth_model::Model;
use hearth_substrate::Substrate;
use mcp::McpHost;
use planner::{HeuristicPlanner, ModelPlanner, Planner};
use prompt::Tier;
use std::path::PathBuf;

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
        let store = brain
            .root
            .parent()
            .map(|p| p.join(".substrate"))
            .unwrap_or_else(|| PathBuf::from(".substrate"));
        let substrate = Substrate::new(brain.root.clone(), store);

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

    /// One turn of the loop.
    pub fn run(&self, intent: &str, approve: bool) -> Result<()> {
        // 1 · context assembly — the constitution + what we already know about the owner
        let system = self.system_prompt()?;
        let mem = hearth_brain::recall::recall(&self.brain.wiki_dir(), intent, 2)?;
        if !mem.is_empty() {
            println!(
                "· recalled from memory: {}",
                mem.iter().map(|h| h.page.clone()).collect::<Vec<_>>().join(", ")
            );
        }
        let known = if mem.is_empty() {
            "- (nothing relevant yet)".to_string()
        } else {
            mem.iter().map(|h| format!("- {}", h.snippet)).collect::<Vec<_>>().join("\n")
        };
        let full_system =
            format!("{system}\n\nWhat you already know about the owner (from memory):\n{known}");

        // 2 · plan — a real model if one is configured (HEARTH_MODEL_*), else the heuristic floor
        let model = hearth_model::HttpModel::from_env().ok();
        let plan = match &model {
            Some(m) => {
                println!("· planning with {}", m.id());
                match (ModelPlanner { model: m }).plan(intent, &full_system) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("· model planner failed ({e}); falling back to the heuristic");
                        HeuristicPlanner.plan(intent, &full_system)?
                    }
                }
            }
            None => HeuristicPlanner.plan(intent, &full_system)?,
        };

        // 3 · glass box — show the plan before doing anything
        print!("{}", plan.render_plain());

        // 4 · act — gated and audited
        for st in &plan.steps {
            let (cap, tool, args) = (st.capability.as_str(), st.tool.as_str(), &st.args);
            match self.decide(cap, tool) {
                Decision::Forbid => println!("   ✗ {cap}.{tool} is not permitted — skipped"),
                Decision::Ask if !approve => {
                    println!("   ⏸ {cap}.{tool} needs your approval — re-run with --yes to proceed")
                }
                _ => {
                    // mutating actions snapshot first, so the gate's "approve" is reversible
                    let res = if self.mutates(cap, tool) {
                        let (txid, r) = self
                            .substrate
                            .transact(&format!("{cap}.{tool}"), || self.execute_tool(cap, tool, args))?;
                        println!("   ✓ snapshot tx-{txid} taken first — `hearthd undo` reverts this");
                        r
                    } else {
                        self.execute_tool(cap, tool, args)?
                    };
                    println!("   → {res}");
                    let _ = hearth_brain::log::append(
                        &self.brain.log_path(),
                        hearth_brain::log::Kind::Action,
                        &format!("hearthd ran {cap}.{tool} {args}"),
                        vec!["hearthd".into()],
                    );
                }
            }
        }
        Ok(())
    }
}
