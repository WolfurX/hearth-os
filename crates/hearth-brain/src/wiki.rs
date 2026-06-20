//! The wiki — the compiled, curated layer: plain-markdown pages the steward keeps
//! about you. Legible and editable by design; a page is parsed as *prose head* +
//! *bullets* so the steward can merge new knowledge without trampling your edits.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

const FOOTER_MARKER: &str = "<!-- hearth-brain:";

/// A single wiki page. Parsed leniently: everything before the first bullet is the
/// `head` (title, blurb, and any prose you add); bullets are the merge target.
pub struct Page {
    pub name: String,
    pub path: PathBuf,
    head: String,
    bullets: Vec<String>,
}

impl Page {
    /// The on-disk path for a wiki-relative page name (e.g. `people/alice`).
    pub fn path_for(wiki_dir: &Path, name: &str) -> PathBuf {
        let mut p = wiki_dir.to_path_buf();
        for part in name.split('/') {
            p.push(part);
        }
        p.set_extension("md");
        p
    }

    /// Load a page, or build an in-memory scaffold if it doesn't exist yet.
    pub fn load_or_new(wiki_dir: &Path, name: &str) -> Result<Self> {
        let path = Self::path_for(wiki_dir, name);
        if path.exists() {
            let raw = std::fs::read_to_string(&path)
                .with_context(|| format!("reading page {}", path.display()))?;
            Ok(Self::parse(name, path, &raw))
        } else {
            Ok(Self {
                name: name.to_string(),
                path,
                head: scaffold(name).trim_end().to_string(),
                bullets: vec![],
            })
        }
    }

    fn parse(name: &str, path: PathBuf, raw: &str) -> Self {
        let mut head_lines = vec![];
        let mut bullets = vec![];
        let mut seen_bullet = false;
        for line in raw.lines() {
            let t = line.trim_start();
            if t.starts_with(FOOTER_MARKER) {
                continue;
            }
            if let Some(b) = t.strip_prefix("- ") {
                bullets.push(b.trim().to_string());
                seen_bullet = true;
            } else if !seen_bullet {
                head_lines.push(line.to_string());
            }
            // non-bullet prose after the bullets begins is intentionally dropped
        }
        Self {
            name: name.to_string(),
            path,
            head: head_lines.join("\n").trim_end().to_string(),
            bullets,
        }
    }

    fn has_bullet(&self, bullet: &str) -> bool {
        let n = normalize(bullet);
        self.bullets.iter().any(|b| normalize(b) == n)
    }

    /// Merge a bullet (idempotent). Returns true if it was actually new.
    pub fn add_bullet(&mut self, bullet: &str) -> bool {
        if self.has_bullet(bullet) {
            return false;
        }
        self.bullets.push(bullet.trim().to_string());
        true
    }

    fn render(&self) -> String {
        let head = if self.head.is_empty() {
            scaffold(&self.name).trim_end().to_string()
        } else {
            self.head.clone()
        };
        let mut s = head;
        s.push_str("\n\n");
        for b in &self.bullets {
            s.push_str("- ");
            s.push_str(b);
            s.push('\n');
        }
        s.push_str(&format!(
            "\n{FOOTER_MARKER} compiled {} UTC · {} note(s) -->\n",
            crate::clock::format_utc(crate::clock::now_unix()),
            self.bullets.len()
        ));
        s
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.path, self.render())
            .with_context(|| format!("writing page {}", self.path.display()))?;
        Ok(())
    }
}

fn normalize(s: &str) -> String {
    s.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_end_matches(['.', '!', ','])
        .to_string()
}

/// The human-facing scaffold for a brand-new page.
fn scaffold(name: &str) -> String {
    let (title, blurb) = describe(name);
    format!(
        "# {title}\n\n*{blurb} Plain markdown — read, edit, or delete any line; the steward will respect your changes.*\n"
    )
}

fn describe(name: &str) -> (String, String) {
    match name {
        "you" => (
            "You".into(),
            "What the steward has learned about how you like to be helped.".into(),
        ),
        "rhythms" => (
            "Rhythms".into(),
            "Your routines and the timing of your days.".into(),
        ),
        "observations" => (
            "Observations".into(),
            "Things the steward has noticed but not yet sorted into curated pages.".into(),
        ),
        n if n.starts_with("people/") => {
            let who = title_case(n.trim_start_matches("people/"));
            (who.clone(), format!("Someone in your life — {who}."))
        }
        n if n.starts_with("projects/") => {
            let p = title_case(n.trim_start_matches("projects/"));
            (p.clone(), format!("A project you're working on — {p}."))
        }
        n if n.starts_with("lessons/") => {
            let l = title_case(n.trim_start_matches("lessons/"));
            (
                format!("Lesson — {l}"),
                "A machine-specific strategy the steward learned operating for you.".into(),
            )
        }
        other => (title_case(other), "Notes.".into()),
    }
}

fn title_case(s: &str) -> String {
    s.split(['-', '_', '/'])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// List all pages (wiki-relative names, no extension), sorted.
pub fn list(wiki_dir: &Path) -> Result<Vec<String>> {
    let mut out = vec![];
    collect(wiki_dir, wiki_dir, &mut out)?;
    out.sort();
    Ok(out)
}

fn collect(base: &Path, dir: &Path, out: &mut Vec<String>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(base, &path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(rel) = path.strip_prefix(base) {
                out.push(rel.with_extension("").to_string_lossy().replace('\\', "/"));
            }
        }
    }
    Ok(())
}

/// Delete a page. Returns true if it existed.
pub fn remove(wiki_dir: &Path, name: &str) -> Result<bool> {
    let path = Page::path_for(wiki_dir, name);
    if path.exists() {
        std::fs::remove_file(&path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Seed the two always-present curated pages so a fresh Brain reads coherently.
pub fn seed_pages(wiki_dir: &Path) -> Result<()> {
    for name in ["you", "rhythms"] {
        let path = Page::path_for(wiki_dir, name);
        if !path.exists() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, scaffold(name))?;
        }
    }
    Ok(())
}
