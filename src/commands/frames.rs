use anyhow::{Context, Result};
use std::path::Path;

use crate::ffmpeg;

pub fn run(clip: &Path, outdir: &Path, fps: f64, json_mode: bool) -> Result<()> {
    let clip = clip
        .canonicalize()
        .with_context(|| format!("Clip not found: {}", clip.display()))?;

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(outdir)
        .with_context(|| format!("Failed to create output directory: {}", outdir.display()))?;

    let outdir_canon = outdir
        .canonicalize()
        .with_context(|| format!("Failed to resolve output directory: {}", outdir.display()))?;

    ffmpeg::extract_frames(&clip, &outdir_canon, fps)?;

    // Count generated frames
    let count = std::fs::read_dir(&outdir_canon)
        .context("Failed to read output directory")?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("png"))
                .unwrap_or(false)
        })
        .count();

    let outdir_str = outdir_canon.to_string_lossy().to_string();

    if json_mode {
        let out = serde_json::json!({
            "frames_written": count,
            "outdir": outdir_str,
            "fps": fps,
        });
        println!("{}", out);
    } else {
        eprintln!("Extracted {} frames at {} fps", count, fps);
        let out = serde_json::json!({
            "frames_written": count,
            "outdir": outdir_str,
            "fps": fps,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    }

    Ok(())
}
