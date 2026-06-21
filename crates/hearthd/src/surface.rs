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

use serde::Serialize;

/// An ephemeral, intent-shaped interface the steward materialises for a task.
#[derive(Serialize, Clone)]
pub struct Surface {
    /// Names the task (the session/window title), in the steward's voice.
    pub title: String,
    /// The composed component tree, rendered top to bottom.
    pub nodes: Vec<Node>,
}

/// One vetted component. Internally tagged (`{"node":"list", ...}`) so the shell can
/// dispatch on a single field, and so an unknown node degrades safely (it is skipped).
#[derive(Serialize, Clone)]
#[serde(tag = "node", rename_all = "snake_case")]
pub enum Node {
    /// A section label — the system's quiet uppercase eyebrow.
    Heading { text: String },
    /// A paragraph. `voice` picks the typeface role (who is speaking).
    Text { text: String, voice: Voice },
    /// A bulleted list.
    List { items: Vec<String> },
    /// Key/value rows — the glass-box "what is this" grammar (mono values).
    Fields { rows: Vec<Field> },
    /// A grid of glass cards (an album, a set of options, a summary).
    Tiles { tiles: Vec<Tile> },
    /// Buttons that hand an intent back to the steward — manifestation bound to action.
    Actions { actions: Vec<Action> },
    /// A hairline rule.
    Divider,
}

/// Who is speaking, encoded in the typeface (UI-SOUL tenet 9): the steward's warm serif,
/// the system's clean sans, or mono for code/data.
#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Voice {
    Steward,
    System,
    Data,
}

#[derive(Serialize, Clone)]
pub struct Field {
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Clone)]
pub struct Tile {
    pub title: String,
    pub caption: String,
}

#[derive(Serialize, Clone)]
pub struct Action {
    pub label: String,
    /// The intent this button hands back to the steward when pressed.
    pub intent: String,
    /// Render as the warm primary action (vs. a quiet link).
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
}
