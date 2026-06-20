//! `hearth-mcp-fs` — a real MCP server exposing read-only filesystem tools. `hearthd`
//! connects to it as a host; it could equally be used by any other MCP client.

use anyhow::Result;
use hearth_mcp::{serve, ToolDef, ToolProvider};
use serde_json::{json, Value};

struct Fs;

impl ToolProvider for Fs {
    fn server_name(&self) -> &str {
        "hearth-fs"
    }

    fn tools(&self) -> Vec<ToolDef> {
        let path_schema = json!({ "type": "object", "properties": { "path": { "type": "string" } }, "required": ["path"] });
        vec![
            ToolDef {
                name: "fs.list".into(),
                description: "List the files in a directory (read-only).".into(),
                input_schema: path_schema.clone(),
            },
            ToolDef {
                name: "fs.read".into(),
                description: "Read a text file (read-only).".into(),
                input_schema: path_schema,
            },
        ]
    }

    fn call(&self, name: &str, args: &Value) -> Result<String> {
        let path = |k: &str| args.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
        match name {
            "fs.list" => {
                let p = {
                    let x = path("path");
                    if x.is_empty() { ".".to_string() } else { x }
                };
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
            "fs.read" => {
                let p = path("path");
                if p.is_empty() {
                    anyhow::bail!("no file path given");
                }
                let text = std::fs::read_to_string(&p).map_err(|e| anyhow::anyhow!("can't read {p}: {e}"))?;
                let snippet: String = text.chars().take(1200).collect();
                let more = if text.len() > snippet.len() { "\n… (truncated)" } else { "" };
                Ok(format!("{p} ({} bytes):\n{snippet}{more}", text.len()))
            }
            _ => anyhow::bail!("unknown tool {name}"),
        }
    }
}

fn main() -> Result<()> {
    serve(Fs)
}
