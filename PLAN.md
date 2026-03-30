# demovid — Standalone Demo Video Production Tool

## What This Is

AI-agent-driven demo video production. Human provides the outcome, scenario, and screen recordings. AI agent handles everything else: shot list, narrative, TTS, rendering, assembly. Fully local by default — no cloud APIs needed.

Works with **Claude Code** and **Codex** — agent instructions ship with every project.

## Architecture

```
demovid (Rust binary, native ARM64, optimized for Apple Silicon)
  ├── TTS:    voice-tts crate (Kokoro 82M on Metal/MLX via mlx-rs)
  ├── Video:  std::process::Command → ffmpeg -hwaccel videotoolbox
  ├── Config: serde + serde_yaml
  ├── CLI:    clap
  └── HTTP:   reqwest (optional, ElevenLabs fallback)
```

**Two layers:**

1. **CLI tool** (`demovid`) — Rust binary. No AI inside. Handles TTS, ffmpeg, file management. The mechanical pipeline.
2. **AI agent instructions** — CLAUDE.md / AGENTS.md templates stamped out by `demovid init`. Any AI coding agent reads them and knows the full 10-step workflow.

## Workflow

Human does the bare minimum. AI agent manages the workflow. Every step loops until the human approves.

```
 1. OUTCOME      Human defines what audience should believe
 2. SCENARIO     Human defines the plot (what happens in the demo)
 3. SHOT LIST    AI suggests clips to record → human approves ↩
 4. NARRATIVE    Human writes/iterates voiceover with AI → approves ↩
 5. CLIPS        Human records, drops in clips/ folder
 6. MAPPING      AI maps narration segments to clips → human approves ↩
 7. INDIVIDUAL   AI renders each clip+narration → human approves each ↩
 8. ASSEMBLY     AI combines approved clips into final video
 9. FINAL REVIEW Human approves or flags specific clips → back to 7 ↩
10. SHIP
```

Steps 1–6 are text-only (free to iterate). TTS + rendering only happens at step 7+. This is intentional — iterate on words before spending compute.

## CLI Commands

```
demovid init <slug>                          # Create project + agent instructions
demovid probe <file>                         # Duration in seconds (ffprobe)
demovid probe-all <project-dir>              # All clips → JSON
demovid frames <clip> <outdir> [--fps 2]     # Extract frames for AI analysis
demovid tts <text> <output> [--voice af_bella] [--speed 0.95]  # Local Kokoro TTS
demovid tts <text> <output> --provider elevenlabs              # Cloud TTS fallback
demovid silence <seconds> <output>           # Generate silence audio
demovid concat-audio <files...> -o <output>  # Concatenate audio files
demovid render-clip <video> <audio> <output> # Overlay audio on video (ffmpeg)
demovid assemble <clips...> -o <output>      # Concatenate clips → final mp4
```

## Project Structure

`demovid init my-demo` creates:

```
~/demos/my-demo/
  CLAUDE.md          # Agent instructions (Claude Code)
  AGENTS.md          # Agent instructions (Codex)
  project.yaml       # Workflow state (agent-managed, never human-edited)
  clips/             # Raw screen recordings (human drops these here)
  audio/             # Generated TTS audio
  renders/           # Individual clip renders + final video
  frames/            # Extracted frames for AI analysis (ephemeral)
```

## project.yaml

Single source of truth for workflow state. Agent-managed. Human never edits it. Agent reads it to resume across conversations.

```yaml
version: 1
slug: my-demo
step: mapping    # current workflow step

outcome: "The PE board believes this system runs itself"
scenario: "Employee onboarding breaks, agent loop detects and resolves autonomously"

voice:
  provider: local          # local (Kokoro) | elevenlabs
  voice: af_bella          # Kokoro voice name or ElevenLabs voice_id
  speed: 0.95

shots:
  - id: 1
    label: "Onboarding blocked"
    description: "Jane tries to onboard, gets blocked"
    status: approved       # draft | approved

narrative:
  - id: 1
    shot_id: 1
    text: "Jane Martinez just tried to complete her first-day onboarding..."
    pause_after: 0
    status: approved

clips:
  - shot_id: 1
    file: clips/clip-1-onboarding.mp4
    duration: 10.5

mapping:
  - clip_id: 1
    narration_ids: [1]
    status: approved

renders:
  - clip_id: 1
    audio_file: audio/clip-1.mp3
    render_file: renders/clip-1-render.mp4
    status: approved       # pending | rendered | approved | rejected

final:
  file: renders/final.mp4
  status: pending
```

## Dependencies

**System (required):**
- ffmpeg / ffprobe (for video operations)
- Xcode CLI tools (for MLX/Metal compilation)

**Rust crates:**
```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1"
voice-tts = "0.2"           # Kokoro TTS on Metal/MLX
anyhow = "1"

[features]
default = []
elevenlabs = ["dep:reqwest"]

[dependencies.reqwest]
version = "0.12"
features = ["blocking"]
optional = true
```

