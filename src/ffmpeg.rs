use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderStrategy {
    PadAudioToVideo,
    FreezeLastFrameToAudio,
}

impl RenderStrategy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PadAudioToVideo => "pad-audio-to-video",
            Self::FreezeLastFrameToAudio => "freeze-last-frame-to-audio",
        }
    }
}

/// Run ffprobe on a file and return the raw JSON output as a serde_json::Value.
pub fn ffprobe_json(path: &Path) -> Result<serde_json::Value> {
    let path_str = path.to_str().context("Path contains invalid UTF-8")?;

    let output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
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
    if let Some(dur_str) = probe
        .get("format")
        .and_then(|f| f.get("duration"))
        .and_then(|d| d.as_str())
    {
        return dur_str
            .parse::<f64>()
            .context("Failed to parse format.duration as f64");
    }

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

pub fn media_duration(path: &Path) -> Result<f64> {
    let probe = ffprobe_json(path)?;
    extract_duration(&probe)
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
        .args(["-i", clip_str, "-vf", &filter, &output_pattern, "-y"])
        .output()
        .context("Failed to run ffmpeg. Is ffmpeg installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg frame extraction failed: {}", stderr);
    }

    Ok(())
}

pub fn generate_silence(seconds: f64, output: &Path) -> Result<()> {
    let output_str = output
        .to_str()
        .context("Output path contains invalid UTF-8")?;
    let duration = format!("{seconds:.6}");

    let output = Command::new("ffmpeg")
        .args([
            "-f",
            "lavfi",
            "-i",
            "anullsrc=r=24000:cl=mono",
            "-t",
            &duration,
            "-c:a",
            "pcm_s16le",
            output_str,
            "-y",
        ])
        .output()
        .context("Failed to run ffmpeg. Is ffmpeg installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg silence generation failed: {}", stderr);
    }

    Ok(())
}

pub fn concat_audio(files: &[PathBuf], output: &Path) -> Result<()> {
    let output_str = output
        .to_str()
        .context("Output path contains invalid UTF-8")?;
    let list_path = concat_manifest_path(output)?;

    let manifest = files
        .iter()
        .map(|file| {
            let canonical = file
                .canonicalize()
                .with_context(|| format!("failed to canonicalize {}", file.display()))?;
            Ok(format!("file '{}'", escape_concat_path(&canonical)))
        })
        .collect::<Result<Vec<_>>>()?
        .join("\n");

    fs::write(&list_path, format!("{manifest}\n"))
        .with_context(|| format!("failed to write concat manifest {}", list_path.display()))?;

    let list_str = list_path
        .to_str()
        .context("Concat manifest path contains invalid UTF-8")?;

    let ffmpeg_output = Command::new("ffmpeg")
        .args([
            "-f", "concat", "-safe", "0", "-i", list_str, "-c", "copy", output_str, "-y",
        ])
        .output()
        .context("Failed to run ffmpeg. Is ffmpeg installed?")?;

    let cleanup_result = fs::remove_file(&list_path);

    if !ffmpeg_output.status.success() {
        let stderr = String::from_utf8_lossy(&ffmpeg_output.stderr);
        anyhow::bail!("ffmpeg concat failed: {}", stderr);
    }

    cleanup_result.ok();
    Ok(())
}

