use anyhow::{Context, Result};
use std::path::PathBuf;

const TEMPLATE_PROJECT_YAML: &str = include_str!("../../templates/project.yaml");
const TEMPLATE_CLAUDE_MD: &str = include_str!("../../templates/CLAUDE.md");
const TEMPLATE_AGENTS_MD: &str = include_str!("../../templates/AGENTS.md");

pub fn run(slug: &str) -> Result<()> {
    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    let base = PathBuf::from(home).join("demos").join(slug);

    if base.exists() {
        anyhow::bail!("Project directory already exists: {}", base.display());
    }

    // Create directory structure
    let subdirs = ["clips", "audio", "renders", "frames"];
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

    println!("Created project: {}", base.display());
    println!("  clips/");
    println!("  audio/");
    println!("  renders/");
    println!("  frames/");
    println!("  project.yaml");
    println!("  CLAUDE.md");
    println!("  AGENTS.md");

    Ok(())
}
