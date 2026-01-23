# Workflow

## Product workflow
1) Import audio from a YouTube link.
2) Preview playback and scrub to find edit points.
3) Apply edits (trim, volume, fade in/out).
4) Export:
   - Audio: m4a (default) or wav.
   - Video: black-screen mp4 (1080x1920 portrait, 30fps).

## Development workflow
1) Keep changes aligned with the MVP scope in `CLAUDE.md`.
2) Prefer export-time processing for edits (non-destructive).
3) Validate export defaults (1080x1920, 30fps, H.264 + AAC).
4) Document user-facing behavior updates in `README.md`.
