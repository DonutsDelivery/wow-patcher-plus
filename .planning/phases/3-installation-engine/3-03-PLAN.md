---
phase: 3-installation-engine
plan: 03
type: execute
wave: 3
depends_on: ["3-01", "3-02"]
files_modified:
  - src-tauri/src/install/mod.rs
  - src-tauri/src/install/repair.rs
  - src-tauri/src/install/manager.rs
  - src-tauri/src/lib.rs
autonomous: true
user_setup: []

must_haves:
  truths:
    - "User can select WoW folder via native dialog"
    - "User can repair patches by re-copying from downloads"
    - "User can install multiple patches with progress events"
    - "User can verify all installed patches via command"
  artifacts:
    - path: "src-tauri/src/install/repair.rs"
      provides: "Repair logic for re-copying patches"
      exports: ["repair_patch", "repair_all"]
    - path: "src-tauri/src/install/manager.rs"
      provides: "InstallManager coordinating all install operations"
      exports: ["InstallManager"]
    - path: "src-tauri/src/lib.rs"
      provides: "Tauri commands for installation"
      contains: ["select_wow_folder", "install_patches", "verify_patches", "repair_patches"]
  key_links:
    - from: "src-tauri/src/lib.rs"
      to: "src-tauri/src/install/manager.rs"
      via: "State<InstallManager>"
      pattern: "State.*InstallManager"
    - from: "src-tauri/src/install/manager.rs"
      to: "src-tauri/src/install/copier.rs"
      via: "uses install_mpq"
      pattern: "install_mpq"
    - from: "src-tauri/src/install/manager.rs"
      to: "src-tauri/src/install/verifier.rs"
      via: "uses verify_patch"
      pattern: "verify_patch"
---

<objective>
Implement repair functionality and Tauri commands for the installation engine.

Purpose: Complete the installation engine with repair capability and expose all functionality to the frontend via Tauri commands.

Output:
- repair.rs with repair_patch and repair_all functions
- manager.rs coordinating install, verify, and repair
- Tauri commands in lib.rs for frontend integration
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
@.planning/phases/3-installation-engine/3-01-SUMMARY.md
@.planning/phases/3-installation-engine/3-02-SUMMARY.md

# Existing install module
@src-tauri/src/install/mod.rs
@src-tauri/src/install/detector.rs
@src-tauri/src/install/settings.rs
@src-tauri/src/install/copier.rs
@src-tauri/src/install/verifier.rs

# Pattern for Tauri commands
@src-tauri/src/lib.rs
</context>

<tasks>

