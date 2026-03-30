mod commands;
mod config;
mod ffmpeg;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "demovid", about = "Standalone demo video production tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new demo project with scaffolding and agent instructions
    Init {
        /// Project slug (used as directory name under ~/demos/)
        slug: String,
    },

    /// Probe a media file and print its duration
    Probe {
        /// Path to media file
        file: PathBuf,
    },

    /// Probe all mp4 files in a directory and print JSON summary
    ProbeAll {
        /// Directory containing mp4 files
        dir: PathBuf,
    },

    /// Extract frames from a video at a given FPS
    Frames {
        /// Input video file
        clip: PathBuf,
        /// Output directory for frames
        outdir: PathBuf,
        /// Frames per second to extract
        #[arg(long, default_value = "2")]
        fps: f64,
    },

    /// Generate TTS audio from text (not yet implemented)
    Tts {
        /// Text to synthesize
        text: String,
        /// Output audio file path
        output: PathBuf,
        /// Voice name
        #[arg(long, default_value = "af_bella")]
        voice: String,
        /// Speech speed
        #[arg(long, default_value = "0.95")]
        speed: f64,
        /// TTS provider (local or elevenlabs)
        #[arg(long, default_value = "local")]
        provider: String,
    },

    /// Generate silence audio (not yet implemented)
    Silence {
        /// Duration in seconds
        seconds: f64,
        /// Output audio file path
        output: PathBuf,
    },

    /// Concatenate audio files (not yet implemented)
    ConcatAudio {
        /// Input audio files
        files: Vec<PathBuf>,
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Render a clip with audio overlay (not yet implemented)
    RenderClip {
        /// Input video file
        video: PathBuf,
        /// Input audio file
        audio: PathBuf,
        /// Output video file
        output: PathBuf,
    },

    /// Assemble multiple clips into a final video (not yet implemented)
    Assemble {
        /// Input clip files
        clips: Vec<PathBuf>,
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { slug } => commands::init::run(&slug),
        Commands::Probe { file } => commands::probe::run(&file),
        Commands::ProbeAll { dir } => commands::probe::run_all(&dir),
        Commands::Frames { clip, outdir, fps } => commands::frames::run(&clip, &outdir, fps),
        Commands::Tts { .. } => commands::tts::run(),
        Commands::Silence { .. } => commands::silence::run(),
        Commands::ConcatAudio { .. } => commands::concat_audio::run(),
        Commands::RenderClip { .. } => commands::render_clip::run(),
        Commands::Assemble { .. } => commands::assemble::run(),
    }
}