pub fn render_clip(
    video: &Path,
    audio: &Path,
    output: &Path,
    strategy: RenderStrategy,
) -> Result<()> {
    let video_str = video
        .to_str()
        .context("Video path contains invalid UTF-8")?;
    let audio_str = audio
        .to_str()
        .context("Audio path contains invalid UTF-8")?;
    let output_str = output
        .to_str()
        .context("Output path contains invalid UTF-8")?;

    let filter = match strategy {
        RenderStrategy::PadAudioToVideo => {
            "[0:v]scale=trunc(iw/2)*2:trunc(ih/2)*2,setsar=1,format=yuv420p[v];[1:a]aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo,apad[a]".to_string()
        }
        RenderStrategy::FreezeLastFrameToAudio => {
            "[0:v]tpad=stop_mode=clone:stop_duration=3600,scale=trunc(iw/2)*2:trunc(ih/2)*2,setsar=1,format=yuv420p[v];[1:a]aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo[a]".to_string()
        }
    };

    let mut args = vec![
        "-v".to_string(),
        "error".to_string(),
        "-i".to_string(),
        video_str.to_string(),
        "-i".to_string(),
        audio_str.to_string(),
        "-filter_complex".to_string(),
        filter,
        "-map".to_string(),
        "[v]".to_string(),
        "-map".to_string(),
        "[a]".to_string(),
        "-c:v".to_string(),
        "h264_videotoolbox".to_string(),
        "-allow_sw".to_string(),
        "1".to_string(),
        "-b:v".to_string(),
        "8M".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
        "-c:a".to_string(),
        "aac".to_string(),
        "-b:a".to_string(),
        "192k".to_string(),
        "-movflags".to_string(),
        "+faststart".to_string(),
    ];

    match strategy {
        RenderStrategy::PadAudioToVideo => args.push("-shortest".to_string()),
        RenderStrategy::FreezeLastFrameToAudio => {
            args.extend(["-t".to_string(), media_duration(audio)?.to_string()])
        }
    }

    args.extend([output_str.to_string(), "-y".to_string()]);

    run_ffmpeg(&args, "ffmpeg render-clip failed")
}

pub fn assemble_clips(clips: &[PathBuf], output: &Path) -> Result<()> {
    let output_str = output
        .to_str()
        .context("Output path contains invalid UTF-8")?;
    let first_clip = clips.first().context("at least one clip is required")?;
    let profile = video_profile(first_clip)?;
    let mut args = vec!["-v".to_string(), "error".to_string()];
    let mut inputs = Vec::with_capacity(clips.len());
    let mut next_input_index = 0usize;

    for clip in clips {
        let clip_str = clip.to_str().context("Clip path contains invalid UTF-8")?;
        let media = media_profile(clip)?;
        args.extend(["-i".to_string(), clip_str.to_string()]);
        let video_input = next_input_index;
        next_input_index += 1;

        let audio_input = if media.has_audio {
            video_input
        } else {
            args.extend([
                "-f".to_string(),
                "lavfi".to_string(),
                "-t".to_string(),
                media.duration.to_string(),
                "-i".to_string(),
                "anullsrc=r=48000:cl=stereo".to_string(),
            ]);
            let silent_input = next_input_index;
            next_input_index += 1;
            silent_input
        };

        inputs.push(ConcatInput {
            video_input,
            audio_input,
        });
    }

    let filter = concat_filter(&inputs, profile.width, profile.height);

    args.extend([
        "-filter_complex".to_string(),
        filter,
        "-map".to_string(),
        "[v]".to_string(),
        "-map".to_string(),
        "[a]".to_string(),
        "-c:v".to_string(),
        "h264_videotoolbox".to_string(),
        "-allow_sw".to_string(),
        "1".to_string(),
        "-b:v".to_string(),
        "10M".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
        "-c:a".to_string(),
        "aac".to_string(),
        "-b:a".to_string(),
        "192k".to_string(),
        "-movflags".to_string(),
        "+faststart".to_string(),
        output_str.to_string(),
        "-y".to_string(),
    ]);

    run_ffmpeg_capture(&args, "ffmpeg assemble failed", None)
}

fn run_ffmpeg(args: &[String], context_msg: &str) -> Result<()> {
    run_ffmpeg_capture(args, context_msg, None)
}

fn run_ffmpeg_capture(
    args: &[String],
    context_msg: &str,
    required_stderr_text: Option<&str>,
) -> Result<()> {
    let output = Command::new("ffmpeg")
        .args(args)
        .output()
        .context("Failed to run ffmpeg. Is ffmpeg installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("{context_msg}: {stderr}");
    }

    if let Some(required_text) = required_stderr_text {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains(required_text) {
            anyhow::bail!(
                "{context_msg}: expected ffmpeg output to mention {required_text}, got: {stderr}"
            );
        }
    }

    Ok(())
}

