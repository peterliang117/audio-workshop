#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use tauri::Manager;

#[derive(Debug, Serialize)]
struct DownloadPaths {
    download_root: String,
    download_dir: String,
    log_path: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct Settings {
    download_root: Option<String>,
    export_root: Option<String>,
}

fn app_root() -> Result<PathBuf, String> {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .ok_or("LOCALAPPDATA not set")?;
    let root = base.join("AudioWorkshop");
    std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    Ok(root)
}

fn settings_path() -> Result<PathBuf, String> {
    Ok(app_root()?.join("settings.json"))
}

fn load_settings() -> Result<Settings, String> {
    let path = settings_path()?;
    if !path.exists() {
        return Ok(Settings::default());
    }
    let contents = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&contents).map_err(|e| e.to_string())
}

fn save_settings(settings: &Settings) -> Result<(), String> {
    let path = settings_path()?;
    let contents = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, contents).map_err(|e| e.to_string())
}

fn default_download_root() -> Result<PathBuf, String> {
    Ok(app_root()?.join("downloads"))
}

fn logs_root() -> Result<PathBuf, String> {
    Ok(app_root()?.join("logs"))
}

fn default_export_root() -> Result<PathBuf, String> {
    let home = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .ok_or("USERPROFILE not set")?;
    Ok(home.join("Downloads"))
}

fn tmp_root() -> Result<PathBuf, String> {
    Ok(app_root()?.join("tmp"))
}

fn resolve_download_root() -> Result<PathBuf, String> {
    let settings = load_settings()?;
    if let Some(root) = settings.download_root {
        let path = PathBuf::from(root);
        if path.is_absolute() {
            return Ok(path);
        }
        return Ok(app_root()?.join(path));
    }
    default_download_root()
}

fn resolve_export_root() -> Result<PathBuf, String> {
    let settings = load_settings()?;
    if let Some(root) = settings.export_root {
        let path = PathBuf::from(root);
        if path.is_absolute() {
            return Ok(path);
        }
        return Ok(app_root()?.join(path));
    }
    default_export_root()
}

fn validate_writable_dir(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
    let probe = path.join(".aw_write_test");
    std::fs::write(&probe, b"test").map_err(|e| e.to_string())?;
    std::fs::remove_file(&probe).map_err(|e| e.to_string())?;
    Ok(())
}

fn is_within(parent: &Path, child: &Path) -> Result<bool, String> {
    let parent = parent
        .canonicalize()
        .map_err(|e| format!("Path error: {e}"))?;
    let child_parent = child
        .parent()
        .ok_or("Invalid path")?
        .canonicalize()
        .map_err(|e| format!("Path error: {e}"))?;
    Ok(child_parent.starts_with(parent))
}

fn append_video_trace_line(session_id: &str, line: &str) -> Result<(), String> {
    if !session_id
        .chars()
        .all(|c| c.is_ascii_digit() || c == '_')
    {
        return Err("Invalid session id".into());
    }
    let logs = logs_root()?;
    std::fs::create_dir_all(&logs).map_err(|e| e.to_string())?;
    let log_path = logs.join(format!("video_export_{}.log", session_id));
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .map_err(|e| e.to_string())?;
    use std::io::Write;
    writeln!(file, "{line}").map_err(|e| e.to_string())
}

