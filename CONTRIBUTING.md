# Contributing to Hearth OS

Hearth is built in public. Issues, design discussion, and pull requests are all welcome — including
"this breaks a principle" critiques.

## Read first
- **[docs/VISION-AND-ARCHITECTURE.md](docs/VISION-AND-ARCHITECTURE.md)** — the north star.
- **[docs/UI-SOUL.md](docs/UI-SOUL.md)** — required before any UI change. If a change breaks one of the
  nine tenets, it's probably the change that's wrong.
- The **locked decisions** (soul, neutral model, purist build) in the README are settled defaults —
  raise an issue before re-litigating them.

## Build & run
Rust stable, nothing else.
```sh
cargo build
cargo run -p hearthd -- init
cargo run -p hearthd -- do "remember I prefer concise replies" --yes
```

**Run the shell (the live UI).** Serve the runtime and open the Aurora shell in Chrome/Edge:
```sh
cargo run -p hearthd -- serve --ui mockup/the-hearth.html
# then open http://127.0.0.1:7878
```
Type an intent and the real steward answers: the tool-trail streams live, every action is gated and
snapshot-undoable (`hearthd undo`), the Brain view shows what it has learned, and *"show me a surface"*
manifests a generated surface. With no model it runs on the offline heuristic floor; set
`HEARTH_MODEL_URL` / `HEARTH_MODEL_KEY` / `HEARTH_MODEL_NAME` (any OpenAI-compatible endpoint) to bring
the steward fully alive.

**Windows-GNU note.** Development uses the self-contained GNU host (`x86_64-pc-windows-gnu`) — no
Visual Studio required. We deliberately keep the build lean and sidestep the `windows-sys`/`dlltool`
and Rust-TLS C-toolchain pitfalls (hence the tiny `clock.rs` instead of `chrono`, trimmed `clap`
features, and a `curl`-based model backend). Keep new dependencies lean and cross-platform — the
target is Arch / Linux.

## Norms
- Match the surrounding code's style and the design language.
- A new capability is an **MCP tool / server**, not a new app (see [docs/APP-MODEL.md](docs/APP-MODEL.md)).
- Keep memory **legible**, actions **gated + snapshotted**, and the **glass box** intact — these are
  load-bearing, not optional.
- Prefer the smallest change that solves the problem.

## License & sign-off
By contributing, you agree your contributions are licensed under **GPL-3.0-or-later** (the project
license). Please sign off your commits — `git commit -s` (the Developer Certificate of Origin).
