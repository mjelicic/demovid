use anyhow::{Context, Result};
use std::path::Path;

use crate::ffmpeg;

/// Probe a single file and print its duration.
pub fn run(file: &Path) -> Result<()> {
    let file = file
        .canonicalize()
        .with_context(|| format!("File not found: {}", file.display()))?;

    let probe = ffmpeg::ffprobe_json(&file)?;
    let duration = ffmpeg::extract_duration(&probe)?;

    println!("{:.2}s", duration);
    Ok(())
}

/// Probe all mp4 files in a directory and print JSON summary.
pub fn run_all(dir: &Path) -> Result<()> {
    let dir = dir
        .canonicalize()
        .with_context(|| format!("Directory not found: {}", dir.display()))?;

    if !dir.is_dir() {
        anyhow::bail!("Not a directory: {}", dir.display());
    }

    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .context("Failed to read directory")?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("mp4"))
                .unwrap_or(false)
        })
        .collect();

    entries.sort_by_key(|e| e.file_name());

    let mut results = Vec::new();

    for entry in &entries {
        let path = entry.path();
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        match ffmpeg::ffprobe_json(&path).and_then(|p| ffmpeg::extract_duration(&p)) {
            Ok(duration) => {
                results.push(serde_json::json!({
                    "file": filename,
                    "duration": duration,
                }));
            }
            Err(e) => {
                results.push(serde_json::json!({
                    "file": filename,
                    "error": format!("{}", e),
                }));
            }
        }
    }

    let json = serde_json::to_string_pretty(&results)?;
    println!("{}", json);
    Ok(())
}
