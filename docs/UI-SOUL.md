# The Hearth — UI Soul & Design Language

*The feel we've found, written down so it survives every future change. If a new screen or
feature breaks one of these, it's probably wrong — fix the feature, not the rule.*

> Reference implementation: `the-hearth.html` (Aurora direction). This doc is the *why*; the
> file is the *how*.

---

## The nine tenets (the feel)

1. **You're inside it, not looking at it.** One inhabited field, not a screen of windows. The
   periphery carries the system (time · model · the steward *tending*); the centre is open.
   *No app-grid, no taskbar, no window chrome as the model.*
2. **Attention, not windows.** A surface breathes between **glance** (ambient, floating) and
   **taking the room** (immersive); the steward recedes the rest. You shift *attention*; you
   never push pixels, tile, or manage windows.
3. **A window is an agent session, not an app.** Titled by the **task**, with an agent inside.
   You **converse to steer** it (left), it **works in the workspace** (right), and it **reaches
   for tools** as the task needs them. One agent, one task.
4. **Intent in, manifestation out.** You hand off tasks (the intent bar, or by talking to an
   agent); the agent materialises the right tool. **You never launch an app.**
5. **The glass box.** Everything the agent does is visible and inspectable — the tool-trail
   chips, "show me the workings," "what is this page doing," the audit trail. Transparency is
   the default, not a panel you go find.
6. **Warm = the steward, cool = the machine.** The palette carries meaning. The ember is the
   living presence; the aurora is the ambient system. The light has a *source* — it brightens
   when the steward attends.
7. **Calm by default.** Motion is gentle and meaningful (materialise · dissolve · breathe ·
   recede), never decorative or attention-grabbing. Things *arrive* and *recede*; nothing
   demands. Generous negative space; restraint over density.
8. **Fractal consistency.** The screen mirrors the window: **dock › agent › workspace**, and
   inside each window **agent › workspace**. The same grammar at every scale.
9. **Voice lives in type.** The steward speaks in a **warm serif** (Fraunces); the **system** in
   a clean sans (Inter Tight); **code/data** in mono (JetBrains Mono). *Who is speaking* is
   encoded in the typeface — never mix them up.

---

## Tokens

- **Ground:** deep indigo-black (`--void #06080f`). Not flat dark-mode — a dark room with a fire in it.
- **Warm (steward):** ember `--warm #f4a259`, hot `--warm-hot #ffd9a8`, deep `--warm-deep #e2643a`.
- **Cool (machine/ambient):** `--cool #6ea0e0`, `--violet #8a7fe0`, `--teal #49c6c9`.
- **Ink/muted/faint:** `#ecf0f8 / #98a3bd / #5d6781`. **OK/warn:** teal / amber.
- **Glass:** `rgba(255,255,255,.05–.09)` + `backdrop-filter: blur(20–26px)`; edge highlight `rgba(255,255,255,.16)`.
- **Type roles:** serif = steward voice · sans = system/UI · mono = code/data. Never decorative.
- **Radius:** ~12–22px. **Motion:** 0.5–0.64s, `cubic-bezier(.2,.7,.2,1)`; breathe ~6s; always respect `prefers-reduced-motion`.

---

## Component grammar (reuse these — don't invent new chrome)

- **The field + aurora** — the living ground; warmth has a source (the presence).
- **The presence (ember)** — the steward, always there; breathes at rest, brightens when attending (`pulse()`).
- **The glass surface** — every materialised thing is a frosted-glass pane over the living light.
- **The session window** — `winbar` (● + task title + ⤢ – ×) · **agent panel** (left: status · thread · tool-chips · "message the agent") · **workspace** (right: the tool's content).
- **Tool-chips** — a read-only trail of the tools the agent opened (never a launcher).
- **The focus state** — "taking the room": one surface immersive, the rest receded; return via `Esc` / the **← the hearth** pill.
- **The dock** — a vertical left rail of parked (set-aside) sessions.
- **The tending feed** — top-right, the machine alive: timestamped work that ticks in on its own.
- **The intent bar** — the calm command line; spawns agent sessions.
- **Motions** — materialise (blur-in + rise), dissolve (blur-out + sink), breathe, recede, the thread's gentle "working…".

---

## How to extend without breaking the feel

- New capability? It's **a tool an agent opens inside a session** — not a new top-level app or icon.
- New information? It **arrives** in the tending feed or the briefing, or the agent surfaces it in
  a thread — it doesn't pop a modal or a notification that demands.
- New control? Prefer **talking to the agent** over adding a button. If a button is unavoidable,
  make it calm glass and contextual, never persistent chrome.
- New surface type? Compose it from the **glass + the type roles + the motion** above; it should
  look like it belongs in the same room.
- Configuration? **Configure by talking** (e.g. "warmer") — there is no settings app.

---

## Anti-patterns (never)

- A **taskbar, app grid, dock-of-all-apps, or app launcher.** (Sessions are tasks; the agent opens tools.)
- **Manual window management** — maximise/minimise buttons, tiling you arrange, overlapping you sort.
- **Modal dialogs / demanding notifications.** Use *arrive & recede*, the feed, or the thread.
- **A settings app.** Configure by talking; choices become Brain pages.
- **Breaking the type voices** (steward serif / system sans / code mono) or the **warm=steward /
  cool=machine** semantics.
- **Decorative motion** or density for its own sake. When in doubt, remove one accessory.

---

*Chanel's rule, for the road: before you ship a screen, look in the mirror and take one thing off.*
