# JVault

## Overview

JVault is a **Tauri-based desktop application** that manages game libraries on Linux. It provides a Rust backend for launching games with ULWGL compatibility, extracting icons from Windows executables, and persisting game metadata.

The frontend (a web-based UI) communicates with the backend through Tauri commands. This document covers only the Rust backend.

## Features

- **Game Library Management**  
  Load and save a list of `Game` objects to a `games.json` file in the platform’s app data directory (`~/.local/share/JVault/`).

- **File Dialog for Adding Games**  
  `pick_file` opens a native file dialog filtered to executables (`*.exe`, `*`).

- **Icon Extraction**  
  `extract_icon` uses `wrestool` (icon type 14) and `icotool` to convert the largest extracted icon into a PNG and saves it to the app’s icons directory.

- **Game Launching**  
  `launch_game` spawns `umu-run` with the executable path and sets the `ULWGL_ID` environment variable (defaults to `"0"`).

- **Debug Mode**  
  In debug builds the log plugin (info level) and developer tools are enabled.

## File Structure

| File | Description |
|------|-------------|
| `src-tauri/build.rs` | Standard Tauri build entry point. |
| `src-tauri/src/lib.rs` | Application entry point – initialises the Tauri builder with optional logging. |
| `src-tauri/src/main.rs` | Core logic – contains data structures, command handlers, and plugin registration. |

## Usage

### `build.rs`

```rust
fn main() {
    tauri_build::build()
}
```

### `lib.rs`

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

*Note: In a debug build the log plugin is registered automatically; in release builds it is omitted.*

### `main.rs`

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tauri::AppHandle;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Game {
    pub id: String,
    pub name: String,
    pub exe_path: String,
    pub icon_path: Option<String>,
    pub ulwgl_id: Option<String>,
    pub play_time_seconds: u64,
    pub last_played: Option<u64>,
}

#[tauri::command]
fn load_games(app: AppHandle) -> Result<Vec<Game>, String> {
    let path = config_path(&app);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&data).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_games(app: AppHandle, games: Vec<Game>) -> Result<(), String> {
    let path = config_path(&app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(&games).map_err(|e| e.to_string())?;
    fs::write(&path, data).map_err(|e| e.to_string())
}

#[tauri::command]
fn launch_game(exe_path: String, ulwgl_id: Option<String>) -> Result<(), String> {
    Command::new("umu-run")
        .env("ULWGL_ID", ulwgl_id.unwrap_or_else(|| "0".to_string()))
        .arg(&exe_path)
        .spawn()
        .map_err(|e| format!("Failed to launch game: {}", e))?;
    Ok(())
}

#[tauri::command]
fn pick_file(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let file = app
        .dialog()
        .file()
        .add_filter("Executables", &["exe", "*"])
        .pick_file();
    Ok(file.map(|f| f.to_string()))
}

#[tauri::command]
fn extract_icon(app: AppHandle, exe_path: String) -> Result<Option<String>, String> {
    let icons_dir = app.path().app_data_dir().map_err(|e| e.to_string())?.join("icons");
    fs::create_dir_all(&icons_dir).map_err(|e| e.to_string())?;

    let output = Command::new("wrestool")
        .args(&["-x", "-t", "14", &exe_path])
        .output()
        .map_err(|e| format!("wrestool failed: {}", e))?;

    let ico_file = icons_dir.join("extracted.ico");
    fs::write(&ico_file, &output.stdout).map_err(|e| e.to_string())?;

    let png_dir = icons_dir.join("png");
    fs::create_dir_all(&png_dir).map_err(|e| e.to_string())?;

    Command::new("icotool")
        .args(&["-x", "--png", ico_file.to_str().unwrap(), "-o", png_dir.to_str().unwrap()])
        .output()
        .map_err(|e| format!("icotool failed: {}", e))?;

    // Find the largest PNG (by filename sorting, largest resolution last)
    let mut png_files: Vec<PathBuf> = fs::read_dir(&png_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.path().extension()? == "png" { Some(entry.path()) } else { None }
        })
        .collect();
    png_files.sort();
    let largest = png_files.last().ok_or("No PNG extracted")?;

    let final_name = format!("{}.png", icons_dir.join(&exe_path).file_stem().unwrap().to_string_lossy());
    fs::rename(largest, &final_name).map_err(|e| e.to_string())?;
    let _ = fs::remove_file(&ico_file);
    let _ = fs::remove_dir_all(&png_dir);

    Ok(Some(final_name.to_string_lossy().to_string()))
}

fn config_path(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir()
        .expect("Failed to get app data dir")
        .join("games.json")
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            load_games,
            save_games,
            launch_game,
            pick_file,
            extract_icon
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                use tauri_plugin_log::LogTarget;
                let _ = app.handle().plugin(
                    tauri_plugin_log::Builder::new()
                        .targets([LogTarget::Stdout, LogTarget::Webview])
                        .build(),
                );
                let _ = app.handle().open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

*Note: The code above is a representative example. In a real project the `main.rs` would also contain the `main()` function from `lib.rs`; here it is consolidated for clarity.*

## Setup

### Prerequisites

- **Operating System:** Linux (tested on distributions that support `/tmp`, `umu-run`, `wrestool`, `icotool`).
- **Rust Toolchain:** Stable Rust with `cargo`.
- **Tauri Development Dependencies:** As required by Tauri 2.x (e.g., `libwebkit2gtk-4.1-dev`, etc.).
- **External Tools:**
  - `umu-run` – required for game launching.
  - `wrestool` – part of `icoutils`.
  - `icotool` – part of `icoutils`.

### Build & Run

```bash
# Clone the repository (assuming this README is part of the repo)
cd jvault

# Build the Tauri application
cargo tauri build

# Or run in development mode
cargo tauri dev
```

The application data directory (`~/.local/share/JVault/`) will be created automatically.

## Notes

- The frontend (web UI) is not documented in this repository. It communicates with the Rust backend via the Tauri commands listed above.
- When running in debug mode, the log plugin and developer tools are enabled automatically.
- The icon extraction workflow depends on `wrestool` and `icotool` being installed and available in `PATH`.
- Game launching relies on `umu-run`; ensure it is installed and configured for your system.
- All file paths are handled using the platform-specific app data directory (e.g., `~/.local/share/JVault/` on Linux).