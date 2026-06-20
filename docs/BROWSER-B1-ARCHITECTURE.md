# Hearth OS — Browser B1 Architecture

*The first real web surface: an embedded Chromium page wearing the Hearth's chrome, in Rust.*

> **Status:** Build sketch for **B1** in `BROWSER-INTEGRATION.md` §12. Engine is locked to
> Chromium/Blink via CEF, driven over CDP (§3.1). This is buildable on the current dev box; the
> Wayland/compositor integration is the later port.

---

## 1 · Scope

**B1 delivers:** a real embedded Chromium page composited into a Hearth glass surface, with *our*
chrome, real navigation, request **blocking + glass-box** telemetry, and **reader + summarize** via
the neutral model. Plus the **`web` capability skeleton over CDP** — the seam the steward will later
drive.

**B1 explicitly excludes:** agency (the steward acting on pages) — that's B2/B3 and is gated by
`AGENT-TRUST-MODEL.md`. B1 is the *human browses, steward observes/assists* slice.

---

## 2 · Layer stack

```
┌───────────────────────────────────────────────────────────────┐
│  HEARTH CHROME (ours)   nav · address-as-intent · ✦ask ·       │  shell
│  reader/summarise/glass-box · materialise/dissolve             │
├───────────────────────────────────────────────────────────────┤
│  COMPOSITOR  wgpu  — draws the page texture inside a glass      │  ours
│  surface; winit (dev) → smithay/Wayland (target)               │
├───────────────────────────────────────────────────────────────┤
│  ENGINE  CEF (Chromium) in OFF-SCREEN RENDER mode              │  embedded
│  OnPaint → GPU texture · input injection · multi-proc isolation │
├──────────────────────────────┬────────────────────────────────┤
│  CONTROL  CDP via chromiumoxide │  NETWORK  request intercept   │  ours
│  → the `web` capability (MCP)   │  → adblock-rust + glass-box   │
└──────────────────────────────┴────────────────────────────────┘
        the page reaches the steward ONLY through `web` (trust boundary)
```

---

## 3 · Components

### 3.1 Engine — CEF, off-screen rendering (OSR)

- **Crate:** `cef` / `cef-rs` (Rust bindings to the Chromium Embedded Framework).
- Initialize CEF (browser process + per-site render processes → isolation) and create the browser in
  **windowless / OSR** mode, so CEF hands us **rendered frames** instead of owning a native window —
  letting us composite the page into our own surface with our chrome and motion.
- Implement the CEF handlers we need:
  - `OnPaint` / `OnAcceleratedPaint` — receive the page bitmap or, preferred, a **shared GPU texture**
    handle (zero-copy). Upload/import into a `wgpu` texture.
  - **Input injection** — `SendMouseClickEvent` / `SendMouseMoveEvent` / `SendMouseWheelEvent` /
    `SendKeyEvent` for the human-drives path (§3.3).
  - Load/lifecycle, dialog/permission handlers (route to our chrome), `CefResourceRequestHandler`
    for the network layer (§3.5).

### 3.2 Compositor — wgpu

- **Crates:** `wgpu` + `winit` (dev) → `smithay` (Wayland target). Same wgpu drawing either way.
- Draw the field (the Aurora; can graduate from canvas to a wgpu shader), then the **page texture as
  a quad inside a glass surface frame** with depth, drop shadow, and the materialise/dissolve
  animation. The web surface is just another floating surface in the inhabited field.

### 3.3 Input routing

- Pointer/keyboard events over the **page region** → translate to page coords → inject into CEF.
- Events over **our chrome** (nav, address, mediation bar, title-drag) → handled by the shell.
- One rule: the human drives the page through injected input; the **steward drives it only through
  the `web` capability** (§3.4) — never a back channel.

### 3.4 Control — the `web` capability over CDP

- **Crate:** `chromiumoxide` (async Rust **CDP** client). CEF exposes a DevTools/CDP endpoint; we
  attach to the *same* browser the human sees.
- Wrap CDP behind the stable `web.*` MCP tools (`BROWSER-INTEGRATION.md` §3.2). **B1 implements the
  read-only slice** the chrome needs: `navigate`, `read` (structured content), `snapshot`. The
  acting tools (`act`, `fill`, `extract`, `watch`) are **stubbed** — wired in at B2/B3 behind the
  trust model.
- **Security note:** the CDP endpoint is a privileged control plane. Bind to **localhost only**,
  require a per-session token, and treat it as T0 — never reachable from page content. (This is the
  isolation `AGENT-TRUST-MODEL.md` §4.5 relies on.)