fn binaries_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let mut diag: Vec<String> = Vec::new();

    match app.path().resource_dir() {
        Ok(resource_dir) => {
            diag.push(format!(
                "resource_dir={}",
                resource_dir.to_string_lossy()
            ));

            // Tauri sidecars can land either in `resources/binaries` or directly
            // in the `resources` (or even the exe) directory depending on build
            // and installer behavior. Check all common locations.
            let parent_dir = resource_dir
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| resource_dir.clone());
            let candidates = [
                resource_dir.join("binaries"),
                resource_dir.clone(),
                parent_dir,
                resource_dir.join("..").join("binaries"),
            ];
            for candidate in candidates {
                let has_bins = has_required_binaries(&candidate);
                diag.push(format!(
                    "candidate(resource)={} exists={} has_bins={}",
                    candidate.to_string_lossy(),
                    candidate.exists(),
                    has_bins
                ));
                if has_bins {
                    return candidate.canonicalize().map_err(|e| {
                        format!(
                            "Binaries found but canonicalize failed: {e}. diag={}",
                            diag.join(" | ")
                        )
                    });
                }
            }
        }
        Err(err) => {
            diag.push(format!("resource_dir_error={err}"));
        }
    }

    match std::env::current_exe() {
        Ok(exe_path) => {
            diag.push(format!("current_exe={}", exe_path.to_string_lossy()));
            if let Some(exe_dir) = exe_path.parent() {
                diag.push(format!("current_exe_dir={}", exe_dir.to_string_lossy()));
                if let Some(candidate) = find_binaries_dir(exe_dir) {
                    diag.push(format!(
                        "candidate(exe_ancestors)={}",
                        candidate.to_string_lossy()
                    ));
                    return candidate.canonicalize().map_err(|e| {
                        format!(
                            "Binaries found via exe ancestors but canonicalize failed: {e}. diag={}",
                            diag.join(" | ")
                        )
                    });
                }
                diag.push("candidate(exe_ancestors)=none".into());
            }
        }
        Err(err) => {
            diag.push(format!("current_exe_error={err}"));
        }
    }

    let cwd = std::env::current_dir().map_err(|e| {
        format!(
            "Unable to resolve current_dir: {e}. diag={}",
            diag.join(" | ")
        )
    })?;
    diag.push(format!("current_dir={}", cwd.to_string_lossy()));
    if let Some(candidate) = find_binaries_dir(&cwd) {
        diag.push(format!(
            "candidate(cwd_ancestors)={}",
            candidate.to_string_lossy()
        ));
        return candidate.canonicalize().map_err(|e| {
            format!(
                "Binaries found via cwd ancestors but canonicalize failed: {e}. diag={}",
                diag.join(" | ")
            )
        });
    }
    diag.push("candidate(cwd_ancestors)=none".into());

    // As a last resort, try to repair the sidecar layout in the install folder
    // by creating a `binaries` directory and copying/renaming any tools that
    // were placed next to the executable.
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            diag.push(format!("repair_attempt_dir={}", exe_dir.to_string_lossy()));
            match repair_binaries_layout(exe_dir) {
                Ok(Some(repaired_dir)) => {
                    diag.push(format!(
                        "repair_success_dir={}",
                        repaired_dir.to_string_lossy()
                    ));
                    return repaired_dir.canonicalize().map_err(|e| {
                        format!(
                            "Binaries repaired but canonicalize failed: {e}. diag={}",
                            diag.join(" | ")
                        )
                    });
                }
                Ok(None) => {
                    diag.push("repair_result=none".into());
                }
                Err(err) => {
                    diag.push(format!("repair_error={err}"));
                }
            }
        }
    }

    Err(format!(
        "Binaries directory not found. diag={}",
        diag.join(" | ")
    ))
}

fn find_binaries_dir(start: &Path) -> Option<PathBuf> {
    for ancestor in start.ancestors() {
        let direct = ancestor.join("binaries");
        if has_required_binaries(&direct) {
            return Some(direct);
        }
        if has_required_binaries(ancestor) {
            return Some(ancestor.to_path_buf());
        }
        let src_tauri = ancestor.join("src-tauri").join("binaries");
        if has_required_binaries(&src_tauri) {
            return Some(src_tauri);
        }
        let resources = ancestor.join("resources");
        if has_required_binaries(&resources) {
            return Some(resources);
        }
        let resources_binaries = resources.join("binaries");
        if has_required_binaries(&resources_binaries) {
            return Some(resources_binaries);
        }
    }
    None
}

