# audio-workshop-ios
Personal desktop audio editor for trimming, fades, and exporting audio or black-screen video (1080x1920) from a YouTube link. iOS is a future MVP.

## Overview
Audio Workshop is a lightweight, personal MVP for quick audio edits and export. MVP 1 targets desktop; iOS is planned later. Audio is imported from a YouTube link and processed at export-time.

## MVP scope
- Personal audio editing + export utility.
- MVP 1 is a desktop app; iOS is a future MVP.
- Input source: YouTube link (download + extract audio).
- Output: audio file or black-screen video export.

## MVP 2 (current)
- Deterministic YouTube audio downloads (m4a-only, no webm fallback).
- Bundled ffmpeg/ffprobe for extraction (no user dependencies).
- Download UI is isolated from AudioMass via a self-contained component.
- Downloads are grouped by date in `AudioWorkshop/downloads/YYYY-MM-DD/`.
- Full download logs are written to disk only (no UI logs).

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

## Export Video (Black Screen MP4)
- Menu: File → Export Video (Black Screen MP4).
- Output folder: `AudioWorkshop/exports/YYYY-MM-DD/`.
- Filename: `audioworkshop__YYYYMMDD_HHMMSS__1080x1920_30fps__black.mp4`.
- Uses bundled ffmpeg/ffprobe; logs written to `AudioWorkshop/logs/video_export_YYYYMMDD_HHMMSS.log`.

### Video export logs
- Logs live in `AudioWorkshop/logs/video_export_{session}.log`.
- Each line is a JSON trace event with a `stage` field.
- Common stages: `export_clicked`, `precheck_audio_loaded_result`, `wav_export_start`,
  `wav_worker_fetch_test`, `wav_blob_ready`, `backend_ffmpeg_start`, `backend_ffmpeg_exit`,
  `export_success`, `export_failure`.
- If export fails, check the last stage to identify the failure point.

## Export defaults
- Black video size: 1080x1920.
- FPS: 30.
- Container: .mp4.
- Video: H.264.
- Audio: AAC.

## Docs
- `docs/WORKFLOW.md` for product and development workflow.
