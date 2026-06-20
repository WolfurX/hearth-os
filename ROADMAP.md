# Hearth OS — Roadmap

A purist OS is a multi-year arc, sequenced so there is something **real and usable early**, then the
hosted scaffolding is progressively replaced with owned components. Full rationale in
[docs/VISION-AND-ARCHITECTURE.md §5](docs/VISION-AND-ARCHITECTURE.md).

### Phase 0 — Prototype & design — ✅ done
The soul (**The Hearth**, chosen over voice-first / infinite-canvas / augmented-desktop), the Brain
concept, and the design canon. → [`mockup/`](mockup/), [`docs/`](docs/).

### Phase 1 — The mind, the fabric, the first Brain — ✅ complete
`hearthd` (intent → plan → gate → act → audit) + the neutral model router + the **MCP** capability
fabric + the **sovereignty substrate** (snapshot-first undo) + the **Brain**. A headless steward you
can converse with, that safely operates a real system with full undo, and learns you in the open —
proven end to end on a live LLM.

### Phase 2 — The Hearth shell, surfaces, the learning Brain — → next
The Aurora shell ([`mockup/the-hearth.html`](mockup/the-hearth.html) is the spec), the Surface DSL +
renderer, voice, the six canonical states; the Brain's automatic consolidation, the lessons layer,
and retrieval. First hosted as a Wayland client. **Bridge the runtime to the UI.**

### Phase 3 — The compositor (purist)
Replace the host compositor with our smithay/Wayland one; native surface rendering, deep `display`
access. The browser track (B1) lands here: embed Chromium/CEF, paint our own chrome.

### Phase 4 — The installer & distro
Custom archiso, the offline all-drivers repo, neutral model selection, first-boot meeting, the
customization engine. → install Hearth OS on bare metal, online anywhere, out of the box.

### Phase 5 — Self-evolution & hardening
System-as-code self-patching, deeper trust + Brain governance, multi-user (per-user Brains),
accessibility re-rendering, security audits, performance.

---

**Browser track** (B1–B5) and the **app model** are detailed in
[docs/BROWSER-INTEGRATION.md](docs/BROWSER-INTEGRATION.md),
[docs/AGENT-TRUST-MODEL.md](docs/AGENT-TRUST-MODEL.md), and [docs/APP-MODEL.md](docs/APP-MODEL.md).
