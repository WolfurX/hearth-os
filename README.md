# Hearth OS

*An agentic operating system on an Arch Linux base, where the machine is a **steward,
not a tool**.* This repository is the **real codebase** — Hearth OS "line by owned
line." The soul (**The Hearth**) and the Brain concept were validated in Phase 0 by
the interactive prototype in [`../hearth-os-prototype/`](../hearth-os-prototype/); the
north-star design lives in that folder's `VISION-AND-ARCHITECTURE.md`.

## Phase 1 — the mind, the fabric, and the first Brain

The first stake in the ground is **the Brain**: the system's legible, model-portable
long-term memory. It is the most novel piece, it stands alone, and building it for
real exercises the two seams everything else needs — the **neutral model router** and
the **append-only activity log**.

### Workspace

| Crate | What it is |
|---|---|
| **`hearth-model`** | The neutral model router. Local / API / subscription backends all implement one `Model` trait; nothing above the router knows which is active. Base build is pure-Rust and offline; a real HTTP backend is behind the `online` feature. |
| **`hearth-brain`** | The Brain (an LLM-wiki). Three layers on disk: an append-only **activity log** (ground truth), a compiled **markdown wiki** (what the steward knows — readable, editable, forgettable), and a **schema** (the rules by which raw activity becomes knowledge). Ships as the `hearth-brain` CLI. |

Future Phase 1 crates — `hearthd` (the agent runtime), the MCP capability fabric, and
the sovereignty substrate — slot in alongside these.

## Build & run

Rust (stable) only — no other system dependencies. On this Windows dev box the
toolchain is the self-contained **GNU** host (`x86_64-pc-windows-gnu`), so no Visual
Studio is needed; the target is Arch Linux and the code is cross-platform.

```sh
cargo build
cargo run -p hearth-brain -- init
cargo run -p hearth-brain -- remember "I prefer concise replies"
cargo run -p hearth-brain -- log --kind lesson --tag wifi \
    "After a kernel update, rebuild the rtl8821ce DKMS module before reconnecting wifi"
cargo run -p hearth-brain -- compile
cargo run -p hearth-brain -- whoami        # "what do you know about me?"
cargo run -p hearth-brain -- recall wifi
```

The Brain's data lives in `$HEARTH_HOME/brain` (default `~/.hearth/brain`), **not** in
this source tree, and is its own git repo so `forget` is auditable and any
consolidation can be rolled back. Override per-invocation with `--brain <dir>`.

### Using a real model (optional)

Consolidation runs offline by default via a deterministic heuristic compiler. To use a
real model for semantic consolidation — any OpenAI-compatible endpoint, including a
local llama.cpp / Ollama / vLLM server — build with the `online` feature and set:

```sh
export HEARTH_MODEL_URL=http://localhost:11434/v1   # a local server, e.g.
export HEARTH_MODEL_NAME=llama3.1
# export HEARTH_MODEL_KEY=...                        # for hosted APIs
cargo run -p hearth-brain --features online -- compile
```

Because the Brain is externalized text, swapping the model never costs you what it has
learned. That is the point: *the intelligence is swappable; the relationship is not.*

## Locked decisions

- **Soul:** The Hearth. **Model strategy:** neutral (no default lean). **Build:**
  purist (own every layer we reasonably can; kernel + drivers stay upstream but remain
  open and AI-patchable). See the Phase 0 vision doc for the full rationale.
