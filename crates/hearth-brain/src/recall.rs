//! Retrieval — context assembly. This is what `hearthd` calls each turn to pull the
//! few most relevant wiki pages into the model's context. A light keyword overlap for
//! now; the design's semantic index (hybrid keyword + embedding) is a drop-in upgrade
//! behind this same function, because the memory itself is legible prose either way.

use crate::wiki::{self, Page};
use anyhow::Result;
use std::path::Path;

pub struct Hit {
    pub page: String,
    pub score: usize,
    pub snippet: String,
}

/// Select up to `k` wiki pages most relevant to `query`.
pub fn recall(wiki_dir: &Path, query: &str, k: usize) -> Result<Vec<Hit>> {
    let terms: Vec<String> = query
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() > 2)
        .map(|t| t.to_string())
        .collect();
    if terms.is_empty() {
        return Ok(vec![]);
    }

    let mut hits = vec![];
    for name in wiki::list(wiki_dir)? {
        let path = Page::path_for(wiki_dir, &name);
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        let hay = text.to_lowercase();
        let score: usize = terms.iter().map(|t| hay.matches(t.as_str()).count()).sum();
        if score > 0 {
            hits.push(Hit {
                page: name,
                score,
                snippet: first_bullets(&text, 2),
            });
        }
    }
    hits.sort_by(|a, b| b.score.cmp(&a.score));
    hits.truncate(k);
    Ok(hits)
}

fn first_bullets(text: &str, n: usize) -> String {
    text.lines()
        .filter(|l| l.trim_start().starts_with("- "))
        .take(n)
        .collect::<Vec<_>>()
        .join("  ")
}
