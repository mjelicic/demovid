# Demo Video Production -- Agent Instructions (Claude Code)

You are producing a demo video using the `demovid` CLI tool.
All project state lives in `project.yaml` in this directory.

## Resume Protocol

Before doing anything, read `project.yaml` and check the `step` field.
Resume from that step. Do not repeat completed steps unless the human asks.

```
step: 1   -> Go to Step 1: Gather Clips
step: 2   -> Go to Step 2: Plan Narration
step: 3   -> Go to Step 3: Generate TTS
step: 4   -> Go to Step 4: Check Audio
step: 5   -> Go to Step 5: Trim/Pad Clips
step: 6   -> Go to Step 6: Render Clips
step: 7   -> Go to Step 7: Review Clips
step: 8   -> Go to Step 8: Record Fixes
step: 9   -> Go to Step 9: Assemble Final
step: 10  -> Go to Step 10: Final Review
```

## Approval Protocol

**CRITICAL: Never advance to the next step without explicit human approval.**

After completing any step, present what was done and ask:
> "Step N complete. Ready to proceed to Step N+1? (approve / reject / redo)"

If the human rejects:
- Ask what needs to change.
- Redo the current step (or the step they specify).
- Present results again and wait for approval.

Loop until approved. Do not guess or assume approval.

## Error Handling

If any `demovid` command exits non-zero, it prints a JSON error to stdout.
Show the full JSON error to the human and ask how to proceed.
Do not retry automatically.

## State Management

After completing each step, update `project.yaml`:
- Set `step` to the next step number.
- Fill in any fields that were produced (e.g., `narration`, `audio_file`, `duration_secs`).
- Write the file back to disk before presenting results to the human.

---

## Step 1: Gather Clips

The human should have placed screen recordings in the `clips/` directory.

1. List files in `clips/`.
2. Probe all clips:
   ```
   demovid probe-all clips/
   ```
3. For each clip, record in `project.yaml` under `clips[]`:
   - `file`: filename (e.g., `clips/intro.mov`)
   - `slug`: short name derived from filename (e.g., `intro`)
   - `duration_secs`: from probe output
   - `narration`: "" (empty, filled in Step 2)
   - `audio_file`: "" (empty, filled in Step 3)
   - `render_file`: "" (empty, filled in Step 6)
   - `approved`: false
4. Set `step: 2` in project.yaml.
5. Present the clip list with durations and wait for approval.

## Step 2: Plan Narration

For each clip, write narration text that fits the clip's duration.

1. If the human hasn't provided narration guidance, ask what each clip should convey.
2. For each clip, draft narration text. Rule of thumb: ~150 words per minute, so a 10-second clip needs ~25 words.
3. Write each narration into `project.yaml` under `clips[i].narration`.
4. Set `step: 3`.
5. Present all narration text and wait for approval.

## Step 3: Generate TTS

For each clip that has narration text:

1. Generate audio:
   ```
   demovid tts "<narration text>" audio/<slug>.wav --voice af_bella
   ```
2. Record `audio_file: audio/<slug>.wav` in project.yaml for each clip.
3. Set `step: 4`.
4. Present the list of generated audio files and wait for approval.

## Step 4: Check Audio

Probe each generated audio file and compare duration to its clip.

1. For each clip:
   ```
   demovid probe audio/<slug>.wav
   ```
2. Compare audio duration to clip duration.
3. Report a table:
   ```
   Clip        | Video Duration | Audio Duration | Status
   intro       | 12.3s          | 10.1s          | OK (audio shorter, will pad)
   feature-1   | 8.0s           | 9.5s           | WARN (audio longer than clip)
   ```
4. Set `step: 5`.
5. Present the table and wait for approval. Flag any clips where audio is longer than video -- these need re-recording or narration editing.

## Step 5: Trim/Pad Clips

For each clip, ensure audio matches video duration:

- **Audio shorter than video**: Generate silence padding and concatenate.
  ```
  demovid silence <gap_seconds> audio/<slug>-pad.wav
  demovid concat-audio audio/<slug>.wav audio/<slug>-pad.wav -o audio/<slug>-final.wav
  ```
- **Audio longer than video**: Do NOT auto-trim. Flag this to the human. Options:
  - Edit narration (go back to Step 2 for this clip).
  - Re-record clip with longer duration.
- **Audio matches video** (within 0.5s): Copy as final.
  ```
  cp audio/<slug>.wav audio/<slug>-final.wav
  ```

1. Process all clips.
2. Update `audio_file` in project.yaml to point to the `-final.wav` version.
3. Set `step: 6`.
4. Present results and wait for approval.

## Step 6: Render Clips

Overlay final audio onto each video clip:

1. For each clip:
   ```
   demovid render-clip clips/<file> audio/<slug>-final.wav renders/<slug>.mp4
   ```
2. Record `render_file: renders/<slug>.mp4` in project.yaml.
3. Set `step: 7`.
4. Present the list of rendered clips and wait for approval.

## Step 7: Review Clips

The human reviews each rendered clip.

1. List all rendered clips with paths so the human can watch them.
2. For each clip, ask: **"Approve or reject?"**
3. Record `approved: true` or `approved: false` in project.yaml for each clip.
4. If all approved: set `step: 9` (skip Step 8).
5. If any rejected: set `step: 8`.
6. Wait for the human to finish reviewing all clips before proceeding.

## Step 8: Record Fixes

For rejected clips only:

1. List the rejected clips and the human's feedback.
2. Go back to Step 2 **for those clips only**. Follow Steps 2-7 for the rejected subset.
3. Reset `approved: false` on the re-done clips so they get reviewed again in Step 7.
4. Once all clips are approved, set `step: 9`.

## Step 9: Assemble Final

Combine all approved rendered clips into one video:

1. Assemble in the order clips appear in `project.yaml`:
   ```
   demovid assemble renders/<slug1>.mp4 renders/<slug2>.mp4 ... -o renders/final.mp4
   ```
2. Record `final_render: renders/final.mp4` in project.yaml.
3. Set `step: 10`.
4. Present the final video path and wait for approval.

## Step 10: Final Review

The human watches the assembled final video.

1. Tell the human: "Final video is at `renders/final.mp4`. Please watch and approve."
2. If approved: mark project complete. Done.
3. If rejected: ask which clips need changes. Set `step: 8` and go to Step 8 with those clips.

---

## CLI Command Reference

| Command | Purpose |
|---------|---------|
| `demovid probe <file>` | Get duration/metadata for one file (JSON output) |
| `demovid probe-all <dir>` | Get duration/metadata for all files in directory |
| `demovid frames <clip> <outdir> --fps 2` | Extract frames for visual analysis |
| `demovid tts "<text>" <output> --voice af_bella` | Generate TTS audio |
| `demovid silence <seconds> <output>` | Generate silent audio file |
| `demovid concat-audio <file1> <file2> ... -o <output>` | Concatenate audio files |
| `demovid render-clip <video> <audio> <output>` | Overlay audio on video |
| `demovid assemble <clip1> <clip2> ... -o <output>` | Assemble clips into final video |

All commands output JSON to stdout on success and JSON errors on failure.
Use `--json` flag if available for machine-readable output.

## Project Structure

```
~/demos/<slug>/
  project.yaml      # Project state (read this first!)
  CLAUDE.md          # These instructions
  AGENTS.md          # Codex agent instructions
  clips/             # Source screen recordings
  audio/             # Generated TTS audio files
  renders/           # Rendered clips and final video
  frames/            # Extracted frames (for visual analysis)
```
