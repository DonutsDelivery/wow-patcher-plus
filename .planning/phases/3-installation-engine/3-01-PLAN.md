---
phase: 3-installation-engine
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src-tauri/Cargo.toml
  - src-tauri/src/lib.rs
  - src-tauri/src/install/mod.rs
  - src-tauri/src/install/detector.rs
  - src-tauri/src/install/settings.rs
  - src-tauri/capabilities/default.json
autonomous: true
user_setup: []

must_haves:
  truths:
    - "App can detect if current directory is a valid WoW installation"
    - "App validates user-selected folders as valid WoW installations"
    - "App persists WoW folder path between sessions"
  artifacts:
    - path: "src-tauri/src/install/detector.rs"
      provides: "WoW folder validation logic"
      exports: ["WowDetector", "is_valid_wow_folder", "auto_detect", "get_data_folder"]
    - path: "src-tauri/src/install/settings.rs"
      provides: "Settings persistence via Tauri store"
      exports: ["Settings", "get_wow_path", "set_wow_path"]
    - path: "src-tauri/src/install/mod.rs"
      provides: "Module exports"
  key_links:
    - from: "src-tauri/src/install/settings.rs"
      to: "tauri-plugin-store"
      via: "StoreExt trait"
      pattern: "app\\.store"
    - from: "src-tauri/src/lib.rs"
      to: "tauri-plugin-dialog"
      via: "plugin registration"
      pattern: "tauri_plugin_dialog::init"
---

<objective>
Add Tauri plugins and create installation foundation with WoW folder detection and settings persistence.

Purpose: Enable the app to detect valid WoW installations, let users select their WoW folder, and remember the path between sessions.

Output:
- install/ module with detector.rs and settings.rs
- Tauri plugins (dialog, store) registered
- WoW folder validation working
</objective>

<execution_context>
@./.claude/get-shit-done/workflows/execute-plan.md
@./.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/3-installation-engine/3-RESEARCH.md

# Existing code structure
@src-tauri/Cargo.toml
@src-tauri/src/lib.rs
@src-tauri/src/download/mod.rs
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add Tauri plugins and create install module scaffold</name>
  <files>
    - src-tauri/Cargo.toml
    - src-tauri/src/lib.rs
    - src-tauri/src/install/mod.rs
    - src-tauri/capabilities/default.json
  </files>
  <action>
    Add dependencies to Cargo.toml:
    ```toml
    tauri-plugin-dialog = "2"
    tauri-plugin-store = "2"
    ```

    Create src-tauri/src/install/mod.rs with:
    ```rust
    //! Installation engine for HD patch files
    //!
    //! This module provides:
    //! - WoW folder detection and validation
    //! - Settings persistence for WoW path
    //! - MPQ file copy operations (added in later plans)

    pub mod detector;
    pub mod settings;

    // Re-exports added as modules are created
    pub use detector::WowDetector;
    pub use settings::Settings;
    ```

    Update src-tauri/src/lib.rs:
    - Add `mod install;` declaration
    - Register plugins in Tauri builder:
      ```rust
      .plugin(tauri_plugin_dialog::init())
      .plugin(tauri_plugin_store::init())
      ```

    Update src-tauri/capabilities/default.json to add dialog and store permissions:
    - Add "dialog:default" to permissions array
    - Add "store:default" to permissions array
  </action>
  <verify>
    `cargo check -p turtle-wow-hd-patcher` compiles without errors
  </verify>
  <done>
    Tauri plugins registered, install module scaffold created, capabilities updated
  </done>
</task>

<task type="auto">
  <name>Task 2: Implement WoW folder detection and validation</name>
  <files>src-tauri/src/install/detector.rs</files>
  <action>
    Create detector.rs with WowDetector struct:

    ```rust
    //! WoW installation folder detection and validation

    use std::path::{Path, PathBuf};

    /// Detects and validates WoW installation folders
    pub struct WowDetector;

    impl WowDetector {
        /// Check if a path is a valid WoW installation
        ///
        /// Valid if it has:
        /// - One of: WoW.exe, WoWFoV.exe, turtle-wow.exe
        /// - A Data subdirectory
        pub fn is_valid_wow_folder(path: &Path) -> bool {
            let has_exe = path.join("WoW.exe").exists()
                || path.join("WoWFoV.exe").exists()
                || path.join("turtle-wow.exe").exists();

            let has_data = path.join("Data").exists() && path.join("Data").is_dir();

            has_exe && has_data
        }

        /// Try to auto-detect WoW folder from current directory or exe location
        pub fn auto_detect() -> Option<PathBuf> {
            // Check current working directory
            if let Ok(cwd) = std::env::current_dir() {
                if Self::is_valid_wow_folder(&cwd) {
                    return Some(cwd);
                }
            }

            // Check executable directory
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(exe_dir) = exe_path.parent() {
                    if Self::is_valid_wow_folder(exe_dir) {
                        return Some(exe_dir.to_path_buf());
                    }
                }
            }

            None
        }

        /// Get the DATA folder path from a WoW installation
        pub fn get_data_folder(wow_path: &Path) -> PathBuf {
            wow_path.join("Data")
        }
    }
    ```

    Add unit tests:
    - Test is_valid_wow_folder returns false for non-existent path
    - Test get_data_folder returns correct path
    - Test auto_detect returns None when not in WoW folder (expected in dev environment)
  </action>
  <verify>
    `cargo test -p turtle-wow-hd-patcher detector` passes all tests
  </verify>
  <done>
    WowDetector validates WoW folders by checking for exe + Data directory
  </done>
