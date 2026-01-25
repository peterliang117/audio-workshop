# Workflow

## Product workflow
1) Import audio from a YouTube link.
2) Preview playback and scrub to find edit points.
3) Apply edits (trim, volume, fade in/out).
4) Export:
   - Audio: m4a (default) or wav.
   - Video: black-screen mp4 (1080x1920 portrait, 30fps).

## Development workflow
1) Keep changes aligned with the MVP scope in `CLAUDE.md` (MVP 1 is desktop; iOS later).
2) Prefer export-time processing for edits (non-destructive).
3) Start the AudioMass static server before `tauri dev`:
   - `powershell -ExecutionPolicy Bypass -File scripts/dev.ps1`
   - or `cd vendor/audiomass/src` then `python -m http.server 5055`
4) Validate export defaults (1080x1920, 30fps, H.264 + AAC).
5) Document user-facing behavior updates in `README.md`.