<task type="auto">
  <name>Task 1: Implement repair functionality</name>
  <files>
    - src-tauri/src/install/repair.rs
    - src-tauri/src/install/mod.rs
  </files>
  <action>
    Create repair.rs with repair logic:

    ```rust
    //! Repair functionality for patch installations
    //!
    //! Repair works by re-copying from the downloads folder.
    //! If the download is missing, the repair fails (requires re-download first).

    use std::path::Path;
    use tauri::ipc::Channel;

    use super::copier::{install_mpq, get_mpq_filename, InstallEvent, InstallError};
    use super::verifier::VerifyResult;

    /// Result of a repair operation
    #[derive(Debug, Clone, serde::Serialize)]
    #[serde(rename_all = "camelCase", tag = "status")]
    pub enum RepairResult {
        /// Repair succeeded
        Repaired { patch_id: String },

        /// Download missing, need to re-download first
        DownloadMissing { patch_id: String },

        /// Repair failed with error
        Failed { patch_id: String, error: String },
    }

    /// Repair a single patch by re-copying from downloads
    ///
    /// Returns RepairResult indicating success or what went wrong.
    pub async fn repair_patch(
        patch_id: &str,
        data_folder: &Path,
        downloads_folder: &Path,
        on_event: Channel<InstallEvent>,
    ) -> RepairResult {
        let mpq_name = get_mpq_filename(patch_id);
        let download_path = downloads_folder.join(&mpq_name);

        // Check if download exists
        if !download_path.exists() {
            return RepairResult::DownloadMissing {
                patch_id: patch_id.to_string(),
            };
        }

        // Re-copy (install_mpq overwrites existing)
        match install_mpq(&download_path, data_folder, patch_id.to_string(), on_event).await {
            Ok(_) => RepairResult::Repaired {
                patch_id: patch_id.to_string(),
            },
            Err(e) => RepairResult::Failed {
                patch_id: patch_id.to_string(),
                error: e.to_string(),
            },
        }
    }

    /// Repair multiple patches
    ///
    /// Attempts to repair each patch in sequence.
    pub async fn repair_all(
        patch_ids: &[&str],
        data_folder: &Path,
        downloads_folder: &Path,
        on_event: Channel<InstallEvent>,
    ) -> Vec<RepairResult> {
        let mut results = Vec::with_capacity(patch_ids.len());

        for id in patch_ids {
            let result = repair_patch(id, data_folder, downloads_folder, on_event.clone()).await;
            results.push(result);
        }

        results
    }

    /// Determine which patches need repair based on verification results
    pub fn patches_needing_repair(verify_results: &[(String, VerifyResult)]) -> Vec<String> {
        verify_results
            .iter()
            .filter_map(|(id, result)| match result {
                VerifyResult::NotInstalled => Some(id.clone()),
                VerifyResult::SizeMismatch { .. } => Some(id.clone()),
                VerifyResult::Error { .. } => Some(id.clone()),
                VerifyResult::Installed { verified: false } => None, // Can't verify, assume OK
                VerifyResult::Installed { verified: true } => None,
            })
            .collect()
    }
    ```

    Update install/mod.rs to add repair module and exports:
    ```rust
    pub mod repair;
    pub use repair::{repair_patch, repair_all, RepairResult, patches_needing_repair};
    ```
  </action>
  <verify>
    `cargo check -p turtle-wow-hd-patcher` compiles without errors
  </verify>
  <done>
    repair_patch re-copies from downloads, repair_all handles multiple patches
  </done>
</task>

