use anyhow::{ensure, Context, Result};
use std::fs;
use std::path::Path;

use crate::ffmpeg;

const DURATION_EPSILON_SECS: f64 = 0.01;

pub fn run(video: &Path, audio: &Path, output: &Path, json_mode: bool) -> Result<()> {
    let video = video
        .canonicalize()
        .with_context(|| format!("Video not found: {}", video.display()))?;
    let audio = audio
        .canonicalize()
        .with_context(|| format!("Audio not found: {}", audio.display()))?;

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory {}", parent.display()))?;
    }

    let video_duration = ffmpeg::media_duration(&video)?;
    let audio_duration = ffmpeg::media_duration(&audio)?;

    ensure!(
        video_duration > 0.0,
        "invalid input: video duration must be > 0"
    );
    ensure!(
        audio_duration > 0.0,
        "invalid input: audio duration must be > 0"
    );

    let strategy = if audio_duration > video_duration + DURATION_EPSILON_SECS {
        ffmpeg::RenderStrategy::FreezeLastFrameToAudio
    } else {
        ffmpeg::RenderStrategy::PadAudioToVideo
    };

    ffmpeg::render_clip(&video, &audio, output, strategy)?;

    let output_duration = ffmpeg::media_duration(output)?;

    if json_mode {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "video": video,
                "audio": audio,
                "output": output,
                "video_duration_secs": video_duration,
                "audio_duration_secs": audio_duration,
                "output_duration_secs": output_duration,
                "strategy": strategy.as_str(),
                "video_encoder": "h264_videotoolbox",
                "audio_encoder": "aac"
            })
        );
    } else {
        println!(
            "render ok → {} [{} | video {:.3}s | audio {:.3}s | out {:.3}s]",
            output.display(),
            strategy.as_str(),
            video_duration,
            audio_duration,
            output_duration
        );
    }

    Ok(())
}
