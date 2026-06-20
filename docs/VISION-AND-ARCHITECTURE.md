# Hearth OS — Vision & Architecture

*An agentic operating system on an Arch Linux base, where the machine is a **steward**, not a tool.*

> **Status:** North-star design document (Phase 0). The interface soul — **The Hearth** — is validated by
> the interactive prototype in `index.html` (open it; it is the living mockup this document refers to).
> Working name: **Hearth OS**. The steward (the AI) is named by you, the first time you meet.

---

## 0 · First principles

For fifty years an operating system has been a **box of tools you operate**. You learn the menus, you
push the pixels, you do the work; the computer waits. Hearth OS inverts that: the computer becomes a
**counterpart you collaborate with** — one that can read and change every line of itself, act on your
behalf, learn you over time, and stay fully accountable to you.

Three decisions are fixed and shape everything below:

| Decision | Choice | Consequence |
|---|---|---|
| **Deliverable** | This document (north-star) + the prototype | Not yet building the real OS; we are committing the design. |
| **AI model** | **Neutral** | Local / API / subscription are equal first-class citizens. No default lean, no lock-in, swappable forever. |
| **Build** | **Purist** | We author every layer we reasonably can. The Linux kernel + driver tree is the only upstream boundary — and even that is fully open and AI-patchable. |
| **Soul** | **The Hearth** | Ambient steward; *intent in, manifestation out*; voice + direct manipulation preserved; glass-box sovereignty. |

**Design tenets** (the axioms everything is checked against):

1. **Intent in, manifestation out.** You express *what you want* in the most human channel available;
   the system manifests the *mechanism*.
2. **You stay sovereign.** The AI is steward, never master. Everything it does is explainable, inspectable,
   and reversible.
3. **The glass box, not the black box.** Because every line is accessible, the system can always *show*
   exactly what it did — as plain language, as a diff, as the literal code.
4. **Trust is earned, then granted.** Autonomy grows only as you allow it. Default to asking; graduate to acting.
5. **Calm by default.** Presence, not interruption. The system is a hearth you return to, not a stream of demands.
6. **No model is the master.** The OS is model-agnostic to its core; the intelligence is a swappable organ.
7. **It learns you, in the open.** The more you use it, the smarter it gets — and you can read, edit, and
   delete everything it has ever learned about you. (This is **the Brain**; see §2.8, §3.5.)

### Requirements traceability (your original asks → where they live)

| Your request | Addressed in |
|---|---|
| Agentic OS on Arch Linux | §3.1 Base & boot |
| Best UI/UX to interact with the AI | §1 Philosophy, §2 The Hearth |
| AI powers the whole system; **every line of code accessible** | §3.4 Capability fabric, §3.9 System-as-code |
| Installer bundles **all wifi/LAN drivers** for max compatibility | §3.2 Installer |
| **Model selection** — local / API / subscription | §3.3 The model layer |
| Once model + internet → **customize the OS to preference** | §2.7 Onboarding, §3.10 Customization engine |
| **The Brain** — persistent knowledge base; gets smarter the more you use it | §2.8, §3.5 |
| **Philosophical** mapping of human interaction → UI/UX | §1 |
| A **breakthrough** in how we communicate with the OS | §1.4, §2.1 |

---

## 1 · Philosophy — how humans have always collaborated, and what it teaches the interface

### 1.1 The fifty-year detour

The desktop metaphor (Xerox PARC, 1973 → today) is a metaphor of **solitary craft**: a workshop where
*you* are the worker and the computer is an inert bench of tools — files, folders, windows, a pointer
(WIMP). Every capability hides behind a menu you must memorize. It is brilliant, and it is a cage: it
assumes the human does all the thinking and all the doing, and the machine merely holds still.

### 1.2 The chatbot is not the answer

The obvious "agentic" reaction is to replace the desktop with a chat box. This is a **regression**. Chat
is a *keyhole*: linear, text-only, blind to dense information, hostile to precise control, with no spatial
memory and no peripheral vision. (2026 HCI research — the *"Keyhole Effect"* — shows chat interfaces fail
at anything dense or exact; Google's generative-UI work found users prefer AI-rendered interfaces ~83% of
the time over plain text.) A chatbot trades one cage for a smaller one.

