//! # The shell↔runtime protocol — the contract a shell (any shell) drives the steward through
//!
//! `hearthd` is the headless runtime; a **shell** (the mockup today, the native Wayland
//! compositor on Arch later) is a client. This module is the single, versioned source of truth
//! for the messages between them, so a native shell targets a stable contract instead of reverse
//! engineering the HTTP handlers.
//!
//! The contract is **transport-agnostic JSON**. Today the transport is HTTP/1.1 + NDJSON over
//! localhost (see [`crate::server`]); a native shell on the same machine can carry the identical
//! messages over a Unix socket later — the shapes here don't change.
//!
//! ## The flow
//! 1. The shell sends an [`IntentRequest`] (the owner's words, typed or transcribed).
//! 2. The runtime streams [`StreamEvent`]s as the turn unfolds: `recalled` → `plan` → `step`…
//!    → `surface` (the manifestation) → `done`. The shell renders the tool-trail and draws the
//!    surface live.
//! 3. The owner interacts with a live surface — an `actions` button hands an intent back (another
//!    [`IntentRequest`]); editing a `note` streams a [`SurfaceEdit`] back through the privacy floor.
//!
//! ## The render contract
//! A manifestation is a [`Surface`] — a constrained declarative tree of vetted [`Node`]s (text,
//! lists, tiles, actions, media, …). The shell renders the tree natively; it is never sent raw
//! markup or code. That constraint *is* the security boundary (see [`crate::surface`]).
//!
//! ## Not yet in v1 (the roadmap this contract grows into)
//! - an explicit **presence/phase** signal (the steward's six canonical states) — today the shell
//!   infers presence from the event stream;
//! - **window/session lifecycle** beyond "a surface appears, lives, dissolves";
//! - a **native socket** transport and **voice** events (mic/level/partial-transcript) — voice in
//!   and out exist at the runtime edge (`hearthd listen` / `--speak`); a GUI shell will carry them
//!   over this contract.

use serde::Deserialize;

/// The contract version. Bumped when a message shape changes; a shell can read it from
/// `GET /api/protocol` ([`descriptor`]) to negotiate.
pub const PROTOCOL_VERSION: u32 = 1;

// ── Shell → runtime (inbound) ───────────────────────────────────────────────
// These mirror the wire shapes the server accepts, as types instead of ad-hoc field lookups.

/// Run a turn. The owner's words, plus whether actions that would normally ask are pre-approved.
/// Carried by `POST /api/intent`, answered with an NDJSON stream of [`StreamEvent`].
#[derive(Debug, Default, Deserialize)]
pub struct IntentRequest {
    #[serde(default)]
    pub intent: String,
    #[serde(default)]
    pub approve: bool,
}

/// An edit the owner made to a live surface (e.g. typed into a `note`). The bidirectional half of
/// generative surfaces — recorded through the privacy floor. Carried by `POST /api/surface/event`.
#[derive(Debug, Default, Deserialize)]
pub struct SurfaceEdit {
    #[serde(default)]
    pub node: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub value: String,
}

/// Forget a curated memory page — snapshot-first, undoable. Carried by `POST /api/forget`.
#[derive(Debug, Default, Deserialize)]
pub struct ForgetRequest {
    #[serde(default)]
    pub page: String,
}

/// Fire one capability directly — what a surface `actions` button with a `run` binding sends, so a
/// button executes that exact gated action without re-planning (the symmetric human/agent path).
/// Carried by `POST /api/action`; answered with a [`StepResult`] (held if it would `ask`).
#[derive(Debug, Default, Deserialize)]
pub struct ActionRequest {
    #[serde(default)]
    pub capability: String,
    #[serde(default)]
    pub tool: String,
    #[serde(default)]
    pub args: serde_json::Value,
    #[serde(default)]
    pub approve: bool,
}

// ── Runtime → shell (outbound) + render contract ────────────────────────────
// Re-exported here so the whole contract is discoverable in one module. Their definitions live
// next to the code that produces them; this is the index.

pub use crate::surface::{Node, Surface};
pub use crate::{BrainPage, ForgetResult, StepResult, StreamEvent, SurfaceEventResult, TurnResult};

/// A machine-readable summary of the contract, served at `GET /api/protocol` so a shell can
/// discover the version and the message surface before driving the runtime.
pub fn descriptor() -> serde_json::Value {
    serde_json::json!({
        "version": PROTOCOL_VERSION,
        "render": "surface-dsl",
        "transport": "http/1.1 + ndjson over localhost (socket-capable later)",
        "server_events": ["recalled", "plan", "step", "surface", "done", "error"],
        "endpoints": {
            "POST /api/intent": "{intent, approve} → ndjson stream of server_events",
            "POST /api/action": "{capability, tool, args, approve} → run one gated action directly",
            "POST /api/surface/event": "{node, kind, value} → recorded surface edit",
            "POST /api/forget": "{page} → snapshot-first forget",
            "GET /api/brain": "curated memory pages",
            "GET /api/surface": "the reference surface (DSL sample)",
            "GET /api/protocol": "this descriptor"
        }
    })
}
