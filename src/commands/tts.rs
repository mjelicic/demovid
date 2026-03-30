use anyhow::{Context, Result};
use std::path::Path;
use std::time::Instant;

const DEFAULT_VOICE: &str = "af_bella";

pub fn run(
    text: &str,
    output: &Path,
    voice: &str,
    speed: f64,
    _provider: &str,
    json_mode: bool,
) -> Result<()> {
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory {}", parent.display()))?;
    }

    let start = Instant::now();

    let voice_name = if voice.is_empty() { DEFAULT_VOICE } else { voice };

    // TtsEngine::new() is async (downloads models on first run).
    // Use a single-threaded tokio runtime to block on it.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to create tokio runtime")?;

    let mut tts = rt
        .block_on(kokoro_micro::TtsEngine::new())
        .map_err(|e| anyhow::anyhow!("failed to initialise TTS engine: {}", e))?;

    // Generate speech with voice and speed control
    let audio = tts
        .synthesize_with_options(text, Some(voice_name), speed as f32, 1.0, Some("en"))
        .map_err(|e| anyhow::anyhow!("TTS generation failed: {}", e))?;

    // Save WAV
    tts.save_wav(
        output.to_str().context("output path is not valid UTF-8")?,
        &audio,
    )
    .map_err(|e| anyhow::anyhow!("failed to save WAV: {}", e))?;

    let elapsed_ms = start.elapsed().as_millis() as u64;

    if json_mode {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "output": output,
                "voice": voice_name,
                "speed": speed,
                "elapsed_ms": elapsed_ms
            })
        );
    } else {
        println!(
            "tts ok → {} (voice={}, speed={}, {}ms)",
            output.display(),
            voice_name,
            speed,
            elapsed_ms
        );
    }

    Ok(())
}