### 1.3 What history actually shows

Strip away the screen and ask: *how have humans collaborated with a capable other across our whole
history?* The answer is remarkably consistent, and each pattern is a design principle.

| Across history, humans collaborated through… | …which becomes a design principle |
|---|---|
| **Conversation / oral culture** — speech, turn-taking, repair ("what do you mean?") | Natural language is the primary command layer; dialogue and repair, not rigid syntax. |
| **Pointing & showing (deixis, joint attention)** — "this one", "like this" | A *shared* workspace both parties can see and touch. Direct manipulation is preserved, not replaced. |
| **Apprenticeship** — learning by watching and being corrected | The system learns *your* way by observing you; you learn its reach by watching it work. Bidirectional, correctable. |
| **The trusted steward** (vizier, chamberlain, chief of staff) — delegated authority that stays answerable | Delegation *with accountability*: it acts, but reports back and can always be overruled. |
| **Writing / the letter** — externalized, durable memory | Persistent, legible memory; the relationship and context survive across time and sessions. (→ **the Brain**, §2.8) |
| **The hearth** — calm, always-there gathering place | Ambient presence over modal interruption. Calm technology. |
| **Ritual & rhythm** — liturgy, routine, the daily round | Dependable touchpoints (a morning briefing, an evening review, nightly consolidation) build trust. |
| **The agora / counsel** — ideas tested by exchange | A thinking partner that can present trade-offs and reason *with* you, not merely obey. |

### 1.4 The synthesis — the breakthrough

Hearth OS is not the desktop (manipulation of inert objects) and not the chatbot (a keyhole of text). It
is a **third thing**: *a shared, living workspace mediated by a trusted intelligence, drawing on the full
repertoire of how humans have always worked together.*

> **The breakthrough, in one line:** **Intent in, manifestation out.**
> You speak, type, point, or sketch a rough wish; the system *materializes the mechanism* — a running
> program, a freshly written tool, a purpose-built interface, a changed system config — interface shaped
> to intent rather than frozen into "apps." And because every line of the system is accessible to the AI,
> the manifestation is never a black box: it can always be shown as plain language, as a diff, or as the
> literal code, and undone with a single gesture.

This is what makes the steward relationship *safe enough to actually use*. A vizier you cannot audit is a
tyrant; a vizier whose every act is transparent and reversible is a gift. **Sovereignty (the glass box +
universal undo) is the precondition that turns delegation from dangerous into liberating.** The same logic
extends to memory: a steward who remembers you in a ledger you can read and correct is trustworthy; one
who remembers you in a black box is not (§2.8).

---

## 2 · The Hearth — experience specification

The canonical experience is the six states in the prototype. Each is described below as a spec.

### 2.1 The core loop

```
  you express intent  ─►  the steward understands & plans  ─►  it MANIFESTS
   (speech/text/point)     (with the Brain's memory of you)    (program · tool · surface · config)
                                                                      │
        you remain sovereign  ◄──────────────────────────────────────┘
   (glass box: explain / diff / code · one-gesture undo · grant or withhold autonomy)
                                                                      │
        the Brain learns  ◄───────────────────────────────────────────┘
   (the outcome is consolidated into legible, editable memory — §2.8)
```

The loop is the whole OS. There is no separate "launch an app, find the menu, operate the tool" detour —
those are *implementation details the steward handles*, surfaced only when they help you. Each turn both
*draws on* the Brain and *feeds* it.

### 2.2 The six canonical states (see `index.html`)

1. **The first meeting** — first boot after install. No tutorial; a *relationship* begins. You choose how
   to talk (type / speak / both) and you name the steward. (State of mind: meeting a new chief of staff.)
2. **The hearth at rest** — the home. No app grid, no taskbar. A breathing **presence** indicator, a
   glanceable **briefing**, and a single line for intent. Everything else is summoned and then dissolves.
3. **Intent → manifestation** — the centerpiece. You say *"make a shared album of my Japan trip"*; the AI
   **builds the exact interface the task needs** (a photo surface with share controls), which you and it
   can both touch. Different intent → different interface (disk cleanup, email drafts, etc.).
