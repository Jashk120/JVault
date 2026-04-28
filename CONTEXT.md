```markdown
# JVault Repository Context

## Architecture
JVault is a **Tauri-based desktop application** with a Rust backend and a web-based frontend. The backend handles game library management, game launching, icon extraction, and file dialogs. The frontend (not documented here) communicates with the Rust backend via Tauri commands.

## Modules
- **`src-tauri/build.rs`** – Standard Tauri build entry point. Invokes `tauri_build::build()` to compile the application for the current platform.
- **`src-tauri/src/lib.rs`** – Application entry point for Tauri. Initializes the Tauri builder with logging (only in debug mode) and runs the application.
- **`src-tauri/src/main.rs`** – Core logic module. Contains:
  - Data structures (`Game`, `GameLibrary`).
  - Tauri commands for library persistence, file selection, icon extraction, and game launching.
  - Plugin registration (shell, dialog).
  - Devtools opening in debug mode.

## Setup Assumptions
- **Operating System**: Primarily Linux (uses `/tmp` for temporary files, `umu-run`, `wrestool`, `icotool`).
- **External Dependencies**:
  - `umu-run` – Required for launching games with ULWGL compatibility.
  - `wrestool` and `icotool` – Required for extracting icons from Windows executables.
- **Rust Toolchain**: Stable Rust with Tauri dependencies.
- **Application Data**: Stored in the platform’s app data directory (e.g., `~/.local/share/JVault/`).
- **Debug Mode**: In debug builds, the log plugin (info level) and developer tools are enabled.

## Key Flows
1. **Game Library Management**
   - `load_games` – Reads `games.json` from the app data directory and deserializes it into a vector of `Game` objects.
   - `save_games` – Serializes the game vector to `games.json`, creating the parent directory if needed.
2. **Adding a Game**
   - `pick_file` – Opens a native file dialog filtered to executables (`*.exe`, `*`). Returns the selected path.
   - `extract_icon` – Uses `wrestool` to extract an icon from the given executable (type 14), then converts with `icotool` to PNG. Saves the largest PNG to the app’s icons directory.
3. **Launching a Game**
   - `launch_game` – Spawns `umu-run` with the executable path and sets the `ULWGL_ID` environment variable (defaults to `"0"`).
4. **Application Lifecycle**
   - `lib.rs` sets up the Tauri builder, conditionally registers the log plugin.
   - `main.rs` registers the shell and dialog plugins, opens devtools in debug mode, and exposes the command handlers.

## Notable Interfaces
### Tauri Commands (Rust → Frontend)
- `load_games(app: AppHandle) -> Result<Vec<Game>, String>`
- `save_games(app: AppHandle, games: Vec<Game>) -> Result<(), String>`
- `launch_game(exe_path: String, ulwgl_id: Option<String>) -> Result<(), String>`
- `pick_file(app: AppHandle) -> Result<Option<String>, String>`
- `extract_icon(app: AppHandle, exe_path: String) -> Result<Option<String>, String>`

### Key Data Structures
- `Game` – Fields: `id`, `name`, `exe_path`, `icon_path`, `ulwgl_id`, `play_time_seconds`, `last_played`.
- `GameLibrary` – Wrapper around a `Vec<Game>`.

### Helper Function
- `config_path(app: &AppHandle) -> PathBuf` – Returns the path to `games.json` in the app data directory.
```