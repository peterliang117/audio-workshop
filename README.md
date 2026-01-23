# audio-workshop-ios
Personal desktop audio editor for trimming, fades, and exporting audio or black-screen video (1080x1920) from a YouTube link. iOS is a future MVP.

## Overview
Audio Workshop is a lightweight, personal MVP for quick audio edits and export. MVP 1 targets desktop; iOS is planned later. Audio is imported from a YouTube link and processed at export-time.

## MVP scope
- Personal audio editing + export utility.
- MVP 1 is a desktop app; iOS is a future MVP.
- Input source: YouTube link (download + extract audio).
- Output: audio file or black-screen video export.

## Features
- Import via YouTube link.
- Preview playback with scrubber.
- Basic edits applied at export-time:
  - Trim (start/end).
  - Volume (0.0 ~ 2.0).
  - Fade in/out (0 ~ 5s).
- Export:
  - Audio: m4a (default), wav (optional).
  - Video: black-screen mp4 (1080x1920 portrait, 30fps).

## Export defaults
- Black video size: 1080x1920.
- FPS: 30.
- Container: .mp4.
- Video: H.264.
- Audio: AAC.

## Docs
- `docs/WORKFLOW.md` for product and development workflow.
