use anyhow::{Context, Result};
use std::path::PathBuf;

const TEMPLATE_PROJECT_YAML: &str = include_str!("../../templates/project.yaml");
const TEMPLATE_CLAUDE_MD: &str = include_str!("../../templates/CLAUDE.md");
const TEMPLATE_AGENTS_MD: &str = include_str!("../../templates/AGENTS.md");

pub fn run(slug: &str, json_mode: bool) -> Result<()> {
    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    let base = PathBuf::from(home).join("demos").join(slug);
    let subdirs = vec!["clips", "audio", "renders", "frames"];

    // Idempotent: if dir exists and has project.yaml, return success
    if base.exists() {
        if base.join("project.yaml").exists() {
            let base_str = base.to_string_lossy().to_string();
            let out = serde_json::json!({
                "project": slug,
                "path": base_str,
                "dirs": subdirs,
                "already_existed": true,
            });
            if json_mode {
                println!("{}", out);
            } else {
                eprintln!("Project already exists at {}", base_str);
                println!("{}", serde_json::to_string_pretty(&out)?);
            }
            return Ok(());
        } else {
            // Directory exists but not a valid project
            crate::exit_error(
                "already_exists",
                crate::EXIT_ALREADY_EXISTS,
                &format!(
                    "Directory exists but is not a valid demovid project (no project.yaml): {}",
                    base.display()
                ),
            );
        }
    }

    // Create directory structure
    for subdir in &subdirs {
        std::fs::create_dir_all(base.join(subdir))
            .with_context(|| format!("Failed to create {}", subdir))?;
    }

    // Write project.yaml with slug substitution
    let project_yaml = TEMPLATE_PROJECT_YAML.replace("{{slug}}", slug);
    std::fs::write(base.join("project.yaml"), &project_yaml)
        .context("Failed to write project.yaml")?;

    // Write agent instruction files
    std::fs::write(base.join("CLAUDE.md"), TEMPLATE_CLAUDE_MD)
        .context("Failed to write CLAUDE.md")?;

    std::fs::write(base.join("AGENTS.md"), TEMPLATE_AGENTS_MD)
        .context("Failed to write AGENTS.md")?;

    let base_str = base.to_string_lossy().to_string();
    let out = serde_json::json!({
        "project": slug,
        "path": base_str,
        "dirs": subdirs,
        "already_existed": false,
    });

    if json_mode {
        println!("{}", out);
    } else {
        eprintln!("Created project: {}", base_str);
        println!("{}", serde_json::to_string_pretty(&out)?);
    }

    Ok(())
}