4. **The glass box** — "what will this do?" reveals the plan in plain language; **"as code"** reveals the
   literal commands, each line clickable for "why this?". A snapshot is always taken first.
5. **Voice** — the oral mode. Voice leads; a surface appears *only when it helps*. This is precisely why
   the Hearth is not a chatbot: speech and rich surfaces coexist.
6. **The steward's return** — a ritual review of what happened while you were away, with a **trust dial**:
   "today I acted on 3 routine things, asked first on 1," and "I consolidated today into the Brain — here's
   what I learned." (Autonomy and memory are both reviewed here.)

### 2.3 Modalities (one fluid medium, not separate "modes")

- **Language** (text + voice) — the command layer. Local STT/TTS so it works offline (§3.7).
- **Generative surfaces** — the response layer (§2.4). The system's primary way of "showing."
- **Direct manipulation** — pointing, dragging, touching. Always available; the fastest channel for many
  tasks; the AI sees what you touch (joint attention).
- **Ambient presence** — the calm, peripheral channel: the breathing dot, the briefing, gentle nudges.

### 2.4 Generative surfaces (the heart of the implementation)

A *surface* is an **ephemeral, intent-shaped interface** the steward materializes, lives for a task, and
dissolves. Specification:

- **What the model emits is *not* arbitrary code.** It emits a **constrained declarative UI description**
  (a "Surface DSL" — a small, safe, composable component grammar: panels, grids, media, controls, charts,
  forms, lists, bound to data and to capability calls). This is the security boundary: the model composes
  from a vetted component library; it cannot inject raw executable UI. (Mirrors the Google finding that UI
  generation is an *emergent* capability — no special model training needed — while keeping it sandboxed.)
- **The shell renders it natively** (§3.7) with the materialize/dissolve motion (blur-in, settle, fade).
- **Bidirectional & live.** Both you and the steward can manipulate a live surface; edits stream back as
  structured events the model understands ("you removed 4 photos").
- **Lifecycle:** summoned → live → dissolved (or *pinned* if you want to keep it). Surfaces are cheap and
  disposable by design — the opposite of installing an app.

### 2.5 The glass box & sovereignty

Every consequential action is presented at three zoom levels, on demand:
- **Plain language** — "copy 312 photos into a new album; share a private link with Mom & Dad; nothing
  else leaves the machine."
- **Diff / plan** — the concrete before/after and the ordered steps.
- **Literal code** — the exact commands/calls, each annotated with "why this?".

And every action is a **transaction**: a btrfs snapshot is taken first, so *one gesture undoes anything*
(§3.8). This is non-negotiable — it is what makes a root-capable AI trustworthy.

### 2.6 Trust & autonomy (apprenticeship model)

Autonomy is a **dial the user controls**, per domain:
- **Ask first** (default) — propose, wait for approval.
- **Act & report** — do routine things, summarize in the next briefing.
- **Just handle it** — for narrow, explicitly granted domains (e.g., "always apply safe security updates").

Trust is earned by track record and granted by you ("a little more rope"). It can always be revoked. The
steward never silently expands its own authority — autonomy changes are themselves logged, glass-box events.

### 2.7 Onboarding & living customization

The moment a **model is chosen and the network is up** (the original requirement), the steward begins the
*relationship*: a short, conversational "getting to know you" (the first pages of the Brain), then it
**shapes the OS to you** — theme and density, which tools to install (pacman/AUR/Flatpak), workflows,
keybindings, services, privacy posture. This never stops: customization is just ordinary use of the loop
("make the text warmer", "I keep doing X, can you handle it?"), and every change is a reversible transaction.

### 2.8 The Brain — memory that learns you, in the open

> *"The more I use it, the smarter it gets about how I use the OS."* This is the Brain. It is built on
> **Andrej Karpathy's "LLM wiki"** idea and his **"system-prompt learning"** ("notes to self") — chosen
> deliberately over an opaque vector database, because **legible memory is the glass-box tenet applied to
> what the system knows about you.**

What you experience:
- **A "what do you know about me?" view** — a small wiki of **plain-language pages** the steward keeps
  about *you*: your preferences, your habits and rhythms, your people, your projects, your tools and
  workflows, and **lessons it has learned about operating this machine for you**. You can **read, edit, or
  forget** any of it. Nothing about you is hidden in a black box.
