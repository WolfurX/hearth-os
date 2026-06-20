//! # hearthd — the agent runtime ("the mind")
//!
//! The seat of the steward. It runs the core loop — **assemble context → plan → gate →
//! act → audit** — over the Brain (memory), the neutral model router (the swappable mind),
//! and the capability fabric (its hands). The deliberate split: the *reasoning* lives in
//! the model; the *judgement, permissions, and memory* live here, in code you can read.

pub mod capability;
pub mod plan;
pub mod planner;
pub mod prompt;

use anyhow::Result;
use capability::{Decision, Registry};
use hearth_brain::Brain;
use planner::{HeuristicPlanner, Planner};
use prompt::Tier;
use std::path::PathBuf;

pub struct Hearthd {
    pub brain: Brain,
    pub prompt_dir: PathBuf,
    pub tier: Tier,
    pub registry: Registry,
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
        Ok(Self { brain, prompt_dir, tier, registry: Registry::default() })
    }

    /// The assembled system prompt for the active tier — the glass-box "show me what you
    /// were told."
    pub fn system_prompt(&self) -> Result<String> {
        prompt::assemble(&self.prompt_dir, self.tier, &self.registry.tool_list())
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

        // 2 · plan — offline heuristic floor; a model planner slots in behind the same trait
        let plan = HeuristicPlanner.plan(intent, &full_system)?;

        // 3 · glass box — show the plan before doing anything
        print!("{}", plan.render_plain());

        // 4 · act — gated and audited
        for st in &plan.steps {
            match self.registry.policy(&st.capability, &st.tool) {
                Decision::Forbid => {
                    println!("   ✗ {}.{} is not permitted — skipped", st.capability, st.tool)
                }
                Decision::Ask if !approve => println!(
                    "   ⏸ {}.{} needs your approval — re-run with --yes to proceed",
                    st.capability, st.tool
                ),
                _ => {
                    // (on a real install, a btrfs snapshot is taken here first)
                    let res = self.registry.execute(&self.brain, &st.capability, &st.tool, &st.args)?;
                    println!("   → {res}");
                    let _ = hearth_brain::log::append(
                        &self.brain.log_path(),
                        hearth_brain::log::Kind::Action,
                        &format!("hearthd ran {}.{} {}", st.capability, st.tool, st.args),
                        vec!["hearthd".into()],
                    );
                }
            }
        }
        Ok(())
    }
}
