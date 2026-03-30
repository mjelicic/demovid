use anyhow::{Context, Result};
use std::path::Path;

use crate::ffmpeg;

/// Probe a single file and output structured JSON.
pub fn run(file: &Path, json_mode: bool) -> Result<()> {
    let file = file
        .canonicalize()
        .with_context(|| format!("File not found: {}", file.display()))?;

    let probe = ffmpeg::ffprobe_json(&file)?;
    let duration = ffmpeg::extract_duration(&probe)?;

    let file_str = file.to_string_lossy().to_string();

    if json_mode {
        let out = serde_json::json!({
            "file": file_str,
            "duration_secs": duration,
        });
        println!("{}", out);
    } else {
        eprintln!("Probing: {}", file_str);
        let out = serde_json::json!({
            "file": file_str,
            "duration_secs": duration,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    }

    Ok(())
}

/// Probe all mp4 files in a directory; output NDJSON (one JSON object per line).
pub fn run_all(dir: &Path, json_mode: bool) -> Result<()> {
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

    if !json_mode {
        eprintln!("Probing {} mp4 files in {}", entries.len(), dir.display());
    }

    for entry in &entries {
        let path = entry.path();
        let file_str = path.to_string_lossy().to_string();

        let line = match ffmpeg::ffprobe_json(&path).and_then(|p| ffmpeg::extract_duration(&p)) {
            Ok(duration) => {
                serde_json::json!({
                    "file": file_str,
                    "duration_secs": duration,
                    "error": null,
                })
            }
            Err(e) => {
                serde_json::json!({
                    "file": file_str,
                    "duration_secs": null,
                    "error": format!("{}", e),
                })
            }
        };

        // Always one JSON object per line (NDJSON)
        println!("{}", line);
    }

    Ok(())
}
