use anyhow::{Context, Result};
use std::path::Path;

use crate::ffmpeg;

pub fn run(clip: &Path, outdir: &Path, fps: f64) -> Result<()> {
    let clip = clip
        .canonicalize()
        .with_context(|| format!("Clip not found: {}", clip.display()))?;

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(outdir)
        .with_context(|| format!("Failed to create output directory: {}", outdir.display()))?;

    ffmpeg::extract_frames(&clip, outdir, fps)?;

    // Count generated frames
    let count = std::fs::read_dir(outdir)
        .context("Failed to read output directory")?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("png"))
                .unwrap_or(false)
        })
        .count();

    println!("Extracted {} frames to {}", count, outdir.display());
    Ok(())
}