fn has_required_binaries(dir: &Path) -> bool {
    let ffmpeg = dir.join("ffmpeg.exe");
    let ffmpeg_triple = dir.join("ffmpeg-x86_64-pc-windows-msvc.exe");
    let ffprobe = dir.join("ffprobe.exe");
    let ffprobe_triple = dir.join("ffprobe-x86_64-pc-windows-msvc.exe");
    let yt_dlp = dir.join("yt-dlp.exe");
    let yt_dlp_triple = dir.join("yt-dlp-x86_64-pc-windows-msvc.exe");
    ffmpeg.exists()
        || ffmpeg_triple.exists()
        || ffprobe.exists()
        || ffprobe_triple.exists()
        || yt_dlp.exists()
        || yt_dlp_triple.exists()
}

fn first_existing(paths: &[PathBuf]) -> Option<PathBuf> {
    for path in paths {
        if path.exists() {
            return Some(path.clone());
        }
    }
    None
}

fn repair_binaries_layout(exe_dir: &Path) -> Result<Option<PathBuf>, String> {
    let binaries_dir = exe_dir.join("binaries");
    std::fs::create_dir_all(&binaries_dir).map_err(|e| e.to_string())?;

    let ffmpeg_src = first_existing(&[
        exe_dir.join("ffmpeg-x86_64-pc-windows-msvc.exe"),
        exe_dir.join("ffmpeg.exe"),
    ]);
    let ffprobe_src = first_existing(&[
        exe_dir.join("ffprobe-x86_64-pc-windows-msvc.exe"),
        exe_dir.join("ffprobe.exe"),
    ]);
    let yt_dlp_src = first_existing(&[
        exe_dir.join("yt-dlp-x86_64-pc-windows-msvc.exe"),
        exe_dir.join("yt-dlp.exe"),
    ]);

    let mut copied_any = false;

    if let Some(src) = ffmpeg_src {
        let dest = binaries_dir.join("ffmpeg-x86_64-pc-windows-msvc.exe");
        if !dest.exists() {
            std::fs::copy(&src, &dest).map_err(|e| e.to_string())?;
            copied_any = true;
        }
    }
    if let Some(src) = ffprobe_src {
        let dest = binaries_dir.join("ffprobe-x86_64-pc-windows-msvc.exe");
        if !dest.exists() {
            std::fs::copy(&src, &dest).map_err(|e| e.to_string())?;
            copied_any = true;
        }
    }
    if let Some(src) = yt_dlp_src {
        let dest = binaries_dir.join("yt-dlp-x86_64-pc-windows-msvc.exe");
        if !dest.exists() {
            std::fs::copy(&src, &dest).map_err(|e| e.to_string())?;
            copied_any = true;
        }
    }

    if has_required_binaries(&binaries_dir) {
        return Ok(Some(binaries_dir));
    }

    if copied_any {
        return Err("Attempted repair but required binaries still missing".into());
    }

    Ok(None)
}

fn ffmpeg_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let bin_dir = binaries_dir(app)?;
    let candidates = [
        bin_dir.join("ffmpeg-x86_64-pc-windows-msvc.exe"),
        bin_dir.join("ffmpeg.exe"),
    ];
    for candidate in candidates {
        if candidate.exists() {
            return candidate
                .canonicalize()
                .map_err(|e| e.to_string());
        }
    }
    Err("ffmpeg executable not found".into())
}

#[tauri::command]
fn get_download_root() -> Result<String, String> {
    let root = resolve_download_root()?;
    validate_writable_dir(&root)?;
    Ok(root.to_string_lossy().to_string())
}

#[tauri::command]
fn set_download_root(path: String) -> Result<String, String> {
    let mut settings = load_settings()?;
    if path.trim().is_empty() {
        settings.download_root = None;
        save_settings(&settings)?;
        return get_download_root();
    }

    let candidate = {
        let raw = PathBuf::from(path.trim());
        if raw.is_absolute() {
            raw
        } else {
            app_root()?.join(raw)
        }
    };

    validate_writable_dir(&candidate)?;
    settings.download_root = Some(candidate.to_string_lossy().to_string());
    save_settings(&settings)?;
    get_download_root()
}

