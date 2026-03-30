use anyhow::{ensure, Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::ffmpeg;

pub fn run(clips: &[PathBuf], output: &std::path::Path, json_mode: bool) -> Result<()> {
    ensure!(
        clips.len() >= 2,
        "invalid input: provide at least two clips to assemble"
    );

    let clips = clips
        .iter()
        .map(|clip| {
            clip.canonicalize()
                .with_context(|| format!("Clip not found: {}", clip.display()))
        })
        .collect::<Result<Vec<_>>>()?;

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory {}", parent.display()))?;
    }

    ffmpeg::assemble_clips(&clips, output)?;

    let output_duration = ffmpeg::media_duration(output)?;

    if json_mode {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "clips": clips,
                "output": output,
                "clip_count": clips.len(),
                "output_duration_secs": output_duration,
                "video_encoder": "h264_videotoolbox",
                "audio_encoder": "aac"
            })
        );
    } else {
        println!(
            "assemble ok → {} ({} clips, {:.3}s, h264_videotoolbox)",
            output.display(),
            clips.len(),
            output_duration
        );
    }

    Ok(())
}
