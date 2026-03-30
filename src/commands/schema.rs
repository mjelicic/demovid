use anyhow::Result;

pub fn run(command: &str) -> Result<()> {
    let schema = match command {
        "probe" => serde_json::json!({
            "command": "probe",
            "output": {
                "file": "string",
                "duration_secs": "number"
            },
            "exit_codes": {
                "0": "success",
                "2": "file not found",
                "3": "ffprobe error"
            }
        }),
        "probe-all" => serde_json::json!({
            "command": "probe-all",
            "output_format": "ndjson",
            "output": {
                "file": "string",
                "duration_secs": "number | null",
                "error": "string | null"
            },
            "exit_codes": {
                "0": "success",
                "2": "directory not found",
                "4": "invalid input"
            }
        }),
        "frames" => serde_json::json!({
            "command": "frames",
            "output": {
                "frames_written": "number",
                "outdir": "string",
                "fps": "number"
            },
            "exit_codes": {
                "0": "success",
                "2": "file not found",
                "3": "ffmpeg error"
            }
        }),
        "init" => serde_json::json!({
            "command": "init",
            "output": {
                "project": "string",
                "path": "string",
                "dirs": ["string"],
                "already_existed": "boolean"
            },
            "exit_codes": {
                "0": "success",
                "5": "directory exists but not a valid project"
            }
        }),
        other => {
            crate::exit_error(
                "invalid_input",
                crate::EXIT_INVALID_INPUT,
                &format!("Unknown command for schema: '{}'. Valid: probe, probe-all, frames, init", other),
            );
        }
    };

    println!("{}", schema);
    Ok(())
}
