//! # hearth-model — the neutral model router
//!
//! Hearth OS is model-agnostic to its core: *"no model is the master."* Local, API, and
//! subscription backends are equal first-class citizens, each implementing the [`Model`]
//! trait, and nothing above the router knows or cares which is active. Swapping the model
//! is a settings change, never a reinstall — and because the Brain is externalized text,
//! accumulated knowledge survives the swap.
//!
//! [`MockModel`] is an offline, dependency-free stand-in. [`HttpModel`] talks to any
//! OpenAI-compatible endpoint (OpenRouter, OpenAI, a local llama.cpp/Ollama server) by
//! shelling out to `curl` — deliberately, so the build needs no Rust TLS stack.

use anyhow::{Context, Result};

/// One request to a model: system instructions + the prompt, with sampling knobs.
#[derive(Debug, Clone)]
pub struct Completion {
    pub system: String,
    pub prompt: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl Completion {
    pub fn new(system: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            system: system.into(),
            prompt: prompt.into(),
            temperature: 0.2,
            max_tokens: 2048,
        }
    }
}

/// The neutral interface every backend implements — the seam that makes the intelligence
/// a swappable organ.
pub trait Model: Send + Sync {
    /// A stable identifier for audit/logging (e.g. `"mock"`, `"http:openai/gpt-4o-mini"`).
    fn id(&self) -> &str;
    /// Complete a request, returning the model's text.
    fn complete(&self, req: &Completion) -> Result<String>;
}

/// An offline, dependency-free stand-in. Not intelligent — it exists so the trait/router
/// seam can be exercised with zero setup, and components can be tested without a network.
pub struct MockModel {
    id: String,
}

impl MockModel {
    pub fn new() -> Self {
        Self { id: "mock".to_string() }
    }
}

impl Default for MockModel {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for MockModel {
    fn id(&self) -> &str {
        &self.id
    }
    fn complete(&self, req: &Completion) -> Result<String> {
        let first = req.prompt.lines().next().unwrap_or("").trim();
        Ok(format!("[{}] {}", self.id, first))
    }
}

/// The router holds the active backend. Policy routing and graceful degradation are future
/// extensions; the trait boundary already makes them additive.
pub struct Router {
    active: Box<dyn Model>,
}

impl Router {
    pub fn new(active: Box<dyn Model>) -> Self {
        Self { active }
    }
    pub fn offline() -> Self {
        Self::new(Box::new(MockModel::new()))
    }
    pub fn active(&self) -> &dyn Model {
        self.active.as_ref()
    }
    pub fn id(&self) -> &str {
        self.active.id()
    }
}

/// A real backend over any OpenAI-compatible `/chat/completions` endpoint — OpenRouter,
/// OpenAI, a local llama.cpp/Ollama server, etc. Configured by environment so no secret is
/// ever baked into the binary:
///
/// ```text
/// HEARTH_MODEL_URL    e.g. https://openrouter.ai/api/v1
/// HEARTH_MODEL_KEY    bearer token (omit for keyless local servers)
/// HEARTH_MODEL_NAME   e.g. openai/gpt-4o-mini
/// ```
///
/// The request is made by shelling out to `curl` — on purpose, so the lean build needs no
/// Rust TLS stack (and thus no C toolchain). `curl` ships with Windows 10+/11 and every
/// Linux; a native client can replace this later without changing any caller.
pub struct HttpModel {
    id: String,
    url: String,
    key: Option<String>,
    model: String,
}

impl HttpModel {
    /// Build from the `HEARTH_MODEL_*` environment variables.
    pub fn from_env() -> Result<Self> {
        let url = std::env::var("HEARTH_MODEL_URL").context("HEARTH_MODEL_URL not set")?;
        let model = std::env::var("HEARTH_MODEL_NAME").context("HEARTH_MODEL_NAME not set")?;
        let key = std::env::var("HEARTH_MODEL_KEY").ok();
        Ok(Self {
            id: format!("http:{model}"),
            url: url.trim_end_matches('/').to_string(),
            key,
            model,
        })
    }
}

impl Model for HttpModel {
    fn id(&self) -> &str {
        &self.id
    }
    fn complete(&self, req: &Completion) -> Result<String> {
        use std::io::Write;
        use std::process::{Command, Stdio};

        let endpoint = format!("{}/chat/completions", self.url);
        let body = serde_json::json!({
            "model": self.model,
            "temperature": req.temperature,
            "max_tokens": req.max_tokens,
            "messages": [
                { "role": "system", "content": req.system },
                { "role": "user", "content": req.prompt },
            ],
        });
        let body = serde_json::to_string(&body)?;

        let mut cmd = Command::new("curl");
        cmd.arg("-sS")
            .arg("-X")
            .arg("POST")
            .arg(&endpoint)
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("-H")
            .arg("HTTP-Referer: https://hearth.local")
            .arg("-H")
            .arg("X-Title: Hearth OS");
        if let Some(k) = &self.key {
            cmd.arg("-H").arg(format!("Authorization: Bearer {k}"));
        }
        cmd.arg("--data-binary")
            .arg("@-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().context("failed to run `curl` (is it installed?)")?;
        {
            let mut si = child.stdin.take().context("could not open curl stdin")?;
            si.write_all(body.as_bytes())?;
        } // drop closes stdin so curl sends and finishes
        let out = child.wait_with_output()?;
        if !out.status.success() {
            anyhow::bail!("curl failed: {}", String::from_utf8_lossy(&out.stderr));
        }
        let v: serde_json::Value = serde_json::from_slice(&out.stdout).with_context(|| {
            format!("model response was not JSON: {}", String::from_utf8_lossy(&out.stdout))
        })?;
        if let Some(err) = v.get("error") {
            anyhow::bail!("model API error: {err}");
        }
        v["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .with_context(|| format!("unexpected response shape: {v}"))
    }
}
