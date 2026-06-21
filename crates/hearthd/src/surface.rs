//! # The Surface DSL — the vetted component grammar a manifestation is composed from
//!
//! "Intent in, **manifestation out**" (VISION §2.4): the steward answers a task by
//! materialising a purpose-built, ephemeral interface — a *surface* — that lives for the
//! task and dissolves. Crucially, what the model emits is **not arbitrary code**: it is a
//! **constrained declarative tree** drawn from the small, safe vocabulary below. That is the
//! security boundary — the model composes from a vetted library; it can never inject
//! executable UI. The shell renders this natively (glass + the type-roles + calm motion).
//!
//! New components are added *here*, to the library — never emitted as free-form markup.

use anyhow::Result;
use hearth_model::{Completion, Model};
use serde::{Deserialize, Serialize};

/// An ephemeral, intent-shaped interface the steward materialises for a task.
#[derive(Serialize, Deserialize, Clone)]
pub struct Surface {
    /// Names the task (the session/window title), in the steward's voice.
    pub title: String,
    /// The composed component tree, rendered top to bottom.
    pub nodes: Vec<Node>,
}

/// One vetted component. Internally tagged (`{"node":"list", ...}`) so the shell can
/// dispatch on a single field, and so an unknown node degrades safely (it is skipped).
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "node", rename_all = "snake_case")]
pub enum Node {
    /// A section label — the system's quiet uppercase eyebrow.
    Heading { text: String },
    /// A paragraph. `voice` picks the typeface role (who is speaking).
    Text {
        text: String,
        #[serde(default)]
        voice: Voice,
    },
    /// A bulleted list.
    List { items: Vec<String> },
    /// Key/value rows — the glass-box "what is this" grammar (mono values).
    Fields { rows: Vec<Field> },
    /// A grid of glass cards (an album, a set of options, a summary).
    Tiles { tiles: Vec<Tile> },
    /// Buttons that hand an intent back to the steward — manifestation bound to action.
    Actions { actions: Vec<Action> },
    /// An editable text field — the bidirectional seam: what the owner types here streams
    /// back to the steward as a structured edit (recorded through the privacy floor).
    Note {
        label: String,
        #[serde(default)]
        value: String,
    },
    /// A hairline rule.
    Divider,
}

