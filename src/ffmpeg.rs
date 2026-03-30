use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Run ffprobe on a file and return the raw JSON output as a serde_json::Value.
pub fn ffprobe_json(path: &Path) -> Result<serde_json::Value> {
    let path_str = path
        .to_str()
        .context("Path contains invalid UTF-8")?;

    let output = Command::new("ffprobe")
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
            path_str,
        ])
        .output()
        .context("Failed to run ffprobe. Is ffmpeg installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffprobe failed: {}", stderr);
    }

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).context("Failed to parse ffprobe JSON output")?;

    Ok(json)
}

/// Extract duration in seconds from ffprobe JSON output.
/// Tries format.duration first, then streams[0].duration.
pub fn extract_duration(probe: &serde_json::Value) -> Result<f64> {
    // Try format.duration first
    if let Some(dur_str) = probe
        .get("format")
        .and_then(|f| f.get("duration"))
        .and_then(|d| d.as_str())
    {
        return dur_str
            .parse::<f64>()
            .context("Failed to parse format.duration as f64");
    }

    // Fallback to streams[0].duration
    if let Some(dur_str) = probe
        .get("streams")
        .and_then(|s| s.as_array())
        .and_then(|arr| arr.first())
        .and_then(|s| s.get("duration"))
        .and_then(|d| d.as_str())
    {
        return dur_str
            .parse::<f64>()
            .context("Failed to parse streams[0].duration as f64");
    }

    anyhow::bail!("No duration found in ffprobe output")
}

/// Extract frames from a video at the given FPS into the output directory.
pub fn extract_frames(clip: &Path, outdir: &Path, fps: f64) -> Result<()> {
    let clip_str = clip.to_str().context("Clip path contains invalid UTF-8")?;
    let outdir_str = outdir
        .to_str()
        .context("Output dir path contains invalid UTF-8")?;

    let filter = format!("fps={}", fps);
    let output_pattern = format!("{}/frame-%04d.png", outdir_str);

    let output = Command::new("ffmpeg")
        .args([
            "-i", clip_str,
            "-vf", &filter,
            &output_pattern,
            "-y",
        ])
        .output()
        .context("Failed to run ffmpeg. Is ffmpeg installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg frame extraction failed: {}", stderr);
    }

    Ok(())
}