#[tauri::command]
fn ensure_downloads_dir(date_folder: String) -> Result<String, String> {
    if !date_folder
        .chars()
        .all(|c| c.is_ascii_digit() || c == '-')
    {
        return Err("Invalid date folder".into());
    }

    let root = resolve_download_root()?;
    validate_writable_dir(&root)?;

    let download_dir = root.join(date_folder);
    std::fs::create_dir_all(&download_dir).map_err(|e| e.to_string())?;
    Ok(download_dir.to_string_lossy().to_string())
}

#[tauri::command(rename_all = "camelCase")]
fn prepare_temp_audio(date_folder: String, log_stamp: String) -> Result<String, String> {
    if !date_folder
        .chars()
        .all(|c| c.is_ascii_digit() || c == '-')
    {
        return Err("Invalid date folder".into());
    }
    if !log_stamp
        .chars()
        .all(|c| c.is_ascii_digit() || c == '_')
    {
        return Err("Invalid log stamp".into());
    }

    let root = tmp_root()?;
    validate_writable_dir(&root)?;

    let tmp_dir = root.join(date_folder);
    std::fs::create_dir_all(&tmp_dir).map_err(|e| e.to_string())?;

    let file_name = format!("audioworkshop__{}.wav", log_stamp);
    let path = tmp_dir.join(file_name);
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command(rename_all = "camelCase")]
fn write_binary_file(path: String, bytes: Vec<u8>) -> Result<(), String> {
    let root = app_root()?;
    let path = PathBuf::from(path);
    if !is_within(&root, &path)? {
        return Err("Invalid output path".into());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, bytes).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "camelCase")]