/// Who is speaking, encoded in the typeface (UI-SOUL tenet 9): the steward's warm serif,
/// the system's clean sans, or mono for code/data.
#[derive(Serialize, Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
pub enum Voice {
    Steward,
    #[default]
    System,
    Data,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Field {
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Tile {
    pub title: String,
    #[serde(default)]
    pub caption: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Action {
    pub label: String,
    /// The intent this button hands back to the steward when pressed.
    pub intent: String,
    /// Render as the warm primary action (vs. a quiet link).
    #[serde(default)]
    pub primary: bool,
}

impl Surface {
    /// The canonical **reference surface** — it exercises every node in the library. It is
    /// the grammar made tangible: the example a model composes against, the renderer's
    /// fixture, and (until a model is wired) what the steward manifests when asked to show a
    /// surface. Real data and real intents, composed from the vetted vocabulary — no raw code.
    pub fn reference() -> Self {
        Surface {
            title: "A generated surface".into(),
            nodes: vec![
                Node::Text {
                    text: "This interface was described, not coded — I composed it from a small \
                           vetted grammar, and the shell rendered it. It lives for this task and \
                           dissolves when you're done."
                        .into(),
                    voice: Voice::Steward,
                },
                Node::Heading { text: "The grammar".into() },
                Node::List {
                    items: vec![
                        "Text speaks in my serif or the system's sans".into(),
                        "Lists, fields, and tiles carry the data".into(),
                        "Actions hand an intent back to me".into(),
                    ],
                },
                Node::Heading { text: "What this surface is doing".into() },
                Node::Fields {
                    rows: vec![
                        Field { key: "source".into(), value: "Surface DSL — no raw code".into() },
                        Field { key: "rendered by".into(), value: "the Hearth shell".into() },
                        Field { key: "left the machine".into(), value: "nothing".into() },
                    ],
                },
                Node::Heading { text: "Composed from glass".into() },
                Node::Tiles {
                    tiles: vec![
                        Tile { title: "Glass".into(), caption: "every surface is a frosted pane".into() },
                        Tile { title: "Calm".into(), caption: "it arrives; it never demands".into() },
                        Tile { title: "Yours".into(), caption: "cheap, disposable, intent-shaped".into() },
                    ],
                },
                Node::Note {
                    label: "Anything you want me to remember".into(),
                    value: String::new(),
                },
                Node::Divider,
                Node::Actions {
                    actions: vec![
                        Action {
                            label: "What do you know about me?".into(),
                            intent: "what do you know about me?".into(),
                            primary: true,
                        },
                        Action {
                            label: "Remember I prefer concise replies".into(),
                            intent: "remember I prefer concise replies".into(),
                            primary: false,
                        },
                    ],
                },
            ],
        }
    }

    /// Parse a model's reply into a Surface, **leniently**: tolerate prose or markdown fences
    /// around the JSON, and silently drop any node that isn't part of the vetted grammar. The
    /// model can only *compose from* the library — it can never widen it or inject anything the
    /// renderer doesn't already understand. A total parse failure yields an empty surface.
    pub fn from_model_json(raw: &str) -> Self {
        #[derive(Deserialize)]
        struct Raw {
            #[serde(default)]
            title: String,
            #[serde(default)]
            nodes: Vec<serde_json::Value>,
        }
        let json = extract_json(raw);
        let r: Raw =
            serde_json::from_str(&json).unwrap_or(Raw { title: String::new(), nodes: vec![] });
        let nodes = r.nodes.into_iter().filter_map(|v| serde_json::from_value::<Node>(v).ok()).collect();
        Surface { title: r.title, nodes }
    }
}

/// The system prompt the model composes against — the vetted grammar, described.
const COMPOSER_SYSTEM: &str = "\
You are the Hearth steward's surface composer. You turn the owner's request into a SURFACE — a \
small, calm interface — by emitting a JSON object drawn from a FIXED component grammar. You never \
write code, HTML, or anything outside this grammar.\n\n\
Output ONLY one JSON object — no prose, no markdown fences:\n\
{\"title\":\"<short title, in your voice>\",\"nodes\":[ ... ]}\n\n\
Each node is exactly one of:\n\
{\"node\":\"heading\",\"text\":\"...\"}\n\
{\"node\":\"text\",\"text\":\"...\",\"voice\":\"steward|system|data\"}  (steward=warm/you, system=neutral, data=mono)\n\
{\"node\":\"list\",\"items\":[\"...\"]}\n\
{\"node\":\"fields\",\"rows\":[{\"key\":\"...\",\"value\":\"...\"}]}\n\
{\"node\":\"tiles\",\"tiles\":[{\"title\":\"...\",\"caption\":\"...\"}]}\n\
{\"node\":\"actions\",\"actions\":[{\"label\":\"...\",\"intent\":\"...\",\"primary\":true}]}  (pressing a button sends \"intent\" back to you)\n\
{\"node\":\"note\",\"label\":\"...\",\"value\":\"\"}  (an editable field the owner can write in)\n\
{\"node\":\"divider\"}\n\n\
Rules:\n\
- Compose the SMALLEST surface that genuinely helps. Calm, never cluttered.\n\
- Use only real content you were given; do not invent specific facts, numbers, or filenames.\n\
- An action's \"intent\" is a natural-language request you could act on next.\n\
- If no surface would help (a plain acknowledgement is enough), output {\"title\":\"\",\"nodes\":[]}.";

/// Compose a bespoke surface for an intent by having the model emit the DSL. The model picks
/// from the vetted grammar (that is the security boundary); [`Surface::from_model_json`] parses
/// it leniently, so a stray or unknown node can never break the manifestation.
pub fn compose(model: &dyn Model, intent: &str, context: &str) -> Result<Surface> {
    let prompt = format!(
        "The owner's request:\n{intent}\n\nWhat you know / what just happened:\n{context}\n\nCompose the surface now — JSON only.",
    );
    let mut req = Completion::new(COMPOSER_SYSTEM, prompt);
    req.temperature = 0.35;
    req.max_tokens = 1100;
    let raw = model.complete(&req)?;
    Ok(Surface::from_model_json(&raw))
}

/// Pull the JSON object out of a reply that may carry markdown fences or stray prose.
fn extract_json(raw: &str) -> String {
    let s = raw
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    match (s.find('{'), s.rfind('}')) {
        (Some(a), Some(b)) if b > a => s[a..=b].to_string(),
        _ => s.to_string(),
    }
}
