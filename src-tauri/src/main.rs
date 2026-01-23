#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tauri::command]
fn ensure_downloads_dir() -> Result<String, String> {
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

    let downloads = cwd.join("downloads");
    std::fs::create_dir_all(&downloads).map_err(|e| e.to_string())?;
    Ok(downloads.to_string_lossy().to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![ensure_downloads_dir])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