fn export_black_video(
    app: tauri::AppHandle,
    input_audio_path: String,
    session_id: String,
    output_root: Option<String>,
) -> Result<String, String> {
    let now = Local::now();
    let date_folder = now.format("%Y-%m-%d").to_string();
    let stamp = session_id.clone();

    let root = app_root()?;
    let input_path = PathBuf::from(input_audio_path);
    if !is_within(&root, &input_path)? {
        let _ = append_video_trace_line(&session_id, "{\"stage\":\"backend_export_video_start\",\"error\":\"invalid_input_path\"}");
        return Err("Invalid input path".into());
    }

    let custom_root = output_root.is_some();
    let output_root = if let Some(root) = output_root {
        let raw = PathBuf::from(root);
        if raw.is_absolute() {
            raw
        } else {
            app_root()?.join(raw)
        }
    } else {
        resolve_export_root()?
    };
    if let Err(err) = validate_writable_dir(&output_root) {
        let _ = append_video_trace_line(&session_id, &format!("{{\"stage\":\"backend_export_video_start\",\"error\":\"{}\"}}", err));
        return Err("Export failed. See logs.".into());
    }

    let export_dir = if custom_root {
        output_root.clone()
    } else {
        output_root.join(&date_folder)
    };
    if let Err(err) = std::fs::create_dir_all(&export_dir).map_err(|e| e.to_string()) {
        let _ = append_video_trace_line(&session_id, &format!("{{\"stage\":\"backend_export_video_start\",\"error\":\"{}\"}}", err));
        return Err("Export failed. See logs.".into());
    }

    let file_name = format!("audioworkshop__{}__1080x1920_30fps__black.mp4", stamp);
    let output_path = export_dir.join(file_name);

    let _ = append_video_trace_line(
        &session_id,
        &format!(
            "{{\"stage\":\"backend_export_video_start\",\"input\":\"{}\"}}",
            input_path.to_string_lossy()
        ),
    );

    let ffmpeg = match ffmpeg_path(&app) {
        Ok(path) => path,
        Err(err) => {
            let _ = append_video_trace_line(&session_id, &format!("{{\"stage\":\"backend_export_video_start\",\"error\":\"{}\"}}", err));
            return Err("Export failed. See logs.".into());
        }
    };
    let input_path_lossy = input_path.to_string_lossy();
    let output_path_lossy = output_path.to_string_lossy();
    let args = [
        "-y",
        "-f",
        "lavfi",
        "-i",
        "color=black:s=1080x1920:r=30",
        "-i",
        input_path_lossy.as_ref(),
        "-shortest",
        "-c:v",
        "libx264",
        "-pix_fmt",
        "yuv420p",
        "-r",
        "30",
        "-c:a",
        "aac",
        "-ar",
        "44100",
        "-ac",
        "2",
        "-b:a",
        "192k",
        "-movflags",
        "+faststart",
        output_path_lossy.as_ref(),
    ];

    let _ = append_video_trace_line(
        &session_id,
        &format!(
            "{{\"stage\":\"backend_ffmpeg_start\",\"args\":\"{}\"}}",
            args.join(" ")
        ),
    );

    let output = std::process::Command::new(ffmpeg).args(args).output();
    let output = match output {
        Ok(output) => output,
        Err(err) => {
            let _ = append_video_trace_line(&session_id, &format!("{{\"stage\":\"backend_ffmpeg_exit\",\"error\":\"{}\"}}", err));
            return Err("Export failed. See logs.".into());
        }
    };

    let mut log_text = String::new();
    log_text.push_str(&String::from_utf8_lossy(&output.stdout));
    log_text.push_str(&String::from_utf8_lossy(&output.stderr));
    let tail_lines: Vec<&str> = log_text.lines().rev().take(50).collect();
    let tail_joined = tail_lines.into_iter().rev().collect::<Vec<&str>>().join("\\n");
    let _ = append_video_trace_line(
        &session_id,
        &format!(
            "{{\"stage\":\"backend_ffmpeg_exit\",\"code\":{},\"tail\":{}}}",
            output.status.code().unwrap_or(-1),
            serde_json::to_string(&tail_joined).unwrap_or_default()
        ),
    );
    let _ = append_video_trace_line(&session_id, &log_text);

    if !output.status.success() {
        return Err("Export failed. See logs.".into());
    }

    Ok(output_path.to_string_lossy().to_string())
}

#[tauri::command]
fn get_binaries_dir(app: tauri::AppHandle) -> Result<String, String> {
    let dir = binaries_dir(&app)?;
    Ok(dir.to_string_lossy().to_string())
}

#[tauri::command]
fn get_export_root() -> Result<String, String> {
    let root = resolve_export_root()?;
    validate_writable_dir(&root)?;
    Ok(root.to_string_lossy().to_string())
}

#[tauri::command]
fn set_export_root(path: String) -> Result<String, String> {
    let mut settings = load_settings()?;
    if path.trim().is_empty() {
        settings.export_root = None;
        save_settings(&settings)?;
        return get_export_root();
    }

    let candidate = {
        let raw = PathBuf::from(path.trim());
        if raw.is_absolute() {
            raw
        } else {
            app_root()?.join(raw)
        }
    };

    validate_writable_dir(&candidate)?;
    settings.export_root = Some(candidate.to_string_lossy().to_string());
    save_settings(&settings)?;
    get_export_root()
}

