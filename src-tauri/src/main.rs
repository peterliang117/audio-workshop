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
}

fn app_root() -> Result<PathBuf, String> {
    let mut cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    if cwd
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.eq_ignore_ascii_case("src-tauri"))
        .unwrap_or(false)
    {
        if let Some(parent) = cwd.parent() {
            cwd = parent.to_path_buf();
        }
    }
    let root = cwd.join("AudioWorkshop");
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

fn exports_root() -> Result<PathBuf, String> {
    Ok(app_root()?.join("exports"))
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
    if let Ok(resource_dir) = app.path().resource_dir() {
        let candidate = resource_dir.join("binaries");
        if candidate.exists() {
            return candidate
                .canonicalize()
                .map_err(|e| e.to_string());
        }
    }

    let mut cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    if !cwd
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.eq_ignore_ascii_case("src-tauri"))
        .unwrap_or(false)
    {
        cwd = cwd.join("src-tauri");
    }

    let candidate = cwd.join("binaries");
    if candidate.exists() {
        return candidate
            .canonicalize()
            .map_err(|e| e.to_string());
    }

    Err("Binaries directory not found".into())
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
        exports_root()?
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
    let root = exports_root()?;
    validate_writable_dir(&root)?;
    Ok(root.to_string_lossy().to_string())
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
            prepare_temp_audio,
            write_binary_file,
            export_black_video,
            write_video_log,
            append_video_trace,
            prepare_download,
            write_download_log,
            write_meta_file,
            read_downloaded_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
