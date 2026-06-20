//! `hearthd` CLI — drive the steward's loop, and inspect the system prompt it assembles.

use anyhow::Result;
use clap::{Parser, Subcommand};
use hearthd::prompt::{self, Tier};
use hearthd::Hearthd;

#[derive(Parser)]
#[command(name = "hearthd", version, about = "The Hearth agent runtime — the mind.")]
struct Cli {
    /// The chosen model's capability tier — sets how idiot-proof the prompt is.
    #[arg(long, value_enum, default_value = "medium", global = true)]
    tier: Tier,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Create the steward's world (memory + the default constitution at ~/.hearth/prompt).
    Init,
    /// Show the assembled system prompt for the current tier (the glass box).
    Prompt,
    /// Run one turn: intent → plan → gate → act → audit.
    Do {
        /// Approve actions that would otherwise ask first.
        #[arg(long)]
        yes: bool,
        /// What you want.
        intent: Vec<String>,
    },
    /// Undo the last mutating action (or a specific transaction id).
    Undo {
        /// Transaction id (default: the most recent not-yet-undone).
        id: Option<u64>,
    },
    /// Show the transaction timeline (snapshots taken before mutating actions).
    Timeline,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let h = Hearthd::open(cli.tier)?;
    prompt::ensure_defaults(&h.prompt_dir)?;

    match cli.cmd {
        Cmd::Init => {
            println!("hearthd ready.");
            println!("  constitution : {}", h.prompt_dir.display());
            println!("  memory       : {}", h.brain.root.display());
            println!("  tier         : {:?}", h.tier);
            println!("\nTry:  hearthd prompt --tier small");
            println!("      hearthd do \"remember I take my coffee black\" --yes");
        }
        Cmd::Prompt => println!("{}", h.system_prompt()?),
        Cmd::Do { yes, intent } => {
            if intent.is_empty() {
                anyhow::bail!("tell me what you'd like: hearthd do \"…\"");
            }
            h.run(&intent.join(" "), yes)?;
        }
        Cmd::Undo { id } => {
            let t = h.substrate.undo(id)?;
            println!("Reverted tx-{} — {} (restored the snapshot).", t.id, t.summary);
        }
        Cmd::Timeline => {
            let txns = h.substrate.timeline()?;
            if txns.is_empty() {
                println!("No transactions yet.");
            }
            for t in txns {
                println!("tx-{:<3} {}{}", t.id, t.summary, if t.undone { "  (undone)" } else { "" });
            }
        }
    }
    Ok(())
}