fn concat_manifest_path(output: &Path) -> Result<PathBuf> {
    let parent = output
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock before unix epoch")?
        .as_millis();
    Ok(parent.join(format!(".demovid-concat-{stamp}.txt")))
}

fn escape_concat_path(path: &Path) -> String {
    path.display().to_string().replace('\'', "'\\''")
}

#[derive(Debug)]
struct VideoProfile {
    width: u32,
    height: u32,
}

fn video_profile(path: &Path) -> Result<VideoProfile> {
    let media = media_profile(path)?;
    Ok(VideoProfile {
        width: even_dimension(media.width),
        height: even_dimension(media.height),
    })
}

#[derive(Debug)]
struct MediaProfile {
    width: u32,
    height: u32,
    duration: f64,
    has_audio: bool,
}

fn media_profile(path: &Path) -> Result<MediaProfile> {
    let probe = ffprobe_json(path)?;
    let streams = probe
        .get("streams")
        .and_then(|s| s.as_array())
        .context("No streams found in ffprobe output")?;

    let video_stream = streams
        .iter()
        .find(|stream| stream.get("codec_type").and_then(|v| v.as_str()) == Some("video"))
        .context("No video stream found in ffprobe output")?;

    let width = video_stream
        .get("width")
        .and_then(|v| v.as_u64())
        .context("No width found in ffprobe output")? as u32;
    let height = video_stream
        .get("height")
        .and_then(|v| v.as_u64())
        .context("No height found in ffprobe output")? as u32;
    let has_audio = streams
        .iter()
        .any(|stream| stream.get("codec_type").and_then(|v| v.as_str()) == Some("audio"));

    Ok(MediaProfile {
        width,
        height,
        duration: extract_duration(&probe)?,
        has_audio,
    })
}

fn even_dimension(value: u32) -> u32 {
    if value % 2 == 0 {
        value
    } else {
        value - 1
    }
}

#[derive(Debug)]
struct ConcatInput {
    video_input: usize,
    audio_input: usize,
}

fn concat_filter(inputs: &[ConcatInput], width: u32, height: u32) -> String {
    let mut parts = Vec::with_capacity(inputs.len() + 1);
    let mut labels = String::new();

    for (index, input) in inputs.iter().enumerate() {
        parts.push(format!(
            "[{video}:v]scale={width}:{height}:force_original_aspect_ratio=decrease,pad={width}:{height}:(ow-iw)/2:(oh-ih)/2:color=black,setsar=1,format=yuv420p,fps=30[v{index}]",
            video = input.video_input
        ));
        parts.push(format!(
            "[{audio}:a]aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo[a{index}]",
            audio = input.audio_input
        ));
        labels.push_str(&format!("[v{index}][a{index}]"));
    }

    parts.push(format!(
        "{labels}concat=n={count}:v=1:a=1[v][a]",
        count = inputs.len()
    ));
    parts.join(";")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concat_filter_builds_expected_graph() {
        let filter = concat_filter(
            &[
                ConcatInput {
                    video_input: 0,
                    audio_input: 0,
                },
                ConcatInput {
                    video_input: 1,
                    audio_input: 2,
                },
            ],
            1280,
            720,
        );
        assert!(filter.contains("[0:v]scale=1280:720"));
        assert!(filter.contains("[1:v]scale=1280:720"));
        assert!(filter.contains("[2:a]aformat"));
        assert!(filter.contains("concat=n=2:v=1:a=1[v][a]"));
    }

    #[test]
    fn even_dimension_rounds_down_odd_values() {
        assert_eq!(even_dimension(1920), 1920);
        assert_eq!(even_dimension(1919), 1918);
    }
}