- **It gets smarter with use, not with retraining.** Every interaction quietly teaches it ("when you say
  'tidy up' you mean group by type and never delete"; "you prefer concise replies"; "escalate code tasks
  to the bigger model"). It writes these down as explicit notes and refines them by what actually works.
- **A consolidation ritual.** While you rest, the steward **consolidates** the day's raw activity into
  these pages — like sleep turning experience into memory. The evening review shows what it learned (§2.2).
- **It stays with you across model swaps.** Because the Brain is *externalized text*, not weights, you can
  switch from a local model to an API to a subscription and **the OS still knows you** (§3.3 + §3.5). The
  intelligence is swappable; the relationship is not.

This is the modern, legible form of *the letter* (§1.3): durable, human-readable memory that lets the
relationship deepen instead of resetting every session. Architecture in §3.5.

### 2.9 Accessibility & the human floor

Because the interface is *generated*, it can be regenerated for the person: larger, higher-contrast,
voice-only, screen-reader-native, simplified. Accessibility is not a bolt-on; it is a re-render. The voice
modality also means the OS is fully operable with no screen at all.

---

## 3 · System architecture (purist)

### 3.1 The layer stack

```
┌──────────────────────────────────────────────────────────────────────┐
│  THE HEARTH SHELL  — presence · briefing · intent · generative surfaces│  ours
│  + voice (STT/TTS, wake word)                                          │
├──────────────────────────────────────────────────────────────────────┤
│  COMPOSITOR (Wayland, Rust/smithay) — renders surfaces natively        │  ours
├──────────────────────────────────────────────────────────────────────┤
│  hearthd — THE AGENT RUNTIME ("the mind"): intent→plan→act, context    │  ours
│  assembly, autonomy/trust engine, audit                                │
├──────────────────────────────────────────────────────────────────────┤
│  THE BRAIN — LLM-wiki:  raw activity log ─► compiled markdown wiki ─►   │  ours
│  schema + strategies   ·  legible · editable · model-portable          │
├───────────────┬────────────────────────────┬─────────────────────────┤
│ MODEL ROUTER  │  CAPABILITY FABRIC (MCP)    │  SOVEREIGNTY SUBSTRATE   │  ours
│ local/API/sub │  every subsystem as a tool  │  snapshots·tx·perms·log  │
├───────────────┴────────────────────────────┴─────────────────────────┤
│  SYSTEM-AS-CODE  — git-tracked config, declarative state, self-index   │  ours
├──────────────────────────────────────────────────────────────────────┤
│  Arch base · systemd · btrfs · pacman          (curated, ours to wire) │  base
├──────────────────────────────────────────────────────────────────────┤
│  LINUX KERNEL + DRIVER TREE + firmware     ── the only upstream layer ─│  upstream
│  (not rewritten — but fully open, readable, and patchable by the AI)   │  (accessible)
└──────────────────────────────────────────────────────────────────────┘
```

**Purist boundary, stated honestly:** "own every line we can" means we author the installer, compositor,
shell, surface engine, agent runtime, the Brain, model router, capability fabric, and sovereignty
substrate. We do **not** rewrite the Linux kernel or hardware drivers — that is infeasible and would
*reduce* compatibility, which contradicts your max-compatibility goal. The kernel/driver layer stays
upstream, but it is 100% open source the AI can **read, explain, and patch** — which fully satisfies
"every line of code on the OS is accessible by the AI."

### 3.2 The installer — maximum hardware compatibility

A custom **archiso**-built ISO with a **conversational/guided installer** (TUI-first so it works before any
model is present; the steward takes over post-network).

**Bundling every wifi/LAN driver (the core compatibility requirement):**
- Ship the full **`linux-firmware`** set (covers most Intel/Atheros/Realtek/Qualcomm/Mediatek radios in-kernel).
- Add the out-of-tree drivers that aren't in `linux-firmware`, built from AUR and pinned **against the ISO's
  exact kernel version**: e.g. `broadcom-wl` / `broadcom-wl-dkms`, `rtl8821ce-dkms`, `rtl88xxau-dkms`,
  `rtw89-dkms`, `8821au`, `aqc111`, plus common USB-ethernet/tethering modules.
- **DKMS variants** included so a driver rebuilds itself against whatever kernel the installed system runs
  (kernel-version independence after updates).
- **Embed a local offline pacman repo on the ISO** (built with `repo-add`) holding all of the above. This
  breaks the chicken-and-egg problem: you can bring up wifi *before* you have internet, entirely offline.
- A **driver-sweep** step probes PCI/USB IDs, loads the right module, and reports what came up (the
  prototype's installer scene shows this).
- Licensing note: some firmware/drivers (e.g. `broadcom-wl`) have redistribution terms — the build pipeline
  tracks licenses and, where redistribution isn't permitted, fetches at install time with a clear notice.

Then, still in the installer: **btrfs** layout with subvolumes for instant rollback (§3.8), **model
selection** (§3.3), **network connect**, and handoff to the **first meeting** (§2.2).

### 3.3 The model layer — neutral by construction

A **model router** presents one model-agnostic interface to the whole OS; the intelligence is a swappable
organ. Three equal paths (no default lean):

- **Local** — a local runtime (llama.cpp / vLLM / an Ollama-class server) serving GGUF/where-appropriate
  models. The installer **detects hardware** (GPU VRAM, system RAM, CPU, emerging NPUs) and offers models
  that actually fit, with honest trade-offs. Private, offline-capable.
- **API** — bring-your-own-key for any provider (Anthropic, OpenAI, Google, …) or a self-hosted gateway.
  Keys live in the secrets vault (§3.8).
- **Subscription** — sign in to a plan you already pay for.

Internally, all three are normalized behind a single **OpenAI/MCP-compatible gateway interface**, so every
component above the router is identical regardless of which path is active. The router supports **policy
routing** (e.g., "keep anything touching personal files on the local model; escalate hard reasoning to the
API") and graceful **degradation** (fall back to local when offline). Switching models is a settings change,
never a reinstall — **and the Brain (§3.5) carries your accumulated knowledge across the switch.**

### 3.4 The capability fabric — exposing every subsystem to the AI (MCP)

This is *how* "the AI powers the whole system." Every OS subsystem is exposed as **MCP** tools (Model
Context Protocol — the Linux Foundation standard since Dec 2025; the *"computer as MCP server"* pattern).
The steward acts only through this fabric — which means every action is typed, permissioned, and auditable.

Representative capability servers:

| Server | Exposes | Example tools |
|---|---|---|
| `fs` | filesystem | read, write, search, move (snapshot-guarded) |
| `pkg` | pacman / AUR / Flatpak | search, install, remove, list, pin |
| `svc` | systemd | status, start/stop, enable, journald query |
| `net` | networking | scan wifi, connect, VPN, firewall |
| `settings` | declarative system state | get/set theme, locale, power, privacy |
| `proc` | processes & resources | list, inspect, limit, kill |
| `display` | compositor & surfaces | render surface, capture screen context |
| `code` | the system's own source | read/patch any component (§3.9) |
| `brain` | the Brain / knowledge base | recall, remember, edit, forget, compile (§3.5) |
| `secrets` | the vault | request/use a secret (never raw-exfiltrate) |

A local **MCP gateway** federates these behind one endpoint with uniform auth, rate-limits, and the
permission checks of §3.8. Third-party MCP servers (the 10,000+ ecosystem) can be added — sandboxed and
permissioned like everything else.

### 3.5 The Brain — the knowledge base (an LLM-wiki)

The Brain is the system's long-term memory and the reason it gets smarter with use. It implements
**Karpathy's LLM-wiki** ("the LLM as a *compiler* that turns raw sources into a curated wiki") plus
**system-prompt learning** (explicit, refinable "notes to self"). It is **local-first, owned by you, and
legible** — the opposite of an opaque embedding store.

**Three layers (the LLM-wiki structure):**

1. **Raw sources — ground truth, never rewritten.** The append-only **activity log** from the sovereignty
   substrate (§3.8): every interaction, action, outcome, and correction — plus the user's own files and
   dotfiles. The Brain *reads* these but never edits them; they are the immutable record. (This reuses the
   audit trail we already need — the audit log *is* the raw-source layer.)
2. **The wiki — compiled, curated markdown.** A **git-tracked directory of plain-markdown pages** the
   steward compiles from the raw log and keeps current: `you.md`, `rhythms.md`, `people/*.md`,
   `projects/*.md`, `workflows/*.md`, `devices.md`, and `lessons/*.md`. Cross-linked, revision-tracked.
   This is the long-term memory the model reads each turn. Per the LLM-wiki principle, knowledge is
   **compiled once and maintained**, not re-derived on every query (the key difference from vanilla RAG).
3. **The schema — procedural memory.** A config file (Hearth's analogue of `CLAUDE.md`) encoding the
   *rules*: how pages are merged, how contradictions are resolved, what gets promoted from raw log to wiki,
   what is retained vs. retired, and what is **never** remembered (privacy redaction). Editing the schema
   changes *how the OS learns*.

**The strategies layer (system-prompt learning / "notes to self").** Beyond facts about you, the Brain
accumulates **explicit, machine-specific strategies** — e.g. *"after a kernel update, rebuild the
rtl8821ce DKMS module before reconnecting wifi"*, *"this user's 'tidy up' = group by type, never delete"*,
*"the local model has failed twice on Rust refactors — route those to the API."* These live in
`lessons/`, are consulted before acting, and are refined by success/failure — a higher-bandwidth feedback
channel than fine-tuning, and one you can read and correct.

**The consolidation ritual ("sleep").** During idle time the steward runs a **compilation pass**: read new
raw-log entries → update/merge the relevant wiki pages and lessons → resolve contradictions per the schema
→ commit to git. The evening review (§2.2) surfaces "what I learned today." This is calm by design (it
happens while you're away) and cheap enough for local models (it's batch, not per-keystroke).

**Retrieval (context assembly).** Each turn, `hearthd` (§3.6) pulls the relevant wiki pages and lessons
into the model's context. Because the corpus is curated prose, a light **semantic index over pages**
(hybrid keyword + embedding) is enough to select the right few pages — embeddings are an *index into*
legible memory, never the memory itself.

**Sovereignty & privacy (non-negotiable, mirrors the glass box):**
- Plain-markdown and **fully user-readable/editable** via the "what do you know about me?" view (§2.8).
- **`forget` is real** — removes the page *and* redacts the relevant raw-log entries; revision history in git.
- **Local-first and encrypted at rest**; the schema's redaction rules keep designated secrets out entirely.
- **Model-portable:** because it's text, swapping the model (§3.3) preserves everything. The Brain is the
  durable self of the OS; the model is just the current mind reading it.

### 3.6 The agent runtime — `hearthd` ("the mind")

A privileged userspace daemon that is the seat of the steward:
- **Intent → plan → act** orchestration, with explicit *plan objects* the glass box renders.
- **Context assembly** — pulls the right Brain pages/lessons (§3.5), current screen/world state, and
  capability schemas into the model's context for each turn (the OS's analogue of an attention manager).
- The **autonomy/trust engine** (§2.6) and the hooks that feed outcomes back to the Brain for consolidation.
- **Audit** — every model decision, tool call, and outcome is logged to the append-only trail (which is
  also the Brain's raw-source layer, §3.5).
- Designed so the *reasoning* lives in the swappable model, while the *judgment, permissions, and memory*
  live in code and data we own and the user can inspect.

### 3.7 The compositor & shell (Rust)

- **Compositor:** a **Wayland** compositor written in Rust on **smithay** (purist: we own the display layer
  end-to-end). It natively renders the Surface DSL, owns the materialize/dissolve motion, exposes screen
  context to the `display` capability, and runs ordinary Wayland apps when needed (the steward can still
  open a browser or editor — it just isn't the *primary* metaphor).
- **The Hearth shell:** presence, briefing, the intent line, surface host, the "what do you know about me?"
  Brain view, and the trust/review rituals.
- **Voice:** local **STT** (whisper.cpp-class) + **TTS** (piper-class) + wake word, so the oral modality
  works offline and privately.
- **Surface renderer + component library:** the vetted, safe widget grammar the model composes against.

### 3.8 The sovereignty & safety substrate

The most important subsystem, because the AI has root-equivalent reach:
- **btrfs snapshots as transactions.** Every mutating action snapshots first; the shell offers one-gesture
  undo; a timeline lets you roll the *whole system* back. (Our own transaction manager, not a generic tool.)
- **Capability permissions.** Per-tool, per-domain policies (auto / ask / forbid), tied to the autonomy dial.
- **Dry-run & preview.** High-impact actions can be simulated and shown before commit.
- **Secrets vault.** Keys/credentials are used by capabilities but never handed to the model in the clear.
- **Append-only audit trail.** Every action reconstructable — and reused as the Brain's raw-source layer (§3.5).
- **Sandboxing.** Third-party MCP servers and generated tools run least-privilege and confined.
- **Prompt-injection defense.** Because surfaces and tools are structured (not free code), and capabilities
  are permissioned, a malicious document can't silently escalate; risky cross-trust actions require
  confirmation.

### 3.9 "Every line accessible" — system-as-code

- The system's **own source and configuration are git-tracked** (an `etckeeper`-style layer over `/etc`
  plus a declarative state model for the whole machine), so every change has history and can be reverted.
  The Brain's wiki lives in this same git world (§3.5), so memory inherits diff/rollback for free.
- A **code index** over the running system lets the steward *find and reason about* any component — including
  its own shell, compositor, runtime, and Brain — via the `code` capability.
- **Live introspection & self-patching:** the steward can read the source of what's currently running,
  propose a patch through the glass box, snapshot, apply, and roll back if it misbehaves. The OS can, with
  your consent, *improve itself.* This is the literal meaning of "every line of code is accessible to the AI."

### 3.10 The customization engine

Post-install personalization (§2.7) is implemented as ordinary capability calls (`pkg`, `settings`,
`display`, `code`, `brain`) driven by the onboarding conversation and continuous use — all transactional and
reversible. There is no separate "settings app"; configuring the OS *is* talking to it, and your choices
become Brain pages so they persist and generalize.

### 3.11 Component ownership map

| Component | Ownership | Likely tech |
|---|---|---|
| Installer + offline driver repo | **ours** | archiso, Rust/Python TUI, pacman repo tooling |
| Compositor | **ours** | Rust + smithay (Wayland) |
| Hearth shell + surface renderer | **ours** | Rust + GPU UI; Surface DSL |
| Agent runtime `hearthd` | **ours** | Rust |
| **The Brain (LLM-wiki)** | **ours** | git-tracked markdown + schema + semantic index |
| Model router | **ours** | Rust; OpenAI/MCP-compatible interface |
| Capability fabric + gateway | **ours** | Rust; MCP (JSON-RPC 2.0) |
| Sovereignty substrate | **ours** | Rust over btrfs |
| System-as-code layer | **ours** | git + declarative state |
| Voice (STT/TTS/wake) | ours (integrating proven engines) | whisper.cpp / piper class |
| Local model runtime | integrated | llama.cpp / vLLM class |
| Base OS, init, fs, packages | curated base | Arch, systemd, btrfs, pacman |
| **Kernel + drivers + firmware** | **upstream** (open, AI-patchable) | Linux, linux-firmware, DKMS |

---

## 4 · Risks & failure modes (named honestly)

| Risk | Mitigation |
|---|---|
| **Root-capable AI doing harm** | Everything is a snapshot transaction; permissions + autonomy dial; dry-run; audit; glass box before consequential acts. |
| **Model unreliability / hallucinated actions** | The model proposes; `hearthd` + permissions + preview + undo dispose. Judgment lives in owned code, not the model. |
| **Local-model latency / capability gaps** | Policy routing escalates hard tasks; honest hardware-fit guidance at install; degrade gracefully offline. |
| **Generative-UI / prompt-injection attacks** | Surfaces are a constrained DSL over a vetted library (no raw code); structured tool calls; cross-trust actions need confirmation; sandboxing. |
| **Brain learns the wrong thing / drifts / over-generalizes** | Memory is legible markdown you can read & correct; schema-governed merge/retire with contradiction resolution; git revision history; "act & report" surfaces new lessons for approval. |
| **Brain privacy** | Local-first, encrypted at rest, schema redaction of secrets, real `forget` (page + raw-log redaction), never silently synced. |
| **Driver bundling — size & licensing** | linux-firmware + DKMS in offline repo; license tracking; fetch-at-install where redistribution is restricted. |
| **Scope / it's an entire OS** | Phased roadmap (§5) yields a usable, daily-drivable system *early*, before the purist replacements land. |
| **Trust UX (too eager / too timid)** | Default to "ask"; autonomy is user-granted, per-domain, revocable, and itself logged. |

---

## 5 · Phased roadmap

A purist OS is a multi-year arc. The sequence is chosen so you have something **real and usable early**,
then progressively replace hosted scaffolding with owned components.

- **Phase 0 — Prototype (done).** Validate the soul + the Brain concept. → *The Hearth, chosen.* (`index.html`)
- **Phase 1 — The mind, the fabric, and the first Brain.** `hearthd` + neutral model router + MCP capability
  servers (`fs`, `pkg`, `svc`, `net`, `settings`, `proc`, `brain`) + the sovereignty substrate (snapshots,
  permissions, **append-only log = the Brain's raw layer**) + a **basic Brain** (manual + on-demand
  compilation of the log into the markdown wiki; legible "what do you know about me?" view). Runs headless/CLI.
  **You can now:** converse with your machine, have it *safely* operate a real system with full undo, and
  read/edit what it has learned about you.
- **Phase 2 — The Hearth shell, surfaces, and the learning Brain.** The shell, Surface DSL + renderer,
  voice, the six states — plus the Brain's **automatic consolidation ritual**, the **strategies/lessons**
  layer, and retrieval (semantic index over pages). First hosted as a Wayland client (usable immediately).
  **You can now:** live in the Hearth, and feel it get smarter the more you use it.
- **Phase 3 — The compositor (purist).** Replace the host compositor with our smithay-based one; native
  surface rendering and deep `display` access.
- **Phase 4 — The installer & distro.** Custom archiso, the offline all-drivers repo, neutral model
  selection, first-boot meeting, the customization engine.
  **You can now:** install Hearth OS on bare metal, online anywhere, out of the box.
- **Phase 5 — Self-evolution & hardening.** System-as-code self-patching, deeper trust + Brain governance,
  multi-user (per-user Brains), accessibility re-rendering, security audits, performance.

---

## 6 · Open questions (to decide before/within Phase 1)

1. **Primary implementation language** — Rust is assumed for systems layers; confirm, and decide the
   scripting/tooling language for capabilities.
2. **Reference local model(s)** to target first, and the minimum hardware tier we promise.
3. **Surface DSL shape** — adopt/extend an existing declarative UI grammar vs. design our own.
4. **Brain schema & policy** — the page taxonomy, the retain/forget/redaction rules, and how aggressively
   the consolidation ritual promotes raw-log entries into lessons.
5. **The steward's default persona & voice** — tone, naming ritual, how much personality.
6. **Distro positioning** — a true independent distro vs. an installable "experience layer" over Arch for
   the first public release.

---

## 7 · What the prototype already proves

Open `index.html` and you have a working argument for every core claim above: presence over app-grid,
intent→manifestation, surfaces that reshape to the task, drag-to-edit shared objects, the glass-box "as
code" view, universal undo, voice that coexists with surfaces, a steward that reports back with a trust
dial — and **the Brain view** ("what do you know about me?"): legible, editable, forgettable pages plus a
machine-specific *lesson*, with a note that it's compiled from your activity and stays with you across
model swaps. The rest of this document is the plan to make that real, line by owned line.

---

### Sources for the Brain's design
- Karpathy's **LLM Wiki** — ["What Is an LLM Knowledge Base? How Karpathy's Wiki Architecture Works"](https://www.mindstudio.ai/blog/what-is-llm-knowledge-base-karpathy-wiki-architecture) and ["Karpathy's LLM Wiki as Agent Memory"](https://aaif.io/blog/karpathys-llm-wiki-as-agent-memory/)
- Karpathy on **system-prompt learning** (the "third paradigm" / "notes to self") — [original post](https://x.com/karpathy/status/1921368644069765486)