## Source Layout

```
Cargo.toml
src/
  main.rs              # Entry point + clap CLI
  commands/
    mod.rs
    init.rs            # Project scaffolding + template stamping
    probe.rs           # ffprobe wrappers
    frames.rs          # Frame extraction
    tts.rs             # Kokoro TTS via voice-tts + ElevenLabs fallback
    silence.rs         # Silence generation
    concat_audio.rs    # Audio concatenation
    render_clip.rs     # Audio-on-video overlay
    assemble.rs        # Multi-clip assembly
  config.rs            # project.yaml serde types
  ffmpeg.rs            # ffmpeg command builder helpers
templates/
  CLAUDE.md            # Agent instructions for Claude Code
  AGENTS.md            # Agent instructions for Codex
  project.yaml         # Skeleton project state
```

---

## Milestones

### M1: Scaffold + Media Probing
**Vector:** H-355CC6 | **State:** ready-for-build | **Depends on:** nothing

Rust project structure, clap CLI framework, `init`, `probe`, `probe-all`, `frames` commands.

**Exit criteria:**
- [ ] `demovid probe ~/clips/clip-1-intro-better.mp4` prints correct duration (±0.1s)
- [ ] `demovid probe-all ~/clips` prints JSON with duration for every mp4
- [ ] `demovid frames ~/clips/clip-1-intro-better.mp4 /tmp/frames --fps 2` creates ~24 PNGs
- [ ] `demovid init test-project` creates `~/demos/test-project/` with `clips/`, `audio/`, `renders/`, `frames/`, and valid `project.yaml`
- [ ] `cargo build --release` produces single ARM64 binary
- [ ] All commands return exit 0 on success, non-zero with useful error on failure

---

### M2: TTS Engine
**Vector:** H-F1F200 | **State:** ready-for-planning | **Depends on:** M1

Kokoro TTS via `voice-tts` crate. `tts`, `silence`, `concat-audio` commands.

**Exit criteria:**
- [ ] `demovid tts "This is a test." /tmp/test.wav` produces clear natural WAV in under 3 seconds
- [ ] `--voice am_adam` uses a different voice than default
- [ ] `--speed 0.9` produces audibly slower speech
- [ ] `demovid silence 2.0 /tmp/silence.wav` produces exactly 2.0s (verified by ffprobe)
- [ ] `demovid concat-audio test.wav silence.wav test.wav -o combined.wav` produces correct concatenation
- [ ] Model auto-downloads on first call, cached for subsequent calls
- [ ] TTS runs on Metal/MLX (GPU usage visible)

---

### M3: Video Assembly
**Vector:** H-10FC4C | **State:** ready-for-planning | **Depends on:** M1

`render-clip` and `assemble` commands via ffmpeg with hardware acceleration.

**Exit criteria:**
- [ ] `demovid render-clip clip.mp4 voiceover.wav rendered.mp4` produces playable MP4 with audio
- [ ] Audio shorter than video: silent tail
- [ ] Audio longer than video: freeze last frame
- [ ] `demovid assemble clip1.mp4 clip2.mp4 clip3.mp4 -o final.mp4` plays all in sequence, no glitches
- [ ] Uses VideoToolbox hardware encoding
- [ ] Tested with ClearHR clips: 5 clips rendered + assembled, comparable to existing demo1-final.mp4

---

### M4: Agent Templates
**Vector:** H-A8D759 | **State:** ready-for-planning | **Depends on:** M1

CLAUDE.md + AGENTS.md templates with complete 10-step workflow instructions.

**Exit criteria:**
- [ ] `demovid init my-demo` creates CLAUDE.md and AGENTS.md
- [ ] Complete 10-step workflow with explicit per-step instructions
- [ ] Exact `demovid` CLI commands for each mechanical step
- [ ] Approval protocol: never advance without human approval, loop on rejection
- [ ] Resume logic: read project.yaml → find step → pick up there
- [ ] project.yaml skeleton has all sections with placeholder values
- [ ] A human can read CLAUDE.md and understand the full workflow without other docs

---

### M5: End-to-End Integration Test
**Vector:** H-AABF3B | **State:** ready-for-planning | **Depends on:** M2, M3, M4

Full workflow with Claude Code + real ClearHR clips.

**Exit criteria:**
- [ ] Claude Code reads CLAUDE.md and correctly identifies step 1
- [ ] Agent navigates all 10 steps, calling `demovid` CLI at each step
- [ ] Approval gates work at steps 3, 4, 6, 7, 9
- [ ] Rejection causes agent to revise and re-render
- [ ] Resume across conversations works (kill + restart picks up at correct step)
- [ ] Final output is watchable with narration synced to visuals
- [ ] Wall-clock time under 10 minutes for 5-clip demo (excluding recording time)
- [ ] Same workflow works with Codex via AGENTS.md