#[tauri::command]
fn prepare_download(date_folder: String, log_stamp: String) -> Result<DownloadPaths, String> {
    if !date_folder
        .chars()
        .all(|c| c.is_ascii_digit() || c == '-')
    {
        return Err("Invalid date folder".into());
    }
    if !log_stamp
        .chars()
        .all(|c| c.is_ascii_digit() || c == '_')
    {
        return Err("Invalid log stamp".into());
    }

    let root = resolve_download_root()?;
    validate_writable_dir(&root)?;

    let download_dir = root.join(date_folder);
    std::fs::create_dir_all(&download_dir).map_err(|e| e.to_string())?;

    let logs = logs_root()?;
    std::fs::create_dir_all(&logs).map_err(|e| e.to_string())?;
    let log_path = logs.join(format!("download_{}.log", log_stamp));

    Ok(DownloadPaths {
        download_root: root.to_string_lossy().to_string(),
        download_dir: download_dir.to_string_lossy().to_string(),
        log_path: log_path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
fn write_download_log(path: String, contents: String) -> Result<(), String> {
    let root = resolve_download_root()?;
    let path = PathBuf::from(path);
    if !is_within(&root, &path)? {
        return Err("Invalid log path".into());
    }
    std::fs::write(path, contents).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "camelCase")]
fn write_video_log(log_stamp: String, contents: String) -> Result<(), String> {
    if !log_stamp
        .chars()
        .all(|c| c.is_ascii_digit() || c == '_')
    {
        return Err("Invalid log stamp".into());
    }
    let logs = logs_root()?;
    std::fs::create_dir_all(&logs).map_err(|e| e.to_string())?;
    let log_path = logs.join(format!("video_export_{}.log", log_stamp));
    std::fs::write(log_path, contents).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "camelCase")]
fn append_video_trace(session_id: String, line: String) -> Result<(), String> {
    append_video_trace_line(&session_id, &line)
}

#[tauri::command]
fn write_meta_file(path: String, contents: String) -> Result<(), String> {
    let root = resolve_download_root()?;
    let path = PathBuf::from(path);
    if !is_within(&root, &path)? {
        return Err("Invalid metadata path".into());
    }
    std::fs::write(path, contents).map_err(|e| e.to_string())
}

#[tauri::command]
fn read_downloaded_file(path: String) -> Result<Vec<u8>, String> {
    let root = resolve_download_root()?;
    let path = PathBuf::from(path);
    if !is_within(&root, &path)? {
        return Err("Invalid download path".into());
    }
    std::fs::read(path).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "camelCase")]
fn find_latest_download(download_dir: String) -> Result<String, String> {
    let root = resolve_download_root()?;
    let dir = PathBuf::from(download_dir);
    if !is_within(&root, &dir.join("probe.txt"))? {
        return Err("Invalid download directory".into());
    }

    let entries = std::fs::read_dir(&dir).map_err(|e| e.to_string())?;
    let mut candidates: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("m4a"))
                .unwrap_or(false)
        })
        .collect();

    candidates.sort_by_key(|path| {
        std::fs::metadata(path)
            .and_then(|meta| meta.modified())
            .ok()
    });

    let latest = candidates.pop().ok_or("No downloaded file found")?;
    let canonical = latest.canonicalize().map_err(|e| e.to_string())?;
    Ok(canonical.to_string_lossy().to_string())
}

fn sanitized_file_name(name: &str, fallback_ext: &str) -> String {
    let candidate = Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("audioworkshop-output");
    if candidate.trim().is_empty() {
        return format!("audioworkshop-output.{fallback_ext}");
    }
    candidate.to_string()
}

#[tauri::command(rename_all = "camelCase")]
fn export_audio_file(
    file_name: String,
    format: String,
    bytes: Vec<u8>,
    output_root: Option<String>,
) -> Result<String, String> {
    let now = Local::now();
    let date_folder = now.format("%Y-%m-%d").to_string();

    let fallback_ext = if format.trim().is_empty() {
        "mp3"
    } else {
        format.trim()
    };
    let file_name = sanitized_file_name(&file_name, fallback_ext);

    let custom_root = output_root.is_some();
    let output_root = if let Some(root) = output_root {
        let raw = PathBuf::from(root);
        if raw.is_absolute() {
            raw
        } else {
            app_root()?.join(raw)
        }
    } else {
        resolve_export_root()?
    };
    validate_writable_dir(&output_root)?;

    let export_dir = if custom_root {
        output_root.clone()
    } else {
        output_root.join(date_folder)
    };
    std::fs::create_dir_all(&export_dir).map_err(|e| e.to_string())?;

    let output_path = export_dir.join(file_name);
    std::fs::write(&output_path, bytes).map_err(|e| e.to_string())?;
    Ok(output_path.to_string_lossy().to_string())
}

