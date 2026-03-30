# Demo Video Production — Agent Instructions (Codex)

This project uses `demovid` CLI for demo video production.
See project.yaml for current workflow state.

## Workflow

Follow the 10-step workflow. Never advance without human approval.
Read project.yaml to determine the current step and resume from there.

1. OUTCOME — Human defines what audience should believe
2. SCENARIO — Human defines the plot
3. SHOT LIST — Suggest clips to record, wait for approval
4. NARRATIVE — Help write voiceover text, wait for approval
5. CLIPS — Human records and drops in clips/
6. MAPPING — Map narration segments to clips, wait for approval
7. INDIVIDUAL — Render each clip+narration, wait for approval on each
8. ASSEMBLY — Combine approved clips into final video
9. FINAL REVIEW — Human approves or flags specific clips
10. SHIP

## Commands

- `demovid probe <file>` — Get clip duration
- `demovid probe-all <dir>` — Get all clip durations as JSON
- `demovid frames <clip> <outdir> --fps 2` — Extract frames for analysis
- `demovid tts <text> <output>` — Generate TTS audio
- `demovid silence <seconds> <output>` — Generate silence
- `demovid concat-audio <files...> -o <output>` — Concatenate audio
- `demovid render-clip <video> <audio> <output>` — Render clip with audio
- `demovid assemble <clips...> -o <output>` — Assemble final video
