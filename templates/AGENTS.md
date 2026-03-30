# Demo Video Production -- Agent Instructions (Codex)

Produce a demo video using `demovid` CLI. State is in `project.yaml`.

## Core Rules

1. Read `project.yaml` first. Resume from the `step` field.
2. Never advance without explicit human approval.
3. On rejection: redo the step, present results, wait for approval again.
4. On command failure: show the JSON error, ask the human how to proceed.
5. After each step: update `project.yaml` with results and next step number.

## 10-Step Workflow

### Step 1: Gather Clips
Probe source clips in `clips/`:
```bash
demovid probe-all clips/
```
Populate `clips[]` in project.yaml with `file`, `slug`, `duration_secs`. Set `step: 2`.

### Step 2: Plan Narration
Draft narration for each clip (~150 words/minute). Write to `clips[i].narration`. Set `step: 3`.

### Step 3: Generate TTS
```bash
demovid tts "<narration>" audio/<slug>.wav --voice af_bella
```
Run for each clip. Record `audio_file`. Set `step: 4`.

### Step 4: Check Audio
```bash
demovid probe audio/<slug>.wav
```
Compare audio vs video duration. Report table. Flag audio-longer-than-video. Set `step: 5`.

### Step 5: Trim/Pad
- Audio shorter: `demovid silence <gap> audio/<slug>-pad.wav` then `demovid concat-audio audio/<slug>.wav audio/<slug>-pad.wav -o audio/<slug>-final.wav`
- Audio longer: flag to human (edit narration or re-record clip).
- Audio matches (within 0.5s): `cp audio/<slug>.wav audio/<slug>-final.wav`

Update `audio_file` to `-final.wav`. Set `step: 6`.

### Step 6: Render Clips
```bash
demovid render-clip clips/<file> audio/<slug>-final.wav renders/<slug>.mp4
```
Record `render_file`. Set `step: 7`.

### Step 7: Review Clips
Present each rendered clip path. Human approves/rejects each. Record `approved` boolean.
All approved -> `step: 9`. Any rejected -> `step: 8`.

### Step 8: Record Fixes
Redo Steps 2-7 for rejected clips only. Once all approved, set `step: 9`.

### Step 9: Assemble
```bash
demovid assemble renders/<slug1>.mp4 renders/<slug2>.mp4 ... -o renders/final.mp4
```
Record `final_render`. Set `step: 10`.

### Step 10: Final Review
Human watches `renders/final.mp4`. Approved -> done. Rejected -> `step: 8` for flagged clips.

## Commands

| Command | Purpose |
|---------|---------|
| `demovid probe <file>` | File metadata (JSON) |
| `demovid probe-all <dir>` | All files metadata |
| `demovid tts "<text>" <out> --voice af_bella` | Generate TTS |
| `demovid silence <secs> <out>` | Generate silence |
| `demovid concat-audio <files...> -o <out>` | Concat audio |
| `demovid render-clip <video> <audio> <out>` | Render clip |
| `demovid assemble <clips...> -o <out>` | Assemble final |
