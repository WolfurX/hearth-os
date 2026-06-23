//! `hearthd` CLI — drive the steward's loop, and inspect the system prompt it assembles.

use anyhow::Result;
use clap::{Parser, Subcommand};
use hearthd::prompt::{self, Tier};
use hearthd::Hearthd;
use std::path::PathBuf;

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
        /// Speak the reply aloud (system text-to-speech).
        #[arg(long)]
        speak: bool,
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
    /// Run the local server — serves the UI and the API for the shell to drive.
    Serve {
        /// Address to bind.
        #[arg(long, default_value = "127.0.0.1:7878")]
        addr: String,
        /// Path to the UI file (the-hearth.html) to serve at `/`.
        #[arg(long)]
        ui: Option<PathBuf>,
    },
    /// Talk by voice: listen on the mic, reply aloud, repeat (Arch: needs alsa-utils +
    /// whisper.cpp + HEARTH_WHISPER_MODEL).
    Listen {
        /// Seconds to record per turn.
        #[arg(long, default_value = "6")]
        seconds: u32,
        /// Approve actions that would otherwise ask first.
        #[arg(long)]
        yes: bool,
    },
    /// The steward's return: review what happened since you last looked.
    Review,
    /// Run a single capability directly — the same gated, undoable path the steward uses.
    Run {
        /// The capability and tool, e.g. `fs.read` or `brain.recall`.
        tool: String,
        /// JSON arguments, e.g. `{"path":"./Cargo.toml"}`.
        #[arg(long, default_value = "{}")]
        args: String,
        /// Approve if it would otherwise ask first.
        #[arg(long)]
        yes: bool,
    },
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
        Cmd::Do { yes, speak, intent } => {
            if intent.is_empty() {
                anyhow::bail!("tell me what you'd like: hearthd do \"…\"");
            }
            h.run(&intent.join(" "), yes, speak)?;
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
        Cmd::Serve { addr, ui } => {
            hearthd::server::serve(h, &addr, ui)?;
        }
        Cmd::Listen { seconds, yes } => {
            h.listen(seconds, yes)?;
        }
        Cmd::Review => {
            let r = h.review()?;
            if r.acted.is_empty() && r.asked.is_empty() && r.learned.is_empty() {
                println!("· nothing new since your last visit.");
            } else {
                println!("Welcome back. While you were away:");
                if !r.acted.is_empty() {
                    println!("  · I handled {} thing(s) on my own — {}", r.acted.len(), distinct(&r.acted));
                }
                if !r.asked.is_empty() {
                    println!("  · I asked you first on {} — {}", r.asked.len(), distinct(&r.asked));
                }
                for l in &r.learned {
                    println!("  · I learned: {l}");
                }
            }
            h.mark_reviewed(r.up_to)?;
        }
        Cmd::Run { tool, args, yes } => {
            let (cap, t) = tool.split_once('.').unwrap_or((tool.as_str(), ""));
            let args: serde_json::Value =
                serde_json::from_str(&args).map_err(|e| anyhow::anyhow!("bad --args JSON: {e}"))?;
            let sr = h.invoke(cap, t, &args, yes)?;
            if sr.ran {
                if let Some(tx) = sr.snapshot {
                    println!("✓ snapshot tx-{tx} taken first — `hearthd undo` reverts this");
                }
                println!("→ {}", sr.result.as_deref().unwrap_or(""));
            } else if sr.decision == "forbid" {
                println!("✗ {}.{} is not permitted", sr.capability, sr.tool);
            } else {
                println!("⏸ {}.{} needs approval — re-run with --yes", sr.capability, sr.tool);
            }
        }
    }
    Ok(())
}

/// Distinct items with counts, e.g. `"fs.write ×2, fs.move"`.
fn distinct(items: &[String]) -> String {
    let mut counts: Vec<(String, usize)> = vec![];
    for it in items {
        match counts.iter_mut().find(|(n, _)| n == it) {
            Some(e) => e.1 += 1,
            None => counts.push((it.clone(), 1)),
        }
    }
    counts
        .iter()
        .map(|(n, c)| if *c > 1 { format!("{n} ×{c}") } else { n.clone() })
        .collect::<Vec<_>>()
        .join(", ")
}
