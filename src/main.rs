mod commands;
mod config;
mod ffmpeg;

use clap::{Parser, Subcommand};
use is_terminal::IsTerminal;
use std::path::PathBuf;

/// Exit codes
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_GENERAL: i32 = 1;
pub const EXIT_FILE_NOT_FOUND: i32 = 2;
pub const EXIT_TOOL_ERROR: i32 = 3;
pub const EXIT_INVALID_INPUT: i32 = 4;
pub const EXIT_ALREADY_EXISTS: i32 = 5;

#[derive(Parser)]
#[command(name = "demovid", about = "Standalone demo video production tool")]
struct Cli {
    /// Force JSON output (default when stdout is not a TTY)
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new demo project with scaffolding and agent instructions
    Init {
        /// Project slug (used as directory name under ~/demos/)
        slug: String,
        /// Skip interactive prompts (no-op, included for agent compatibility)
        #[arg(long, alias = "no-input")]
        yes: bool,
    },

    /// Probe a media file and print its duration
    Probe {
        /// Path to media file
        file: PathBuf,
    },

    /// Probe all mp4 files in a directory and print NDJSON
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

    /// Output the JSON schema for a command's output
    Schema {
        /// Command name to show schema for
        command: String,
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

/// Whether to use JSON output (either --json flag or non-TTY stdout)
pub fn use_json(json_flag: bool) -> bool {
    json_flag || !std::io::stdout().is_terminal()
}

/// Print a machine-readable error to stderr and exit with the given code.
pub fn exit_error(error_type: &str, code: i32, detail: &str) -> ! {
    let err = serde_json::json!({
        "error": error_type,
        "code": code,
        "detail": detail,
    });
    eprintln!("{}", err);
    std::process::exit(code)
}

fn main() {
    let cli = Cli::parse();
    let json_mode = use_json(cli.json);

    let result = match cli.command {
        Commands::Init { slug, yes: _ } => commands::init::run(&slug, json_mode),
        Commands::Probe { file } => commands::probe::run(&file, json_mode),
        Commands::ProbeAll { dir } => commands::probe::run_all(&dir, json_mode),
        Commands::Frames { clip, outdir, fps } => commands::frames::run(&clip, &outdir, fps, json_mode),
        Commands::Schema { command } => commands::schema::run(&command),
        Commands::Tts { .. } => commands::tts::run(),
        Commands::Silence { .. } => commands::silence::run(),
        Commands::ConcatAudio { .. } => commands::concat_audio::run(),
        Commands::RenderClip { .. } => commands::render_clip::run(),
        Commands::Assemble { .. } => commands::assemble::run(),
    };

    if let Err(e) = result {
        // Determine the exit code from the error chain
        let detail = format!("{:#}", e);
        let (error_type, code) = categorize_error(&detail);
        exit_error(error_type, code, &detail);
    }
}

fn categorize_error(detail: &str) -> (&'static str, i32) {
    let lower = detail.to_lowercase();
    if lower.contains("not found") || lower.contains("no such file") {
        ("file_not_found", EXIT_FILE_NOT_FOUND)
    } else if lower.contains("ffprobe") || lower.contains("ffmpeg") {
        ("tool_error", EXIT_TOOL_ERROR)
    } else if lower.contains("invalid") || lower.contains("bad argument") {
        ("invalid_input", EXIT_INVALID_INPUT)
    } else if lower.contains("already exists") {
        ("already_exists", EXIT_ALREADY_EXISTS)
    } else {
        ("general_error", EXIT_GENERAL)
    }
}
