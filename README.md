# audio-workshop-ios
Personal iOS audio editor for trimming, fades, and exporting audio or black-screen video (1080x1920).

## Overview
Audio Workshop iOS is a lightweight, personal MVP for quick audio edits and export. Audio is imported from Files/iCloud or Share Sheet and processed at export-time.

## MVP scope
- Personal audio editing + export utility.
- Audio sources: Files/iCloud/Share Sheet imports only.
- No YouTube downloading/scraping features.

## Features
- Import audio (Files/iCloud).
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
