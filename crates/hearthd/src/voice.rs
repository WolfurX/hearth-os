//! Voice out — the steward speaking its reply through the system's text-to-speech, shelling out
//! like the rest of the runtime (no audio crates). The core turn stays text-in/text-out; this is
//! a presentation layer for the direct CLI client. Graceful: a missing engine, no audio device, or
//! any error just means silence — speaking is never allowed to fail a turn.
//!
//! On the Arch target the engine is `espeak-ng` (or `piper` later, for warmth); on the Windows
//! dev box it's SAPI via PowerShell; on macOS, `say`. The listening half (mic → STT → intent)
//! belongs to the shell and feeds the same text-intent path the runtime already serves.

use anyhow::{anyhow, bail, Result};
use std::path::Path;
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

/// Capture one short utterance from the microphone and transcribe it — the listening half of
/// voice. Records mono 16 kHz (what whisper wants) for `secs`, then runs whisper.cpp over it.
/// Returns `None` if nothing was said. Errors (no recorder, no whisper, no model) are surfaced so
/// `hearthd listen` can explain what to install — this path lives on Arch, not the Windows dev box.
pub fn listen_once(secs: u32) -> Result<Option<String>> {
    let wav = std::env::temp_dir().join("hearth-listen.wav");
    record(secs, &wav)?;
    let text = transcribe(&wav)?;
    let _ = std::fs::remove_file(&wav);
    let text = text.trim();
    Ok(if text.is_empty() { None } else { Some(text.to_string()) })
}

/// Record `secs` of mic audio to `wav` via ALSA `arecord` (standard on Arch via `alsa-utils`,
/// works over the PipeWire bridge too).
fn record(secs: u32, wav: &Path) -> Result<()> {
    let status = Command::new("arecord")
        .args(["-q", "-f", "S16_LE", "-c", "1", "-r", "16000", "-d", &secs.to_string()])
        .arg(wav)
        .status()
        .map_err(|e| anyhow!("no microphone recorder (arecord): {e} — install alsa-utils"))?;
    if !status.success() {
        bail!("recording failed");
    }
    Ok(())
}

/// Transcribe a WAV with whisper.cpp (`whisper-cli` on `PATH`, model from `HEARTH_WHISPER_MODEL`).
/// `-nt` drops timestamps, leaving plain text on stdout.
fn transcribe(wav: &Path) -> Result<String> {
    let model = std::env::var("HEARTH_WHISPER_MODEL")
        .map_err(|_| anyhow!("set HEARTH_WHISPER_MODEL to a whisper.cpp ggml model (e.g. ggml-base.en.bin)"))?;
    let out = Command::new("whisper-cli")
        .args(["-m", &model, "-nt", "-f"])
        .arg(wav)
        .output()
        .map_err(|e| anyhow!("no whisper-cli on PATH: {e} — install whisper.cpp"))?;
    if !out.status.success() {
        bail!("whisper failed: {}", String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" "))
}
