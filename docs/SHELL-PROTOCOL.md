# Shell ↔ Runtime Protocol

The contract between the **runtime** (`hearthd`, headless) and a **shell** (a client that
displays the steward and carries the owner's input). The mockup is one shell; the Phase‑3 native
Wayland compositor on Arch will be another. Both target this contract, so the compositor is built
against something stable rather than reverse‑engineered from the HTTP handlers.

**Source of truth:** [`crates/hearthd/src/protocol.rs`](../crates/hearthd/src/protocol.rs).
The shapes there are authoritative; this document is the prose.

**Version:** `1` (read live from `GET /api/protocol`).

## Transport

Transport‑agnostic JSON. Today: HTTP/1.1 over localhost, with the turn streamed as
**NDJSON** (one JSON object per line, flushed as it happens). A native shell on the same machine
can carry the identical messages over a Unix socket later — the message shapes do not change.

## The render contract

A manifestation is a **Surface** — a constrained, declarative tree of vetted **Node**s
(`heading`, `text`, `list`, `fields`, `tiles`, `actions`, `note`, `divider`, and the media nodes
`image` / `audio` / `video` / `document`). The shell renders the tree natively. It is **never**
handed raw markup or code — that constraint is the security boundary. Media nodes carry a **path**,
not bytes: the shell renders/plays the file; the model never moves the data. See
[`crates/hearthd/src/surface.rs`](../crates/hearthd/src/surface.rs).

## The flow

1. Shell → runtime: an **IntentRequest** (the owner's words, typed or transcribed).
2. Runtime → shell: a stream of **StreamEvent**s as the turn unfolds.
3. Owner interacts with a live surface: an `actions` button hands an intent back (another
   IntentRequest); editing a `note` streams a **SurfaceEdit** back (through the privacy floor).

## Runtime → shell (the event stream)

NDJSON from `POST /api/intent`. Each line is internally tagged by `event`:

| `event`    | payload | meaning |
|------------|---------|---------|
| `recalled` | `{recalled: [page,…]}` | memory pages pulled into context |
| `plan`     | `{planner, summary}` | the steward's plan for the turn |
| `step`     | `{step: StepResult}` | one gated, maybe‑snapshotted action just completed |
| `surface`  | `{surface: Surface}` | the manifestation to draw |
| `done`     | `{result: TurnResult}` | turn finished |
| `error`    | `{message}` | the turn failed |

A `step`'s `StepResult` carries `capability`, `tool`, `why`, `decision` (`auto`/`ask`/`forbid`),
`ran`, `snapshot` (the undo id, if any), and `result`.

## Shell → runtime (messages)

| message | endpoint | shape |
|---------|----------|-------|
| **IntentRequest** | `POST /api/intent` | `{intent: string, approve: bool}` → NDJSON event stream |
| **SurfaceEdit**   | `POST /api/surface/event` | `{node, kind, value}` → recorded edit |
| **ForgetRequest** | `POST /api/forget` | `{page}` → snapshot‑first, undoable |

Reads: `GET /api/brain` (curated memory pages), `GET /api/surface` (the reference surface — the
DSL made tangible), `GET /api/protocol` (this contract, machine‑readable).

## Not yet in v1 (the roadmap)

- an explicit **presence/phase** signal (the steward's six canonical states); today the shell
  infers presence from the event stream;
- **window/session lifecycle** beyond "a surface appears, lives, dissolves";
- a **native socket** transport;
- **voice** events (mic level, partial transcript) — voice in/out exist at the runtime edge
  (`hearthd listen` / `hearthd do --speak`); a GUI shell will carry them over this contract.