fn collect_files_recursively(root: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursively(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn latest_file_with_prefix(root: &Path, prefix: &str) -> Option<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();
    collect_files_recursively(root, &mut files);
    let mut candidates: Vec<PathBuf> = files
        .into_iter()
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with(prefix))
                .unwrap_or(false)
        })
        .collect();
    candidates.sort_by_key(|p| {
        std::fs::metadata(p)
            .and_then(|m| m.modified())
            .ok()
    });
    candidates.pop()
}

fn tail_lines(path: &Path, max_lines: usize) -> String {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(err) => return format!("(unable to read {}: {err})", path.to_string_lossy()),
    };
    let lines: Vec<&str> = text.lines().rev().take(max_lines).collect();
    lines.into_iter().rev().collect::<Vec<&str>>().join("\n")
}

#[tauri::command]
fn write_support_bundle(app: tauri::AppHandle) -> Result<String, String> {
    let logs = logs_root()?;
    std::fs::create_dir_all(&logs).map_err(|e| e.to_string())?;

    let stamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let bundle_path = logs.join(format!("support_bundle_{stamp}.txt"));

    let app_root_text = app_root()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|e| format!("(error: {e})"));
    let resource_dir_text = app
        .path()
        .resource_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|e| format!("(error: {e})"));
    let current_exe_text = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|e| format!("(error: {e})"));
    let current_dir_text = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|e| format!("(error: {e})"));

    let binaries_result = binaries_dir(&app)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|e| format!("(error: {e})"));

    let download_root = resolve_download_root()?;
    let latest_download = latest_file_with_prefix(&download_root, "download_");
    let latest_download_text = latest_download
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "(none found)".into());
    let latest_download_tail = latest_download
        .as_ref()
        .map(|p| tail_lines(p, 120))
        .unwrap_or_else(|| "(no download log tail)".into());

    let latest_video = latest_file_with_prefix(&logs, "video_export_");
    let latest_video_text = latest_video
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "(none found)".into());
    let latest_video_tail = latest_video
        .as_ref()
        .map(|p| tail_lines(p, 120))
        .unwrap_or_else(|| "(no video log tail)".into());

    let contents = format!(
        "Audio Workshop Support Bundle\n\
generated_at={stamp}\n\n\
[paths]\n\
app_root={app_root_text}\n\
resource_dir={resource_dir_text}\n\
current_exe={current_exe_text}\n\
current_dir={current_dir_text}\n\
binaries_dir={binaries_result}\n\n\
[latest_download_log]\n\
path={latest_download_text}\n\
{latest_download_tail}\n\n\
[latest_video_log]\n\
path={latest_video_text}\n\
{latest_video_tail}\n",
        stamp = stamp,
        app_root_text = app_root_text,
        resource_dir_text = resource_dir_text,
        current_exe_text = current_exe_text,
        current_dir_text = current_dir_text,
        binaries_result = binaries_result,
        latest_download_text = latest_download_text,
        latest_download_tail = latest_download_tail,
        latest_video_text = latest_video_text,
        latest_video_tail = latest_video_tail
    );

    std::fs::write(&bundle_path, contents).map_err(|e| e.to_string())?;
    Ok(bundle_path.to_string_lossy().to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_download_root,
            set_download_root,
            ensure_downloads_dir,
            get_binaries_dir,
            get_export_root,
            set_export_root,
            prepare_temp_audio,
            write_binary_file,
            export_black_video,
            write_video_log,
            append_video_trace,
            prepare_download,
            write_download_log,
            write_support_bundle,
            export_audio_file,
            write_meta_file,
            read_downloaded_file,
            find_latest_download
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
