use anyhow::{ensure, Context, Result};
use std::fs;
use std::path::Path;

use crate::ffmpeg;

pub fn run(files: &[std::path::PathBuf], output: &Path, json_mode: bool) -> Result<()> {
    ensure!(
        files.len() >= 2,
        "invalid input: concat-audio needs at least two input files"
    );

    for file in files {
        ensure!(file.exists(), "file not found: {}", file.display());
    }

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory {}", parent.display()))?;
    }

    ffmpeg::concat_audio(files, output)?;

    if json_mode {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "inputs": files,
                "output": output,
                "count": files.len()
            })
        );
    } else {
        println!(
            "concat-audio ok → {} ({} inputs)",
            output.display(),
            files.len()
        );
    }

    Ok(())
}
