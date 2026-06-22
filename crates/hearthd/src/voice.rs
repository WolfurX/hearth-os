//! Voice out — the steward speaking its reply through the system's text-to-speech, shelling out
//! like the rest of the runtime (no audio crates). The core turn stays text-in/text-out; this is
//! a presentation layer for the direct CLI client. Graceful: a missing engine, no audio device, or
//! any error just means silence — speaking is never allowed to fail a turn.
//!
//! On the Arch target the engine is `espeak-ng` (or `piper` later, for warmth); on the Windows
//! dev box it's SAPI via PowerShell; on macOS, `say`. The listening half (mic → STT → intent)
//! belongs to the shell and feeds the same text-intent path the runtime already serves.

use std::process::Command;

/// Speak `text` aloud via the platform's TTS. Best-effort: returns quietly on any failure.
pub fn speak(text: &str) {
    let clean = text.replace(['\n', '\r'], " ");
    let clean = clean.trim();
    if clean.is_empty() {
        return;
    }

    #[cfg(windows)]
    {
        // SAPI via PowerShell; single-quote escape by doubling for the PS string literal.
        let safe = clean.replace('\'', "''");
        let script = format!(
            "Add-Type -AssemblyName System.Speech; \
             (New-Object System.Speech.Synthesis.SpeechSynthesizer).Speak('{safe}')"
        );
        let _ = Command::new("powershell").args(["-NoProfile", "-Command", &script]).status();
    }

    #[cfg(not(windows))]
    {
        // Arch/Linux: espeak-ng. macOS: say. First engine present wins; absence is silent.
        if Command::new("espeak-ng").arg(clean).status().map(|s| s.success()).unwrap_or(false) {
            return;
        }
        let _ = Command::new("say").arg(clean).status();
    }
}
