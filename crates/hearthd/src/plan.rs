//! The plan object — what the steward proposes before it acts. The glass box renders it;
//! `hearthd` validates and gates it. It is the seam between "the model reasoned" and
//! "the system did," and the unit of accountability.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct Step {
    pub capability: String,
    pub tool: String,
    #[serde(default)]
    pub args: Value,
    #[serde(default)]
    pub why: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Plan {
    pub summary: String,
    pub steps: Vec<Step>,
}

impl Plan {
    /// Glass box, plain language.
    pub fn render_plain(&self) -> String {
        let mut s = format!("  plan · {}\n", self.summary);
        for (i, st) in self.steps.iter().enumerate() {
            let why = if st.why.is_empty() { "(no reason given)" } else { &st.why };
            s.push_str(&format!("   {}. {}.{} — {}\n", i + 1, st.capability, st.tool, why));
        }
        s
    }
}
