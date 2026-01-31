# AudioWorkshop
A desktop audio editor (Tauri + AudioMass) focused on fast YouTube-to-audio workflows and simple export, including black-screen vertical video export.

This repo currently reflects the shareable desktop build: **0.1.10**.

## What It Does (Current Status)
- Downloads audio from a YouTube link using bundled sidecar tools.
- Loads the downloaded audio directly into the editor.
- Exports edited audio to a real file path you control.
- Exports black-screen MP4 video (1080x1920, 30fps) using ffmpeg.

## Key Flows
1. Download From YouTube
- Use the Download panel.
- Paste a YouTube URL and click Download.
- The status line shows where the extracted file was saved.

2. Export / Download (Audio)
- Menu: `File -> Export / Download`.
- You can set a persistent Save Folder (Browse + Set).
- After export, the modal shows the saved path.

3. Export Video (Black Screen MP4)
- Menu: `File -> Export Video (Black Screen MP4)`.
- You can choose a custom Save Folder for the export.

## Save Paths And Logs
The app writes data under `%LOCALAPPDATA%\AudioWorkshop` by default.

Key locations:
- Downloads: `%LOCALAPPDATA%\AudioWorkshop\downloads\YYYY-MM-DD\`
- Exports (default): `%USERPROFILE%\Downloads\YYYY-MM-DD\`
- Logs: `%LOCALAPPDATA%\AudioWorkshop\logs\`

Helpful files:
- Latest download log: `%LOCALAPPDATA%\AudioWorkshop\logs\download_*.log`
- Latest video export log: `%LOCALAPPDATA%\AudioWorkshop\logs\video_export_*.log`
- Last resolved download path: `%LOCALAPPDATA%\AudioWorkshop\downloads\YYYY-MM-DD\last_download.txt`

## Release Notes (0.1.10)
- Download UI: default single-video downloads (playlist only if enabled).
- Download UI: Stop button added to cancel downloads.
- App data now stored under `%LOCALAPPDATA%\AudioWorkshop`.
- Default export folder is now `%USERPROFILE%\Downloads\YYYY-MM-DD\`.

## Share With Friends
Preferred installer:
- `src-tauri/target/release/bundle/nsis/Audio Workshop_0.1.10_x64-setup.exe`

Recommended quick test plan:
1. Install to a custom folder.
2. Download a known-good YouTube link.
3. Export audio using `File -> Export / Download` and set a custom Save Folder.
4. Export a black-screen video.

If something fails, ask for:
- The newest `AudioWorkshop/downloads/YYYY-MM-DD/download_*.log`
- The newest `AudioWorkshop/logs/video_export_*.log`


## Repo Hygiene
This repo previously had heavy generated folders checked in locally.

Use the cleanup script when things get bulky:
- PowerShell: `scripts/clean.ps1`
- CMD: `scripts/clean.cmd`

Then reinstall dependencies:
- `npm install`

## Developer Notes
- This repo vendors AudioMass as a git submodule in `vendor/audiomass`.
- The desktop app is implemented with Tauri under `src-tauri`.
- See `docs/WORKFLOW.md` for product and development workflow.

Submodules:
- Clone with submodules:
  - `git clone --recurse-submodules https://github.com/peterliang117/AudioWorkshop.git`
- Initialize/update submodules:
  - `git submodule update --init --recursive`



