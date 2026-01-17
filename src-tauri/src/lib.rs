mod parser;
mod models;
mod download;
mod install;

use std::collections::HashSet;
use std::path::PathBuf;
use tauri::{ipc::Channel, Manager, State};
use tauri_plugin_dialog::DialogExt;

use models::{PatchModule, PatchId, DownloadProvider as ProviderType};
use parser::{
    forum::{fetch_forum_post_with_fallback, ForumParser, FORUM_URL},
    links::extract_download_links,
    modules::parse_modules,
    dependencies::{validate_module_selection, auto_select_dependencies},
};
use download::{DownloadManager, progress::DownloadEvent};
use install::{
    InstallManager, InstallEvent, InstallError,
    VerifyResult, RepairResult, WowDetector, Settings,
};

#[tauri::command]
async fn fetch_patches() -> Result<Vec<PatchModule>, String> {
    // Fetch forum HTML
    let html = fetch_forum_post_with_fallback()
        .await
        .map_err(|e| e.to_string())?;

    // Parse the HTML
    let parser = ForumParser::new().map_err(|e| e.to_string())?;
    let parsed = parser.parse(&html).map_err(|e| e.to_string())?;

    // Extract download links
    let links = extract_download_links(&parsed.content);

    // Also extract from raw link URLs
    let mut all_links = links;
    for url in &parsed.links {
        all_links.extend(extract_download_links(url));
    }

    // Build module list with links
    let modules = parse_modules(&parsed.content, &all_links).map_err(|e| e.to_string())?;

    Ok(modules)
}

#[tauri::command]
fn validate_selection(selected: Vec<String>) -> Result<(), Vec<String>> {
    let patch_ids: HashSet<PatchId> = selected
        .iter()
        .filter_map(|s| match s.as_str() {
            "A" => Some(PatchId::A),
            "B" => Some(PatchId::B),
            "C" => Some(PatchId::C),
            "D" => Some(PatchId::D),
            "E" => Some(PatchId::E),
            "G" => Some(PatchId::G),
            "I" => Some(PatchId::I),
            "L" => Some(PatchId::L),
            "M" => Some(PatchId::M),
            "N" => Some(PatchId::N),
            "O" => Some(PatchId::O),
            "S" => Some(PatchId::S),
            "U" => Some(PatchId::U),
            "V" => Some(PatchId::V),
            _ => None,
        })
        .collect();

    validate_module_selection(&patch_ids)
}

#[tauri::command]
fn auto_select_deps(selected: Vec<String>) -> Vec<String> {
    let patch_ids: HashSet<PatchId> = selected
        .iter()
        .filter_map(|s| match s.as_str() {
            "A" => Some(PatchId::A),
            "B" => Some(PatchId::B),
            "C" => Some(PatchId::C),
            "D" => Some(PatchId::D),
            "E" => Some(PatchId::E),
            "G" => Some(PatchId::G),
            "I" => Some(PatchId::I),
            "L" => Some(PatchId::L),
            "M" => Some(PatchId::M),
            "N" => Some(PatchId::N),
            "O" => Some(PatchId::O),
            "S" => Some(PatchId::S),
            "U" => Some(PatchId::U),
            "V" => Some(PatchId::V),
            _ => None,
        })
        .collect();

    let with_deps = auto_select_dependencies(&patch_ids);

    with_deps
        .iter()
        .map(|id| format!("{:?}", id))
        .collect()
}

#[tauri::command]
fn get_forum_url() -> String {
    FORUM_URL.to_string()
}

/// Start a download for a patch module
///
/// Spawns an async download task that reports progress via the Channel.
/// Returns the download_id immediately for tracking.
#[tauri::command]
async fn start_download(
    manager: State<'_, DownloadManager>,
    share_url: String,
    provider: String,
    dest_dir: String,
    on_progress: Channel<DownloadEvent>,
    target_filename: Option<String>,
) -> Result<String, String> {
    let download_id = uuid::Uuid::new_v4().to_string();

    let provider_type = match provider.to_lowercase().as_str() {
        "googledrive" | "google_drive" | "gdrive" => ProviderType::GoogleDrive,
        "mediafire" => ProviderType::Mediafire,
        _ => ProviderType::Unknown,
    };

    let dest_path = PathBuf::from(dest_dir);

    // Clone values for spawned task
    let manager_clone = manager.inner().clone();
    let download_id_clone = download_id.clone();

    tauri::async_runtime::spawn(async move {
        let result = manager_clone
            .download(
                share_url,
                provider_type,
                dest_path,
                download_id_clone.clone(),
                on_progress.clone(),
                target_filename,
            )
            .await;

        if let Err(e) = result {
            let _ = on_progress.send(DownloadEvent::Failed {
                download_id: download_id_clone,
                error: e.to_string(),
            });
        }
    });

    Ok(download_id)
}

/// Get current active download count
#[tauri::command]
fn get_active_downloads(manager: State<'_, DownloadManager>) -> usize {
    manager.active_downloads()
}

// ============================================================================
// Installation Commands
// ============================================================================

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
        Some(file_path) => {
            // Convert FilePath to PathBuf
            let path = file_path.into_path()
                .map_err(|e| format!("Invalid path: {}", e))?;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
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