<task type="auto">
  <name>Task 2: Create InstallManager to coordinate operations</name>
  <files>
    - src-tauri/src/install/manager.rs
    - src-tauri/src/install/mod.rs
  </files>
  <action>
    Create manager.rs with InstallManager:

    ```rust
    //! Installation manager coordinating all install operations
    //!
    //! Provides a centralized manager for:
    //! - WoW folder path management
    //! - Downloads folder path management
    //! - Install, verify, and repair operations

    use std::path::{Path, PathBuf};
    use std::sync::RwLock;
    use tauri::ipc::Channel;

    use super::detector::WowDetector;
    use super::copier::{install_mpq, get_mpq_filename, InstallEvent, InstallError};
    use super::verifier::{verify_patch, verify_all, VerifyResult};
    use super::repair::{repair_patch, repair_all, RepairResult};

    /// Centralized installation manager
    ///
    /// Manages paths and coordinates install/verify/repair operations.
    /// Thread-safe via RwLock for path storage.
    pub struct InstallManager {
        wow_path: RwLock<Option<PathBuf>>,
        downloads_path: PathBuf,
    }

    impl InstallManager {
        /// Create a new InstallManager with the downloads directory
        pub fn new(downloads_path: PathBuf) -> Self {
            Self {
                wow_path: RwLock::new(None),
                downloads_path,
            }
        }

        /// Set the WoW installation path
        ///
        /// Validates the path is a valid WoW installation before setting.
        pub fn set_wow_path(&self, path: PathBuf) -> Result<(), InstallError> {
            if !WowDetector::is_valid_wow_folder(&path) {
                return Err(InstallError::InvalidWowFolder);
            }

            let mut wow_path = self.wow_path.write().unwrap();
            *wow_path = Some(path);
            Ok(())
        }

        /// Get the current WoW path (if set)
        pub fn get_wow_path(&self) -> Option<PathBuf> {
            self.wow_path.read().unwrap().clone()
        }

        /// Get the Data folder path
        pub fn get_data_folder(&self) -> Result<PathBuf, InstallError> {
            let wow_path = self.wow_path.read().unwrap();
            let path = wow_path.as_ref().ok_or(InstallError::WowPathNotSet)?;
            Ok(WowDetector::get_data_folder(path))
        }

        /// Get the downloads folder path
        pub fn get_downloads_folder(&self) -> &Path {
            &self.downloads_path
        }

        /// Install a single patch
        pub async fn install_patch(
            &self,
            patch_id: &str,
            on_event: Channel<InstallEvent>,
        ) -> Result<PathBuf, InstallError> {
            let data_folder = self.get_data_folder()?;
            let mpq_name = get_mpq_filename(patch_id);
            let source_path = self.downloads_path.join(&mpq_name);

            if !source_path.exists() {
                return Err(InstallError::DownloadNotFound(mpq_name));
            }

            install_mpq(&source_path, &data_folder, patch_id.to_string(), on_event).await
        }

        /// Install multiple patches
        pub async fn install_patches(
            &self,
            patch_ids: &[&str],
            on_event: Channel<InstallEvent>,
        ) -> Vec<Result<PathBuf, InstallError>> {
            let mut results = Vec::with_capacity(patch_ids.len());

            for id in patch_ids {
                let result = self.install_patch(id, on_event.clone()).await;
                results.push(result);
            }

            results
        }

        /// Verify a single patch
        pub async fn verify_patch(&self, patch_id: &str) -> Result<VerifyResult, InstallError> {
            let data_folder = self.get_data_folder()?;
            Ok(verify_patch(patch_id, &data_folder, &self.downloads_path).await)
        }

        /// Verify multiple patches
        pub async fn verify_patches(&self, patch_ids: &[&str]) -> Result<Vec<(String, VerifyResult)>, InstallError> {
            let data_folder = self.get_data_folder()?;
            Ok(verify_all(patch_ids, &data_folder, &self.downloads_path).await)
        }

        /// Repair a single patch
        pub async fn repair_patch(
            &self,
            patch_id: &str,
            on_event: Channel<InstallEvent>,
        ) -> Result<RepairResult, InstallError> {
            let data_folder = self.get_data_folder()?;
            Ok(repair_patch(patch_id, &data_folder, &self.downloads_path, on_event).await)
        }

        /// Repair multiple patches
        pub async fn repair_patches(
            &self,
            patch_ids: &[&str],
            on_event: Channel<InstallEvent>,
        ) -> Result<Vec<RepairResult>, InstallError> {
            let data_folder = self.get_data_folder()?;
            Ok(repair_all(patch_ids, &data_folder, &self.downloads_path, on_event).await)
        }

        /// Try to auto-detect WoW folder and set it
        pub fn try_auto_detect(&self) -> bool {
            if let Some(path) = WowDetector::auto_detect() {
                self.set_wow_path(path).is_ok()
            } else {
                false
            }
        }
    }
    ```

    Update install/mod.rs to add manager module and exports:
    ```rust
    pub mod manager;
    pub use manager::InstallManager;
    ```

    The final install/mod.rs should look like:
    ```rust
    //! Installation engine for HD patch files
    //!
    //! This module provides:
    //! - WoW folder detection and validation
    //! - Settings persistence for WoW path
    //! - MPQ file copy operations with progress
    //! - Installation verification
    //! - Repair functionality

    pub mod detector;
    pub mod settings;
    pub mod copier;
    pub mod verifier;
    pub mod repair;
    pub mod manager;

    pub use detector::WowDetector;
    pub use settings::{Settings, SettingsError};
    pub use copier::{install_mpq, InstallEvent, InstallError, get_mpq_filename};
    pub use verifier::{verify_patch, verify_all, VerifyResult, is_patch_installed, get_installed_patches};
    pub use repair::{repair_patch, repair_all, RepairResult, patches_needing_repair};
    pub use manager::InstallManager;
    ```
  </action>
  <verify>
    `cargo check -p turtle-wow-hd-patcher` compiles without errors
  </verify>
  <done>
    InstallManager coordinates install, verify, and repair with path management
  </done>
