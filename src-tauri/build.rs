```rust
/// Builds the Tauri application for the current platform.
///
/// This function serves as the entry point for the build process, invoking
/// Tauri's build system to compile and prepare the application for distribution.
///
/// # Returns
/// * `()` - The function completes the build process without returning a value.
fn main() {
  tauri_build::build()
}
```