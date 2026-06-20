# Hearth OS — The App Model (adopt · strip · cohere)

*How apps work in Hearth: not rewritten from scratch, not run as foreign black boxes.*

> Companion to `BROWSER-INTEGRATION.md` (the browser is the first instance of this) and
> `UI-SOUL.md` (apps must obey the design language). A Phase-2/3 (compositor era) build;
> the decision is recorded now.

---

## Thesis

**Adopt a mature open-source app, strip it to its engine, and cohere it into the Hearth** —
visually *and* as an agent-drivable capability. The browser already proves the move: embed
Chromium's engine, discard its chrome, render our own. Make that the general law.

## Two tiers

### Tier 1 — first-class surfaces (the basics we ship)
Thin **Hearth surfaces over an OSS *engine/core*** — we own the surface, we drive the engine.

| Basic | OSS core | Hearth layer |
|---|---|---|
| **Browser** | Chromium / CEF (locked) | our chrome + the `web` capability over CDP |
| **Mail** | an OSS IMAP/JMAP sync engine | a Hearth mail surface + a `mail` capability |
| **Editor / docs** | an OSS text/rich-text core (+ LibreOffice **headless** for compat/convert) | a Hearth document surface + a `doc` capability |

Fully coherent and fully agentic, because nothing foreign is on screen — these *are* Hearth
surfaces.

### Tier 2 — adopted apps (anything installed on demand)
Run the real OSS app; the OS coheres it **automatically, where it actually can**:

- **The compositor owns the frame.** Our Wayland compositor strips native title bars/decorations
  and wraps *every* client window in a Hearth session surface. Works for **all** apps.
- **System theme.** Apply a Hearth theme where the toolkit allows (GTK/Qt); partial, best-effort.
- **Agent-drivable by default.** Drive any app via the **accessibility tree (AT-SPI) + input
  injection** (universal), with deeper hooks when the app exposes an **API / CLI / D-Bus**.
- **Adapter manifests.** A curated, community per-app manifest (a Hearth-flavored Flathub) encodes
  the best strip/theme/drive recipe for popular apps and raises their polish.

**On-demand install = "download · strip · cohere":** the OS fetches the OSS app (Flatpak / AUR /
source), applies framing + theme + the a11y/adapter layer, and registers it as a capability the
agent can open inside a session.

## The honest limit

**Deep re-skinning of an arbitrary app's *internal* UI is not automatable** — toolkits differ and
you can't repaint a foreign app's guts. We don't need it: *compositor-owns-frame + theme +
a11y-driving + Tier-1 engine-surfaces* reaches ~80% coherence, the right target. (OSS-only — you can
only legally strip what's open; honor licenses; on-device stripping keeps redistribution clean.)

## How it plugs into the runtime

Every app — both tiers — is exposed to the agent through the **capability fabric** (MCP, vision §3.4):

- **Tier 1** as native capabilities: we drive the engine (`web` over CDP, `mail` over the sync
  engine, `doc` over the editor core).
- **Tier 2** via a generic **`app` capability**: open / read-a11y-tree / act (input) / close, over
  any wrapped window.

`hearthd` drives them, the **trust model** gates them, the **glass box** shows them, and the
**UI-SOUL** rules make them feel like one room. There is no "app launcher" — the agent opens the app
a task needs, inside a session (`UI-SOUL.md`).