</task>

<task type="auto">
  <name>Task 3: Add Tauri commands for installation</name>
  <files>src-tauri/src/lib.rs</files>
  <action>
    Update lib.rs to add installation commands and register InstallManager:

    Add imports at top:
    ```rust
    use std::sync::Arc;
    use tauri_plugin_dialog::DialogExt;
    use install::{
        InstallManager, InstallEvent, InstallError as InstallErr,
        VerifyResult, RepairResult, WowDetector, Settings,
    };
    ```

    Add new Tauri commands:

    ```rust
    /// Select WoW folder via native dialog
    ///
    /// Opens folder picker and validates selection is a valid WoW installation.
    #[tauri::command]
    async fn select_wow_folder(
        app: tauri::AppHandle,
        manager: State<'_, InstallManager>,
    ) -> Result<Option<String>, String> {
        let folder = app.dialog()
            .file()
            .set_title("Select Turtle WoW Installation Folder")
            .blocking_pick_folder();

        match folder {
            Some(f) => {
                let path = f.path;
                if WowDetector::is_valid_wow_folder(&path) {
                    // Update manager
                    manager.set_wow_path(path.clone())
                        .map_err(|e| e.to_string())?;

                    // Save to settings
                    let settings = Settings::new(&app);
                    let _ = settings.set_wow_path(&path.to_string_lossy());

                    Ok(Some(path.to_string_lossy().to_string()))
                } else {
                    Err("Selected folder is not a valid Turtle WoW installation. Must contain WoW.exe and Data folder.".to_string())
                }
            }
            None => Ok(None), // User cancelled
        }
    }

    /// Get the current WoW folder path
    #[tauri::command]
    fn get_wow_path(manager: State<'_, InstallManager>) -> Option<String> {
        manager.get_wow_path().map(|p| p.to_string_lossy().to_string())
    }

    /// Try to auto-detect WoW folder
    #[tauri::command]
    fn auto_detect_wow(
        app: tauri::AppHandle,
        manager: State<'_, InstallManager>,
    ) -> Option<String> {
        if manager.try_auto_detect() {
            let path = manager.get_wow_path()?;

            // Save to settings
            let settings = Settings::new(&app);
            let _ = settings.set_wow_path(&path.to_string_lossy());

            Some(path.to_string_lossy().to_string())
        } else {
            None
        }
    }

    /// Install patches to WoW Data folder
    #[tauri::command]
    async fn install_patches(
        manager: State<'_, InstallManager>,
        patch_ids: Vec<String>,
        on_event: Channel<InstallEvent>,
    ) -> Result<Vec<String>, String> {
        let ids: Vec<&str> = patch_ids.iter().map(|s| s.as_str()).collect();
        let results = manager.install_patches(&ids, on_event).await;

        let mut installed = Vec::new();
        for (id, result) in patch_ids.iter().zip(results.iter()) {
            if result.is_ok() {
                installed.push(id.clone());
            }
        }

        Ok(installed)
    }

    /// Verify installed patches
    #[tauri::command]
    async fn verify_patches(
        manager: State<'_, InstallManager>,
        patch_ids: Vec<String>,
    ) -> Result<Vec<(String, VerifyResult)>, String> {
        let ids: Vec<&str> = patch_ids.iter().map(|s| s.as_str()).collect();
        manager.verify_patches(&ids).await.map_err(|e| e.to_string())
    }

    /// Repair patches by re-copying from downloads
    #[tauri::command]
    async fn repair_patches(
        manager: State<'_, InstallManager>,
        patch_ids: Vec<String>,
        on_event: Channel<InstallEvent>,
    ) -> Result<Vec<RepairResult>, String> {
        let ids: Vec<&str> = patch_ids.iter().map(|s| s.as_str()).collect();
        manager.repair_patches(&ids, on_event).await.map_err(|e| e.to_string())
    }

    /// Load saved settings on startup
    #[tauri::command]
    fn load_saved_wow_path(
        app: tauri::AppHandle,
        manager: State<'_, InstallManager>,
    ) -> Option<String> {
        let settings = Settings::new(&app);
        if let Some(path_str) = settings.get_wow_path() {
            let path = PathBuf::from(&path_str);
            if WowDetector::is_valid_wow_folder(&path) {
                let _ = manager.set_wow_path(path);
                return Some(path_str);
            }
        }
        None
    }
    ```

    Update the run() function to:
    1. Create downloads directory path (use app data directory)
    2. Register InstallManager with Tauri state
    3. Register new plugins (dialog, store)
    4. Add new commands to invoke_handler

    ```rust
    #[cfg_attr(mobile, tauri::mobile_entry_point)]
    pub fn run() {
        tauri::Builder::default()
            .plugin(tauri_plugin_opener::init())
            .plugin(tauri_plugin_http::init())
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_store::init())
            .setup(|app| {
                // Get app data directory for downloads
                let app_data = app.path().app_data_dir()
                    .expect("Failed to get app data directory");
                let downloads_path = app_data.join("downloads");

                // Create downloads directory if it doesn't exist
                std::fs::create_dir_all(&downloads_path)
                    .expect("Failed to create downloads directory");

                // Create and register InstallManager
                let install_manager = InstallManager::new(downloads_path);
                app.manage(install_manager);

                Ok(())
            })
            .manage(DownloadManager::new())
            .invoke_handler(tauri::generate_handler![
                // Parser commands
                fetch_patches,
                validate_selection,
                auto_select_deps,
                get_forum_url,
                // Download commands
                start_download,
                get_active_downloads,
                // Install commands
                select_wow_folder,
                get_wow_path,
                auto_detect_wow,
                install_patches,
                verify_patches,
                repair_patches,
                load_saved_wow_path,
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }
    ```

    Add necessary imports at top of lib.rs:
    ```rust
    use tauri::Manager; // for app.path() in setup
    ```
  </action>
  <verify>
    `cargo check -p turtle-wow-hd-patcher` compiles without errors
    `cargo test -p turtle-wow-hd-patcher` all tests pass
  </verify>
  <done>
    Tauri commands available: select_wow_folder, get_wow_path, auto_detect_wow, install_patches, verify_patches, repair_patches, load_saved_wow_path
  </done>
</task>

</tasks>

<verification>
After all tasks complete:
1. `cargo check -p turtle-wow-hd-patcher` compiles successfully
2. `cargo test -p turtle-wow-hd-patcher` all tests pass
3. New files exist: src-tauri/src/install/{repair.rs, manager.rs}
4. lib.rs has all new Tauri commands registered
5. InstallManager is created in setup() with downloads path
6. Dialog and store plugins registered
</verification>

<success_criteria>
- repair_patch re-copies from downloads folder
- InstallManager coordinates all install operations
- Tauri commands expose full installation API:
  - select_wow_folder: Native folder picker with validation
  - get_wow_path: Get current WoW path
  - auto_detect_wow: Try to auto-detect WoW folder
  - install_patches: Install selected patches with progress
  - verify_patches: Verify installed patches
  - repair_patches: Repair patches from downloads
  - load_saved_wow_path: Load path from settings on startup
- All existing tests pass
- Project compiles cleanly
</success_criteria>

<output>
After completion, create `.planning/phases/3-installation-engine/3-03-SUMMARY.md`
</output>
