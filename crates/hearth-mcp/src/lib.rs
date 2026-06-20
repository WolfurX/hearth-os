//! # hearth-mcp — Model Context Protocol over stdio (minimal)
//!
//! The Linux-Foundation standard for exposing tools to agents (JSON-RPC 2.0). This is a
//! small, faithful implementation of the stdio transport: newline-delimited JSON-RPC.
//!
//! - [`serve`] runs a capability **server** (initialize · tools/list · tools/call).
//! - [`Client`] is the **host** side: it spawns a server subprocess, does the handshake,
//!   and lists/calls its tools.
//!
//! Because it is the real protocol, our capabilities become servers anyone can use, and
//! `hearthd` can connect to any third-party MCP server through the exact same `Client`.

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};

/// The MCP protocol version we speak.
pub const PROTOCOL_VERSION: &str = "2024-11-05";

/// A tool a server exposes.
#[derive(Clone, Debug)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    /// JSON Schema for the arguments (an object schema).
    pub input_schema: Value,
}

/// The backend behind an MCP server: it lists tools and answers calls.
pub trait ToolProvider {
    fn server_name(&self) -> &str;
    fn tools(&self) -> Vec<ToolDef>;
    fn call(&self, name: &str, args: &Value) -> Result<String>;
}

/// Run the stdio JSON-RPC loop for a server until stdin closes.
pub fn serve(provider: impl ToolProvider) -> Result<()> {
    let stdin = std::io::stdin();
    let mut out = std::io::stdout();
    for line in BufReader::new(stdin.lock()).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let msg: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let id = msg.get("id").cloned();
        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");

        // None => a notification (no response is sent).
        let result: Option<Result<Value>> = match method {
            "initialize" => Some(Ok(json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": { "tools": {} },
                "serverInfo": { "name": provider.server_name(), "version": "0.1.0" }
            }))),
            "notifications/initialized" => None,
            "tools/list" => {
                let tools: Vec<Value> = provider
                    .tools()
                    .iter()
                    .map(|t| json!({ "name": t.name, "description": t.description, "inputSchema": t.input_schema }))
                    .collect();
                Some(Ok(json!({ "tools": tools })))
            }
            "tools/call" => {
                let params = msg.get("params").cloned().unwrap_or_else(|| json!({}));
                let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
                Some(match provider.call(name, &args) {
                    Ok(text) => Ok(json!({ "content": [{ "type": "text", "text": text }], "isError": false })),
                    Err(e) => Ok(json!({ "content": [{ "type": "text", "text": format!("{e}") }], "isError": true })),
                })
            }
            _ => Some(Err(anyhow::anyhow!("method not found: {method}"))),
        };

        if let (Some(id), Some(res)) = (id, result) {
            let response = match res {
                Ok(r) => json!({ "jsonrpc": "2.0", "id": id, "result": r }),
                Err(e) => json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": e.to_string() } }),
            };
            writeln!(out, "{}", serde_json::to_string(&response)?)?;
            out.flush()?;
        }
    }
    Ok(())
}

/// A host connection to an MCP server subprocess.
pub struct Client {
    child: std::process::Child,
    stdin: std::process::ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
    next_id: u64,
    pub name: String,
}

impl Client {
    /// Spawn a server and complete the initialize handshake.
    pub fn spawn(program: &std::path::Path, args: &[&str]) -> Result<Self> {
        use std::process::{Command, Stdio};
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| format!("spawning MCP server {}", program.display()))?;
        let stdin = child.stdin.take().context("server has no stdin")?;
        let stdout = child.stdout.take().context("server has no stdout")?;
        let mut c = Client {
            child,
            stdin,
            reader: BufReader::new(stdout),
            next_id: 1,
            name: String::new(),
        };
        let init = c.request(
            "initialize",
            json!({ "protocolVersion": PROTOCOL_VERSION, "capabilities": {}, "clientInfo": { "name": "hearthd", "version": "0.1.0" } }),
        )?;
        c.name = init
            .get("serverInfo")
            .and_then(|s| s.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("mcp")
            .to_string();
        c.notify("notifications/initialized", json!({}))?;
        Ok(c)
    }

    fn send(&mut self, msg: &Value) -> Result<()> {
        writeln!(self.stdin, "{}", serde_json::to_string(msg)?)?;
        self.stdin.flush()?;
        Ok(())
    }

    fn read_response(&mut self) -> Result<Value> {
        let mut line = String::new();
        loop {
            line.clear();
            if self.reader.read_line(&mut line)? == 0 {
                bail!("MCP server closed the connection");
            }
            if line.trim().is_empty() {
                continue;
            }
            return serde_json::from_str(&line).context("parsing MCP response");
        }
    }

    fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id;
        self.next_id += 1;
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }))?;
        let resp = self.read_response()?;
        if let Some(err) = resp.get("error") {
            bail!("MCP error: {err}");
        }
        Ok(resp.get("result").cloned().unwrap_or_else(|| json!({})))
    }

    fn notify(&mut self, method: &str, params: Value) -> Result<()> {
        self.send(&json!({ "jsonrpc": "2.0", "method": method, "params": params }))
    }

    pub fn list_tools(&mut self) -> Result<Vec<ToolDef>> {
        let r = self.request("tools/list", json!({}))?;
        let mut out = vec![];
        if let Some(arr) = r.get("tools").and_then(|t| t.as_array()) {
            for t in arr {
                out.push(ToolDef {
                    name: t.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string(),
                    description: t.get("description").and_then(|d| d.as_str()).unwrap_or("").to_string(),
                    input_schema: t.get("inputSchema").cloned().unwrap_or_else(|| json!({})),
                });
            }
        }
        Ok(out)
    }

    pub fn call(&mut self, name: &str, args: &Value) -> Result<String> {
        let r = self.request("tools/call", json!({ "name": name, "arguments": args }))?;
        let is_error = r.get("isError").and_then(|b| b.as_bool()).unwrap_or(false);
        let text = r
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|a| a.first())
            .and_then(|c| c.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        if is_error {
            bail!("{text}");
        }
        Ok(text)
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
