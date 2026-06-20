//! The capability fabric (seed). In the full OS these are MCP servers (`fs`, `pkg`, `web`,
//! …); here they are a small in-process registry so the loop is real end-to-end. Every
//! tool carries a permission [`Decision`] the runtime enforces *before* the model's plan
//! runs — this is where a weak (or hijacked) model is made safe structurally, not by prose.

use anyhow::Result;
use hearth_brain::Brain;
use serde_json::Value;

/// What the steward may do with a tool, decided before it runs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Decision {
    /// Safe / read-only — just do it.
    Auto,
    /// Consequential — needs the owner's approval.
    Ask,
    /// Not permitted.
    Forbid,
}

pub struct ToolSpec {
    pub cap: &'static str,
    pub tool: &'static str,
    pub args: &'static str,
    pub about: &'static str,
    pub policy: Decision,
    /// Does this tool change state? Mutating actions are snapshotted first (undoable).
    pub mutating: bool,
}

pub struct Registry {
    pub tools: Vec<ToolSpec>,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            tools: vec![
                ToolSpec { cap: "respond", tool: "say", args: "{ text }", about: "Answer the owner in plain words (no side effects).", policy: Decision::Auto, mutating: false },
                ToolSpec { cap: "brain", tool: "recall", args: "{ query }", about: "Look up what is known about the owner.", policy: Decision::Auto, mutating: false },
                ToolSpec { cap: "brain", tool: "remember", args: "{ text }", about: "Write a fact or preference to the owner's memory.", policy: Decision::Ask, mutating: true },
                ToolSpec { cap: "fs", tool: "list", args: "{ path }", about: "List the files in a directory (read-only).", policy: Decision::Auto, mutating: false },
                ToolSpec { cap: "fs", tool: "read", args: "{ path }", about: "Read a text file (read-only).", policy: Decision::Auto, mutating: false },
            ],
        }
    }
}

impl Registry {
    /// The permission for a tool — unknown tools are forbidden by default (deny-by-default).
    pub fn policy(&self, cap: &str, tool: &str) -> Decision {
        self.tools
            .iter()
            .find(|t| t.cap == cap && t.tool == tool)
            .map(|t| t.policy)
            .unwrap_or(Decision::Forbid)
    }

    /// Whether a tool changes state (so the runtime snapshots before running it).
    pub fn is_mutating(&self, cap: &str, tool: &str) -> bool {
        self.tools.iter().find(|t| t.cap == cap && t.tool == tool).map(|t| t.mutating).unwrap_or(false)
    }

    /// The tool list spliced into the system prompt each turn.
    pub fn tool_list(&self) -> String {
        self.tools
            .iter()
            .map(|t| format!("- {}.{} {}  — {}", t.cap, t.tool, t.args, t.about))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Run a tool. Returns a human-readable result. Unknown tools are refused.
    pub fn execute(&self, brain: &Brain, cap: &str, tool: &str, args: &Value) -> Result<String> {
        let s = |k: &str| args.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
        match (cap, tool) {
            ("respond", "say") => Ok(s("text")),
            ("brain", "recall") => {
                let q = s("query");
                let hits = hearth_brain::recall::recall(&brain.wiki_dir(), &q, 3)?;
                if hits.is_empty() {
                    Ok(format!("(nothing in memory about \"{q}\" yet)"))
                } else {
                    Ok(hits
                        .iter()
                        .map(|h| format!("{} — {}", h.page, h.snippet))
                        .collect::<Vec<_>>()
                        .join("  ·  "))
                }
            }
            ("brain", "remember") => {
                let text = s("text");
                if text.trim().is_empty() {
                    anyhow::bail!("nothing to remember");
                }
                if let Some(term) = brain.schema.forbidden_term(&text) {
                    return Ok(format!(
                        "Refused — that looks like a secret (matched \"{term}\"). Secrets go in the vault, not memory."
                    ));
                }
                let ev = hearth_brain::log::append(
                    &brain.log_path(),
                    hearth_brain::log::Kind::Preference,
                    &text,
                    vec![],
                )?;
                Ok(format!("remembered (#{}) — a Brain compile will fold it into the wiki", ev.id))
            }
            ("fs", "list") => {
                let p = { let x = s("path"); if x.is_empty() { ".".to_string() } else { x } };
                let mut out = vec![];
                for entry in std::fs::read_dir(&p).map_err(|e| anyhow::anyhow!("can't list {p}: {e}"))? {
                    let entry = entry?;
                    let slash = if entry.path().is_dir() { "/" } else { "" };
                    out.push(format!("{}{slash}", entry.file_name().to_string_lossy()));
                    if out.len() >= 60 {
                        break;
                    }
                }
                out.sort();
                Ok(format!("{} item(s) in {p}:  {}", out.len(), out.join("  ")))
            }
            ("fs", "read") => {
                let p = s("path");
                if p.is_empty() {
                    anyhow::bail!("no file path given");
                }
                let text = std::fs::read_to_string(&p).map_err(|e| anyhow::anyhow!("can't read {p}: {e}"))?;
                let snippet: String = text.chars().take(1200).collect();
                let more = if text.len() > snippet.len() { "\n… (truncated)" } else { "" };
                Ok(format!("{p} ({} bytes):\n{snippet}{more}", text.len()))
            }
            _ => anyhow::bail!("unknown or unpermitted tool {cap}.{tool}"),
        }
    }
}
