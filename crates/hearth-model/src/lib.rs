//! # hearth-model — the neutral model router
//!
//! Hearth OS is model-agnostic to its core: *"no model is the master."* Local, API,
//! and subscription backends are equal first-class citizens, each implementing the
//! [`Model`] trait, and nothing above the router knows or cares which is active.
//! Swapping the model is a settings change, never a reinstall — and because the
//! Brain (`hearth-brain`) is externalized text, your accumulated knowledge survives
//! the swap.
//!
//! The base build is pure-Rust and fully offline (see [`MockModel`]). A real HTTP
//! backend — OpenAI / Anthropic / any local llama.cpp-class server speaking a
//! compatible API — lives behind the `online` feature so it never bloats or blocks
//! an offline build.

use anyhow::Result;

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

/// The neutral interface every backend implements. This is the seam that makes the
/// intelligence a swappable organ.
pub trait Model: Send + Sync {
    /// A stable identifier for audit/logging (e.g. `"mock"`, `"http:claude-…"`).
    fn id(&self) -> &str;
    /// Complete a request, returning the model's text.
    fn complete(&self, req: &Completion) -> Result<String>;
}

/// An offline, dependency-free stand-in. It is **not** intelligent — it exists so
/// the trait/router seam is exercised with zero setup, and so components that take a
/// `&dyn Model` can be unit-tested without a network or a key. Components that need
/// real reasoning (e.g. the Brain's model-backed consolidation) degrade to their own
/// offline path when only the mock is available.
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
        // Deterministic echo of the first prompt line — enough to prove the wiring.
        let first = req.prompt.lines().next().unwrap_or("").trim();
        Ok(format!("[{}] {}", self.id, first))
    }
}

/// The router holds the active backend. Policy routing ("keep anything touching
/// personal files on the local model; escalate hard reasoning to the API") and
/// graceful degradation are future extensions; the trait boundary already makes them
/// additive rather than invasive.
pub struct Router {
    active: Box<dyn Model>,
}

impl Router {
    pub fn new(active: Box<dyn Model>) -> Self {
        Self { active }
    }

    /// A router backed by the offline mock — the default when nothing is configured.
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

#[cfg(feature = "online")]
pub mod http {
    //! A real backend speaking the OpenAI-compatible `/chat/completions` shape, which
    //! Anthropic-compatible gateways, vLLM, llama.cpp's server, Ollama, and LM Studio
    //! all expose. Configured by environment so no secret is ever baked into the
    //! binary:
    //!
    //! ```text
    //! HEARTH_MODEL_URL   e.g. http://localhost:11434/v1   (a local server)
    //! HEARTH_MODEL_KEY   bearer token (omit for keyless local servers)
    //! HEARTH_MODEL_NAME  e.g. llama3.1 / claude-… / gpt-…
    //! ```
    use super::{Completion, Model};
    use anyhow::{Context, Result};
    use serde_json::json;

    pub struct OpenAiCompatModel {
        id: String,
        url: String,
        key: Option<String>,
        model: String,
        client: reqwest::blocking::Client,
    }

    impl OpenAiCompatModel {
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
                client: reqwest::blocking::Client::new(),
            })
        }
    }

    impl Model for OpenAiCompatModel {
        fn id(&self) -> &str {
            &self.id
        }
        fn complete(&self, req: &Completion) -> Result<String> {
            let endpoint = format!("{}/chat/completions", self.url);
            let body = json!({
                "model": self.model,
                "temperature": req.temperature,
                "max_tokens": req.max_tokens,
                "messages": [
                    { "role": "system", "content": req.system },
                    { "role": "user", "content": req.prompt },
                ],
            });
            let mut r = self.client.post(&endpoint).json(&body);
            if let Some(k) = &self.key {
                r = r.bearer_auth(k);
            }
            let resp = r.send().context("model request failed")?;
            let status = resp.status();
            let v: serde_json::Value = resp.json().context("model response was not JSON")?;
            if !status.is_success() {
                anyhow::bail!("model backend returned {status}: {v}");
            }
            let text = v["choices"][0]["message"]["content"]
                .as_str()
                .context("unexpected response shape (no choices[0].message.content)")?;
            Ok(text.to_string())
        }
    }
}
