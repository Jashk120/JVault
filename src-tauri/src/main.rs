// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tauri::Manager;
 
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Game {
    id: String,
    name: String,
    exe_path: String,
    icon_path: Option<String>,
    ulwgl_id: Option<String>,
    play_time_seconds: u64,
    last_played: Option<String>,
}
 
#[derive(Debug, Serialize, Deserialize)]
struct GameLibrary {
    games: Vec<Game>,
}
 
fn config_path(app: &tauri::AppHandle) -> PathBuf {
    app.path().app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("games.json")
}
 
#[tauri::command]
fn load_games(app: tauri::AppHandle) -> Result<Vec<Game>, String> {
    let path = config_path(&app);
    if !path.exists() {
        return Ok(vec![]);
    }
    let data = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let lib: GameLibrary = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    Ok(lib.games)
}
 
#[tauri::command]
fn save_games(app: tauri::AppHandle, games: Vec<Game>) -> Result<(), String> {
    let path = config_path(&app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let lib = GameLibrary { games };
    let data = serde_json::to_string_pretty(&lib).map_err(|e| e.to_string())?;
    fs::write(&path, data).map_err(|e| e.to_string())?;
    Ok(())
}
 
#[tauri::command]
fn launch_game(exe_path: String, ulwgl_id: Option<String>) -> Result<(), String> {
    let id = ulwgl_id.unwrap_or_else(|| "0".to_string());
    Command::new("umu-run")
        .arg(&exe_path)
        .env("ULWGL_ID", &id)
        .spawn()
        .map_err(|e| format!("Failed to launch: {}", e))?;
    Ok(())
}
 
#[tauri::command]
async fn pick_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let file = app.dialog()
        .file()
        .add_filter("Executable", &["exe", "*"])
        .blocking_pick_file();
    Ok(file.map(|f| f.to_string()))
}
#[tauri::command]
fn extract_icon(app: tauri::AppHandle, exe_path: String) -> Result<Option<String>, String> {
    use std::path::Path;

    let exe = Path::new(&exe_path);
    let icon_dir = app.path().app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("icons");
    fs::create_dir_all(&icon_dir).map_err(|e| e.to_string())?;

    let tmp_ico = "/tmp/jvault_icon.ico";
    let tmp_dir = "/tmp/jvault_icons";
    fs::create_dir_all(tmp_dir).map_err(|e| e.to_string())?;

    let w = Command::new("wrestool")
        .args(["-x", "-t", "14", &exe_path, "-o", tmp_ico])
        .output()
        .map_err(|e| e.to_string())?;
    eprintln!("wrestool status: {}", w.status);
    eprintln!("wrestool stderr: {}", String::from_utf8_lossy(&w.stderr));

    let i = Command::new("icotool")
        .args(["-x", "-o", tmp_dir, tmp_ico])
        .output()
        .map_err(|e| e.to_string())?;
    eprintln!("icotool status: {}", i.status);
    eprintln!("icotool stderr: {}", String::from_utf8_lossy(&i.stderr));

    let mut pngs: Vec<_> = fs::read_dir(tmp_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "png").unwrap_or(false))
        .collect();

    eprintln!("Found {} pngs", pngs.len());

    pngs.sort_by_key(|e| e.file_name());

    if let Some(best) = pngs.last() {
        let final_path = icon_dir.join(format!("{}.png",
            exe.file_stem().unwrap_or_default().to_string_lossy()));
        fs::copy(best.path(), &final_path).map_err(|e| e.to_string())?;
        eprintln!("Icon saved to: {}", final_path.display());
        return Ok(Some(final_path.to_string_lossy().to_string()));
    }

    eprintln!("No icon found!");
    Ok(None)
}
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            #[cfg(debug_assertions)]
            app.get_webview_window("main").unwrap().open_devtools();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_games,
            save_games,
            launch_game,
            pick_file,
            extract_icon,
        ])
        .run(tauri::generate_context!())
        .expect("error while running JVault");
}
