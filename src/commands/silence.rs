use anyhow::{ensure, Context, Result};
use std::fs;
use std::path::Path;

use crate::ffmpeg;

pub fn run(seconds: f64, output: &Path, json_mode: bool) -> Result<()> {
    ensure!(
        seconds.is_finite() && seconds >= 0.0,
        "invalid input: seconds must be >= 0"
    );

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory {}", parent.display()))?;
    }

    ffmpeg::generate_silence(seconds, output)?;

    if json_mode {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "seconds": seconds,
                "output": output,
                "format": "wav",
                "sample_rate": 24000
            })
        );
    } else {
        println!("silence ok → {} ({seconds:.3}s)", output.display());
    }

    Ok(())
}
