```rust
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tauri::Manager;

/// Represents a single game entry in the library.
///
/// # Fields
/// * `id` - Unique identifier for the game.
/// * `name` - Display name of the game.
/// * `exe_path` - File path to the game executable.
/// * `icon_path` - Optional file path to the game icon.
/// * `ulwgl_id` - Optional ULWGL identifier for compatibility layers.
/// * `play_time_seconds` - Total play time in seconds.
/// * `last_played` - Optional timestamp of the last time the game was played.
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

/// Represents the entire game library stored as a collection of games.
///
/// # Fields
/// * `games` - A vector containing all `Game` entries in the library.
#[derive(Debug, Serialize, Deserialize)]
struct GameLibrary {
    games: Vec<Game>,
}

/// Returns the file path to the game library configuration file.
///
/// # Arguments
/// * `app` - The Tauri application handle.
///
/// # Returns
/// * `PathBuf` - The path to the `games.json` configuration file within the application data directory.
fn config_path(app: &tauri::AppHandle) -> PathBuf {
    app.path().app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("games.json")
}

/// Loads the game library from the configuration file.
///
/// # Arguments
/// * `app` - The Tauri application handle.
///
/// # Returns
/// * `Ok(Vec<Game>)` - A vector of all games in the library.
/// * `Err(String)` - If the configuration file cannot be read or parsed.
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

/// Saves the game library to the configuration file.
///
/// # Arguments
/// * `app` - The Tauri application handle.
/// * `games` - A vector of `Game` objects to save.
///
/// # Returns
/// * `Ok(())` - If the library was saved successfully.
/// * `Err(String)` - If the directory cannot be created or the file cannot be written.
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

/// Launches a game executable using the UMU runner with optional ULWGL compatibility.
///
/// # Arguments
/// * `exe_path` - The file path to the game executable.
/// * `ulwgl_id` - An optional ULWGL identifier for compatibility layer configuration.
///
/// # Returns
/// * `Ok(())` - If the game process was spawned successfully.
/// * `Err(String)` - If the game process failed to launch.
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

/// Opens a native file dialog to pick an executable file.
///
/// # Arguments
/// * `app` - The Tauri application handle.
///
/// # Returns
/// * `Ok(Option<String>)` - The selected file path, or `None` if the dialog was cancelled.
/// * `Err(String)` - If the dialog failed to open.
#[tauri::command]
async fn pick_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let file = app.dialog()
        .file()
        .add_filter("Executable", &["exe", "*"])
        .blocking_pick_file();
    Ok(file.map(|f| f.to_string()))
}

/// Extracts an icon from a Windows executable file and saves it as a PNG.
///
/// This function uses `wrestool` and `icotool` command-line utilities to extract
/// and convert the icon. The resulting PNG is saved to the application's icon directory.
///
/// # Arguments
/// * `app` - The Tauri application handle.
/// * `exe_path` - The file path to the executable from which to extract the icon.
///
/// # Returns
/// * `Ok(Option<String>)` - The path to the extracted PNG icon, or `None` if the extraction failed.
/// * `Err(String)` - If an error occurs during the extraction process.
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

/// The main entry point for the JVault application.
///
/// Initializes the Tauri application with the shell and dialog plugins,
/// sets up the window, and registers the command handlers.
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
```