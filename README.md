# Hearth OS

> *An agentic operating system on an Arch Linux base, where the machine is a **steward, not a tool**.*

You express intent; the steward materializes the exact interface the task needs, then dissolves it.
Every action is explainable, reversible, and yours. The intelligence is a **swappable organ**; your
memory is **legible text** you can read, edit, and forget. **The machine is yours — and stays yours.**

**Status:** built in public · **Phase 1 (the runtime) is complete** and runs on a real model.

---

## The idea

For fifty years an operating system has been a box of tools you operate. Hearth inverts that: the
computer becomes a counterpart you collaborate with — one that can read and change every line of
itself, act on your behalf, learn you over time, and stay accountable to you.

Not the desktop (manipulating inert objects), not the chatbot (a keyhole of text), but a **shared,
living workspace mediated by a trusted intelligence**. The breakthrough, in one line: **intent in,
manifestation out** — and because every line is accessible, the manifestation is never a black box.

The full thesis: **[docs/VISION-AND-ARCHITECTURE.md](docs/VISION-AND-ARCHITECTURE.md)**.

## See it

**The UI** — open **[`mockup/the-hearth.html`](mockup/the-hearth.html)** in Chrome or Edge (just
double-click it). The definitive "Aurora" interface: an inhabited field where windows are *agent
sessions* (not apps), attention-not-windows focus, the glass box, the living Brain. Scripted — no
model needed.

**The runtime** — a real, headless steward:

```sh
cargo run -p hearthd -- init
cargo run -p hearthd -- do "remember I take my coffee black" --yes
cargo run -p hearthd -- do "what do you know about me?"
cargo run -p hearthd -- timeline      # snapshots taken before each mutating action
cargo run -p hearthd -- undo          # one gesture reverts the last
cargo run -p hearthd -- prompt        # the editable constitution it runs on
```

Point it at any OpenAI-compatible model and it reasons for real — OpenRouter, OpenAI, or a local
llama.cpp / Ollama server:

```sh
export HEARTH_MODEL_URL=https://openrouter.ai/api/v1
export HEARTH_MODEL_KEY=sk-...
export HEARTH_MODEL_NAME=openai/gpt-4o-mini
cargo run -p hearthd -- do "what files are in this folder"
```

## The runtime (Phase 1 — complete)

| Crate | What it is |
|---|---|
| **`hearth-brain`** | Legible, model-portable long-term memory — an LLM-wiki you can read, edit, and forget. |
| **`hearth-model`** | The neutral model router — local / API / subscription behind one trait. |
| **`hearth-substrate`** | Snapshot-first transactions — one-gesture `undo`. |
| **`hearth-mcp` · `hearth-mcp-fs`** | The capability fabric, as real **MCP** (JSON-RPC) servers. |
| **`hearthd`** | The mind: assemble context → plan (real model) → permission gate → act → audit. |

A steward that **converses with your machine, safely operates a real system with full undo, and
learns you in the open** — proven end to end on a live LLM.

## The design canon

The *why* is written down. Start with the vision, then the soul of the UI.

| Document | |
|---|---|
| [VISION-AND-ARCHITECTURE](docs/VISION-AND-ARCHITECTURE.md) | philosophy → the Hearth spec → purist architecture → roadmap |
| [UI-SOUL](docs/UI-SOUL.md) | the UI design language — **read before touching the UI** |
| [APP-MODEL](docs/APP-MODEL.md) | how apps work: adopt · strip · cohere |
| [BROWSER-INTEGRATION](docs/BROWSER-INTEGRATION.md) | the most-used surface (engine locked: Chromium/CEF via CDP) |
| [AGENT-TRUST-MODEL](docs/AGENT-TRUST-MODEL.md) | defeating prompt injection by design |
| [BROWSER-B1-ARCHITECTURE](docs/BROWSER-B1-ARCHITECTURE.md) | the first real web surface |

## Locked decisions

- **Soul** — The Hearth: ambient presence; *intent in, manifestation out*; glass-box sovereignty.
- **Model** — neutral: local / API / subscription are equal and swappable; no lock-in.
- **Build** — purist: own every layer we reasonably can. The kernel + driver tree stay upstream, but
  remain fully open and AI-patchable — that is the only boundary.

## Roadmap

[ROADMAP.md](ROADMAP.md). Phase 0 (soul + prototype) ✓ · Phase 1 (runtime) ✓ · **Phase 2 (the Hearth
shell + generative surfaces) → next**.

## Build

Rust stable, nothing else. The target is Arch Linux; the code is cross-platform and developed on
Windows-GNU (no Visual Studio). See [CONTRIBUTING.md](CONTRIBUTING.md) for the toolchain note.

## Contributing

Built in public — issues, design discussion, and PRs welcome. See **[CONTRIBUTING.md](CONTRIBUTING.md)**,
and please read [UI-SOUL](docs/UI-SOUL.md) before any UI change.

## License

**[GPL-3.0-or-later](LICENSE)** — copyleft, by design. The freedom Hearth gives you can't be taken
away downstream: no one can enclose it or ship a closed version. (The upstream kernel and drivers
keep their own licenses.)
