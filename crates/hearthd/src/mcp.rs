//! The MCP host. `hearthd` connects to capability **servers** (subprocesses), federates
//! their tools alongside the in-process ones, and subjects them to the same permission
//! gate and snapshot substrate. The same `Client` would connect to any third-party MCP
//! server in the ecosystem — that interop is the whole point of speaking the standard.

use crate::capability::Decision;
use anyhow::{Context, Result};
use hearth_mcp::{Client, ToolDef};
use serde_json::Value;
use std::cell::RefCell;
use std::path::PathBuf;

pub struct McpHost {
    servers: Vec<RefCell<Client>>,
    tools: Vec<(usize, ToolDef)>,
}

impl McpHost {
    /// Connect to each server program that exists, listing its tools. A missing or broken
    /// server is surfaced and skipped — it never stops the steward.
    pub fn connect(programs: Vec<PathBuf>) -> Self {
        let mut servers = vec![];
        let mut tools = vec![];
        for prog in programs {
            if !prog.exists() {
                continue;
            }
            match Client::spawn(&prog, &[]) {
                Ok(mut c) => match c.list_tools() {
                    Ok(ts) => {
                        let names: Vec<&str> = ts.iter().map(|t| t.name.as_str()).collect();
                        eprintln!("· MCP host: connected '{}' ({})", c.name, names.join(", "));
                        let idx = servers.len();
                        for t in ts {
                            tools.push((idx, t));
                        }
                        servers.push(RefCell::new(c));
                    }
                    Err(e) => eprintln!("· MCP host: {} listed no tools: {e}", prog.display()),
                },
                Err(e) => eprintln!("· MCP host: could not start {}: {e}", prog.display()),
            }
        }
        Self { servers, tools }
    }

    pub fn tools(&self) -> &[(usize, ToolDef)] {
        &self.tools
    }

    pub fn has(&self, name: &str) -> bool {
        self.tools.iter().any(|(_, t)| t.name == name)
    }

    pub fn call(&self, name: &str, args: &Value) -> Result<String> {
        let entry = self
            .tools
            .iter()
            .find(|(_, t)| t.name == name)
            .context("no such MCP tool")?;
        self.servers[entry.0].borrow_mut().call(name, args)
    }
}

/// The gateway's permission policy for MCP tools. A real deployment configures this per
/// server/tool; here read-only `fs.*` is auto, and anything else asks first (untrusted by
/// default — third-party servers should never auto-run).
pub fn policy(name: &str) -> Decision {
    if name.starts_with("fs.") {
        Decision::Auto
    } else {
        Decision::Ask
    }
}

pub fn mutating(name: &str) -> bool {
    !name.starts_with("fs.")
}