</task>

<task type="auto">
  <name>Task 3: Implement settings persistence with Tauri store</name>
  <files>src-tauri/src/install/settings.rs</files>
  <action>
    Create settings.rs with Settings struct:

    ```rust
    //! Settings persistence using Tauri store plugin

    use tauri::AppHandle;
    use tauri_plugin_store::StoreExt;
    use serde_json::json;
    use thiserror::Error;

    const SETTINGS_FILE: &str = "settings.json";
    const KEY_WOW_PATH: &str = "wow_path";
    const KEY_SELECTED_MODULES: &str = "selected_modules";

    #[derive(Debug, Error)]
    pub enum SettingsError {
        #[error("Store error: {0}")]
        StoreError(String),

        #[error("Failed to save settings: {0}")]
        SaveError(String),
    }

    /// Settings manager using Tauri store plugin
    pub struct Settings<'a> {
        app: &'a AppHandle,
    }

    impl<'a> Settings<'a> {
        pub fn new(app: &'a AppHandle) -> Self {
            Self { app }
        }

        /// Get the saved WoW installation path
        pub fn get_wow_path(&self) -> Option<String> {
            let store = self.app.store(SETTINGS_FILE).ok()?;
            store.get(KEY_WOW_PATH)
                .and_then(|v| v.as_str().map(String::from))
        }

        /// Save the WoW installation path
        pub fn set_wow_path(&self, path: &str) -> Result<(), SettingsError> {
            let store = self.app.store(SETTINGS_FILE)
                .map_err(|e| SettingsError::StoreError(e.to_string()))?;
            store.set(KEY_WOW_PATH, json!(path));
            store.save()
                .map_err(|e| SettingsError::SaveError(e.to_string()))?;
            Ok(())
        }

        /// Get the list of selected module IDs
        pub fn get_selected_modules(&self) -> Vec<String> {
            let store = match self.app.store(SETTINGS_FILE) {
                Ok(s) => s,
                Err(_) => return Vec::new(),
            };

            store.get(KEY_SELECTED_MODULES)
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect())
                .unwrap_or_default()
        }

        /// Save the list of selected module IDs
        pub fn set_selected_modules(&self, modules: &[String]) -> Result<(), SettingsError> {
            let store = self.app.store(SETTINGS_FILE)
                .map_err(|e| SettingsError::StoreError(e.to_string()))?;
            store.set(KEY_SELECTED_MODULES, json!(modules));
            store.save()
                .map_err(|e| SettingsError::SaveError(e.to_string()))?;
            Ok(())
        }
    }
    ```

    Update install/mod.rs to export SettingsError:
    ```rust
    pub use settings::{Settings, SettingsError};
    ```
  </action>
  <verify>
    `cargo check -p turtle-wow-hd-patcher` compiles without errors (full test requires Tauri runtime)
  </verify>
  <done>
    Settings struct persists WoW path and selected modules via Tauri store plugin
  </done>
</task>

</tasks>

<verification>
After all tasks complete:
1. `cargo check -p turtle-wow-hd-patcher` compiles successfully
2. `cargo test -p turtle-wow-hd-patcher` all tests pass (including new detector tests)
3. New files exist: src-tauri/src/install/{mod.rs, detector.rs, settings.rs}
4. Cargo.toml has tauri-plugin-dialog and tauri-plugin-store dependencies
5. capabilities/default.json has dialog:default and store:default permissions
</verification>

<success_criteria>
- Tauri dialog and store plugins added and registered
- WowDetector validates folders by checking for WoW.exe variants + Data directory
- Settings struct can get/set WoW path and selected modules
- All existing tests still pass
- Project compiles cleanly
</success_criteria>

<output>
After completion, create `.planning/phases/3-installation-engine/3-01-SUMMARY.md`
</output>