### 3.5 Network — blocking + glass-box

- **Crate:** `adblock` (adblock-rust, Brave's Rust engine) — EasyList-format filter matching.
- Intercept requests (CEF `CefResourceRequestHandler` and/or CDP `Network` domain). For each:
  match against filter lists → **block** trackers/ads; record **origins contacted, blocked count,
  cookies, what would leave the machine** → feed the **glass-box** panel ("what is this page doing").
- Privacy is the **default**, not a toggle.

### 3.6 Content pipeline — reader & summarize

- **Extract:** run a readability pass — inject a Readability-class script via CDP `Runtime.evaluate`
  (or a Rust DOM pass) → clean title/main-content/structure.
- **Reader surface:** render the cleaned content in the Aurora reader style (Surface DSL) — the
  look-and-feel "match," by owning the re-render.
- **Summarise:** send the cleaned text to the **neutral router** (`hearth-model`) → the gist. Even at
  B1, the summariser is a **quarantined reader** (no tools, output is tainted) — the trust model's
  cheapest layer, in from day one.

### 3.7 Chrome — our UI

The slim nav (‹ › ⟲), the **address-as-intent** field, **✦ ask**, and the mediation bar
(reader / summarise / glass-box) — drawn by the shell, matching `the-hearth.html`.

---

## 4 · Crates

| Crate | Role |
|---|---|
| `cef` / `cef-rs` | Embed Chromium (OSR), handlers, input injection |
| `wgpu` + `winit` → `smithay` | GPU compositing; dev window → Wayland surface |
| `chromiumoxide` | CDP client → the `web` capability |
| `adblock` (adblock-rust) | Filter-list blocking + glass-box data |
| `hearth-model` *(ours)* | Neutral router for reader/summarise |
| `tokio`, `serde`/`serde_json` | Async runtime; CDP/config messages |
| `hearth-brain` *(ours, later)* | Feed history/extracts into memory (B4) |

---

## 5 · Process & threading

- CEF has strict threading/lifecycle rules and its own message loop. Integrate it with our event
  loop (multi-threaded message-loop mode, or pump CEF in our loop) — **budget real time here; it's
  the finicky part.**
- The CDP client is **async (tokio)**, off the UI thread; commands marshalled to the engine.
- **Trust boundary (enforced structurally):** page content → renderer process (sandboxed) →
  frames/requests to us; the steward → `web` capability → CDP → engine. No other edges.

---

## 6 · Data flow

```
 human input ─► shell ─► CEF(render proc) ─► OnPaint(texture) ─► wgpu ─► screen
 steward ─► web cap ─► CDP ─► CEF                                 ▲
 CEF requests ─► adblock-rust + glass-box ─► (blocked | allowed) ─┘
 page text ─► quarantined reader (hearth-model) ─► reader/gist surface
```

---

## 7 · Dev-on-Windows → Wayland port

- **Now (Windows, 8 GB):** CEF ships Windows binaries; `wgpu`/`winit` are cross-platform — so B1's
  engine + composite + chrome + CDP + adblock are all buildable on the current box as a standalone
  Rust app. CEF is **heavy** (~150–200 MB, RAM-hungry) — fine for developing one surface; production
  leanness comes from space-suspension (`BROWSER-INTEGRATION.md` §10).
- **Later (target):** swap `winit` for the `smithay` compositor surface; the wgpu drawing, CEF, CDP,
  and adblock layers are unchanged. The port is the windowing seam, not the stack.

---

## 8 · Milestones (B1, in order)

1. **Hello, embedded web** — CEF OSR → texture → wgpu quad; a real page renders in our window.
2. **Drivable** — input injection: click/type/scroll the page.
3. **Our chrome** — nav + address-as-intent driving navigation; the glass surface frame + motion.
4. **Private by default** — adblock-rust request filtering + the glass-box telemetry panel.
5. **Distilled** — readability extraction → reader surface; summarise via `hearth-model` (quarantined).
6. **The seam** — `web` capability skeleton over CDP (`navigate`/`read`/`snapshot`) — ready for B2.

Each milestone is demoable on its own; (1)–(4) give a *fast, private* browser before any model is
involved.

---

## 9 · What this sets up

B1 ends with the engine embedded, the chrome ours, privacy on, and — crucially — the **`web`
capability** as the single, structured channel between the steward and the page. B2 fills in the
acting tools and **spaces**; B3 turns on agency *behind* `AGENT-TRUST-MODEL.md`. The hardest
integration risk (embedding + compositing + control) is retired first, in B1, on hardware we already
have.
