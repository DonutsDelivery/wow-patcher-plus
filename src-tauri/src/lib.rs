mod parser;
mod models;
mod download;
mod install;

use std::collections::HashSet;
use std::fs::File;
use std::path::PathBuf;
use std::sync::RwLock;
use tauri::{ipc::Channel, Manager, State};
use tauri_plugin_dialog::DialogExt;
use log::LevelFilter;
use simplelog::{CombinedLogger, Config, WriteLogger};

use models::{PatchModule, PatchId, PatchGroup, DownloadLink, DownloadProvider as ProviderType};
use parser::dependencies::{validate_module_selection, auto_select_dependencies};
use download::{DownloadManager, progress::DownloadEvent};
use install::{
    InstallManager, InstallEvent,
    VerifyResult, RepairResult, WowDetector, Settings,
};

/// GitHub raw URL for patches.json
const PATCHES_JSON_URL: &str = "https://raw.githubusercontent.com/DonutsDelivery/wow-patcher-plus/main/patches.json";

/// Cached patches data for validation
pub struct PatchesCache {
    modules: RwLock<Vec<PatchModule>>,
    groups: RwLock<Vec<PatchGroup>>,
}

impl PatchesCache {
    pub fn new() -> Self {
        Self {
            modules: RwLock::new(Vec::new()),
            groups: RwLock::new(Vec::new()),
        }
    }

    pub fn update(&self, modules: Vec<PatchModule>, groups: Vec<PatchGroup>) {
        *self.modules.write().unwrap() = modules;
        *self.groups.write().unwrap() = groups;
    }

    pub fn get_modules(&self) -> Vec<PatchModule> {
        self.modules.read().unwrap().clone()
    }

    pub fn get_groups(&self) -> Vec<PatchGroup> {
        self.groups.read().unwrap().clone()
    }

    pub fn get_linked_groups(&self) -> Vec<Vec<PatchId>> {
        self.groups
            .read()
            .unwrap()
            .iter()
            .filter(|g| g.linked)
            .map(|g| g.ids.clone())
            .collect()
    }
}

/// Response type that includes both patches and groups
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchesResponse {
    pub patches: Vec<PatchModule>,
    pub groups: Vec<PatchGroup>,
}

#[tauri::command]
async fn fetch_patches(cache: State<'_, PatchesCache>) -> Result<PatchesResponse, String> {
    let client = reqwest::Client::new();

    // Try to fetch from GitHub
    let response = client
        .get(PATCHES_JSON_URL)
        .header("User-Agent", "WoW-HD-Patcher")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch patches: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Failed to fetch patches: HTTP {}", response.status()));
    }

    let json: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse patches JSON: {}", e))?;

    // Parse the JSON into PatchModule structs
    let patches = json["patches"].as_object()
        .ok_or("Invalid patches.json format")?;

    let mut modules = Vec::new();
    for (id, patch) in patches {
        let links: Vec<DownloadLink> = patch["links"].as_array()
            .map(|arr| arr.iter().filter_map(|link| {
                Some(DownloadLink {
                    provider: match link["provider"].as_str()? {
                        "mediafire" => ProviderType::Mediafire,
                        "googledrive" | "gdrive" => ProviderType::GoogleDrive,
                        "dropbox" => ProviderType::Dropbox,
                        "transfer" => ProviderType::Transfer,
                        "mega" => ProviderType::Mega,
                        _ => ProviderType::Unknown,
                    },
                    url: link["url"].as_str()?.to_string(),
                    file_name: link["file_name"].as_str().map(|s| s.to_string()),
                    variant: link["variant"].as_str().map(|s| s.to_string()),
                })
            }).collect())
            .unwrap_or_default();

        let dependencies: Vec<PatchId> = patch["dependencies"].as_array()
            .map(|arr| arr.iter().filter_map(|d| d.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        let conflicts: Vec<PatchId> = patch["conflicts"].as_array()
            .map(|arr| arr.iter().filter_map(|c| c.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        let variants: Option<Vec<String>> = patch["variants"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());

        let preview: Option<String> = patch["preview"].as_str().map(|s| s.to_string());
        let author: Option<String> = patch["author"].as_str().map(|s| s.to_string());
        let forum_url: Option<String> = patch["forumUrl"].as_str().map(|s| s.to_string());

        modules.push(PatchModule {
            id: id.clone(),
            name: patch["name"].as_str().unwrap_or("Unknown").to_string(),
            description: patch["description"].as_str().unwrap_or("").to_string(),
            downloads: links,
            dependencies,
            conflicts,
            file_size: None,
            last_updated: None,
            variants,
            preview,
            author,
            forum_url,
        });
    }

    // Parse groups from JSON
    let groups: Vec<PatchGroup> = json["groups"].as_array()
        .map(|arr| arr.iter().filter_map(|g| {
            Some(PatchGroup {
                name: g["name"].as_str()?.to_string(),
                description: g["description"].as_str().unwrap_or("").to_string(),
                ids: g["ids"].as_array()?
                    .iter()
                    .filter_map(|id| id.as_str().map(|s| s.to_string()))
                    .collect(),
                linked: g["linked"].as_bool().unwrap_or(false),
            })
        }).collect())
        .unwrap_or_default();

    // Update cache for validation
    cache.update(modules.clone(), groups.clone());

    Ok(PatchesResponse { patches: modules, groups })
}

#[tauri::command]
fn validate_selection(
    cache: State<'_, PatchesCache>,
    selected: Vec<String>,
) -> Result<(), Vec<String>> {
    let patch_ids: HashSet<PatchId> = selected.into_iter().collect();
    let modules = cache.get_modules();
    let linked_groups = cache.get_linked_groups();

    validate_module_selection(&patch_ids, &modules, &linked_groups)
}

#[tauri::command]
fn auto_select_deps(
    cache: State<'_, PatchesCache>,
    selected: Vec<String>,
) -> Vec<String> {
    let patch_ids: HashSet<PatchId> = selected.into_iter().collect();
    let modules = cache.get_modules();
    let linked_groups = cache.get_linked_groups();

    let with_deps = auto_select_dependencies(&patch_ids, &modules, &linked_groups);

    with_deps.into_iter().collect()
}

/// Get conflicts for selected patches
#[tauri::command]
fn get_conflicts(
    cache: State<'_, PatchesCache>,
    selected: Vec<String>,
) -> Vec<String> {
    let patch_ids: HashSet<PatchId> = selected.into_iter().collect();
    let modules = cache.get_modules();

    parser::dependencies::get_conflicts(&patch_ids, &modules)
        .into_iter()
        .collect()
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
        "dropbox" => ProviderType::Dropbox,
        "transfer" => ProviderType::Transfer,
        "mega" => ProviderType::Mega,
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
        .set_title("Select WoW Installation Folder")
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
                Err("Selected folder is not a valid WoW installation. Must contain WoW.exe and Data folder.".to_string())
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
    log::info!("[Install] install_patches called for: {:?}", patch_ids);
    log::info!("[Install] WoW path: {:?}", manager.get_wow_path());
    log::info!("[Install] Downloads path: {:?}", manager.get_downloads_folder());

    let ids: Vec<&str> = patch_ids.iter().map(|s| s.as_str()).collect();
    let results = manager.install_patches(&ids, on_event).await;

    let mut installed = Vec::new();
    let mut errors = Vec::new();
    for (id, result) in patch_ids.iter().zip(results.iter()) {
        match result {
            Ok(_) => installed.push(id.clone()),
            Err(e) => {
                log::error!("[Install] Failed to install {}: {:?}", id, e);
                errors.push(format!("{}: {:?}", id, e));
            }
        }
    }

    if !errors.is_empty() {
        log::error!("[Install] Installation errors: {:?}", errors);
    }

    log::info!("[Install] Successfully installed: {:?}", installed);
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

/// Detect which patches are already installed in the WoW Data folder
#[tauri::command]
async fn detect_installed_patches(
    manager: State<'_, InstallManager>,
    patch_ids: Vec<String>,
) -> Result<Vec<String>, String> {
    let wow_path = manager.get_wow_path()
        .ok_or("WoW path not set")?;

    let data_folder = wow_path.join("Data");
    if !data_folder.exists() {
        return Ok(Vec::new());
    }

    let ids: Vec<&str> = patch_ids.iter().map(|s| s.as_str()).collect();
    let installed_paths = install::verifier::get_installed_patches(&ids, &data_folder).await;

    // Extract just the patch IDs from the installed paths
    let installed_ids: Vec<String> = installed_paths
        .iter()
        .filter_map(|path| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .and_then(|name| name.strip_prefix("Patch-"))
                .map(|id| id.to_string())
        })
        .collect();

    Ok(installed_ids)
}

/// Uninstall patches by removing MPQ files from the WoW Data folder
#[tauri::command]
async fn uninstall_patches(
    manager: State<'_, InstallManager>,
    patch_ids: Vec<String>,
) -> Result<Vec<String>, String> {
    let wow_path = manager.get_wow_path()
        .ok_or("WoW path not set")?;

    let data_folder = wow_path.join("Data");
    if !data_folder.exists() {
        return Err("Data folder not found".to_string());
    }

    let mut uninstalled = Vec::new();
    for patch_id in &patch_ids {
        match install::copier::uninstall_mpq(&data_folder, patch_id).await {
            Ok(_) => uninstalled.push(patch_id.clone()),
            Err(e) => log::error!("[Uninstall] Failed to uninstall {}: {:?}", patch_id, e),
        }
    }

    // Clear WDB cache after uninstalling
    if let Err(e) = manager.clear_wdb_folder().await {
        log::warn!("[Uninstall] Failed to clear WDB: {:?}", e);
    }

    Ok(uninstalled)
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

/// Check requirements status (VanillaHelpers and DXVK)
#[derive(serde::Serialize)]
pub struct RequirementsStatus {
    vanilla_helpers: bool,
    dxvk: bool,
}

#[tauri::command]
fn check_requirements(manager: State<'_, InstallManager>) -> Option<RequirementsStatus> {
    let wow_path = manager.get_wow_path()?;

    // Check for VanillaHelpers.dll (case-insensitive)
    let vanilla_helpers = wow_path.join("VanillaHelpers.dll").exists()
        || wow_path.join("vanillahelpers.dll").exists();

    // Check for d3d9.dll (DXVK)
    let dxvk = wow_path.join("d3d9.dll").exists();

    Some(RequirementsStatus {
        vanilla_helpers,
        dxvk,
    })
}

/// Install VanillaHelpers from GitHub
#[tauri::command]
async fn install_vanilla_helpers(manager: State<'_, InstallManager>) -> Result<(), String> {
    let wow_path = manager.get_wow_path()
        .ok_or("WoW path not set")?;

    // GitHub releases API to get latest release
    let client = reqwest::Client::new();
    let releases_url = "https://api.github.com/repos/isfir/VanillaHelpers/releases/latest";

    let release: serde_json::Value = client
        .get(releases_url)
        .header("User-Agent", "WoW-HD-Patcher")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch releases: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse releases: {}", e))?;

    // Find the DLL asset
    let assets = release["assets"].as_array()
        .ok_or("No assets found in release")?;

    let dll_asset = assets.iter()
        .find(|a| a["name"].as_str().map(|n: &str| n.to_lowercase().ends_with(".dll")).unwrap_or(false))
        .ok_or("VanillaHelpers.dll not found in release")?;

    let download_url = dll_asset["browser_download_url"].as_str()
        .ok_or("No download URL for DLL")?;

    // Download the DLL
    let response = client
        .get(download_url)
        .header("User-Agent", "WoW-HD-Patcher")
        .send()
        .await
        .map_err(|e| format!("Failed to download: {}", e))?;

    let bytes = response.bytes().await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Write to WoW folder
    let dest_path = wow_path.join("VanillaHelpers.dll");
    std::fs::write(&dest_path, bytes)
        .map_err(|e| format!("Failed to write DLL: {}", e))?;

    Ok(())
}

/// Check for app updates from GitHub
#[derive(serde::Serialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub download_url: Option<String>,
    pub release_notes: Option<String>,
}

#[tauri::command]
async fn check_for_updates() -> Result<UpdateInfo, String> {
    // Current version from Cargo.toml
    let current_version = env!("CARGO_PKG_VERSION").to_string();

    let releases_url = "https://api.github.com/repos/DonutsDelivery/wow-patcher-plus/releases/latest";

    let client = reqwest::Client::new();
    let response = client
        .get(releases_url)
        .header("User-Agent", "WoW-HD-Patcher")
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            let release: serde_json::Value = resp.json().await
                .map_err(|e| format!("Failed to parse release info: {}", e))?;

            let latest_version = release["tag_name"].as_str()
                .unwrap_or("v0.0.0")
                .trim_start_matches('v')
                .to_string();

            let update_available = version_compare(&current_version, &latest_version);

            let download_url = release["assets"].as_array()
                .and_then(|assets| {
                    assets.iter().find(|a| {
                        a["name"].as_str()
                            .map(|n| n.contains("linux") || n.contains("AppImage") || n.ends_with(".deb"))
                            .unwrap_or(false)
                    })
                })
                .and_then(|a| a["browser_download_url"].as_str())
                .map(|s| s.to_string());

            let release_notes = release["body"].as_str().map(|s| s.to_string());

            Ok(UpdateInfo {
                current_version,
                latest_version,
                update_available,
                download_url,
                release_notes,
            })
        }
        _ => {
            // No update info available - could be no releases yet or network error
            Ok(UpdateInfo {
                current_version: current_version.clone(),
                latest_version: current_version,
                update_available: false,
                download_url: None,
                release_notes: None,
            })
        }
    }
}

/// Simple semver comparison - returns true if latest > current
fn version_compare(current: &str, latest: &str) -> bool {
    let parse_version = |v: &str| -> (u32, u32, u32) {
        let parts: Vec<u32> = v.split('.')
            .filter_map(|p| p.parse().ok())
            .collect();
        (
            parts.first().copied().unwrap_or(0),
            parts.get(1).copied().unwrap_or(0),
            parts.get(2).copied().unwrap_or(0),
        )
    };

    let current_v = parse_version(current);
    let latest_v = parse_version(latest);

    latest_v > current_v
}

/// Get Last-Modified date for a patch URL
#[derive(serde::Serialize)]
pub struct PatchFreshness {
    pub patch_id: String,
    pub remote_modified: Option<String>,
    pub local_modified: Option<String>,
    pub needs_update: bool,
}

#[tauri::command]
async fn check_patch_freshness(
    manager: State<'_, InstallManager>,
    patch_id: String,
    download_url: String,
) -> Result<PatchFreshness, String> {
    // Get local file modification time
    let wow_path = manager.get_wow_path();
    let local_modified = wow_path.and_then(|p| {
        let patch_file = p.join("Data").join(format!("Patch-{}.mpq", patch_id.to_uppercase()));
        std::fs::metadata(&patch_file).ok()
            .and_then(|m| m.modified().ok())
            .map(|t| {
                let datetime: chrono::DateTime<chrono::Utc> = t.into();
                datetime.format("%Y-%m-%d %H:%M:%S").to_string()
            })
    });

    // Get remote Last-Modified via HEAD request
    let client = reqwest::Client::new();
    let response = client
        .head(&download_url)
        .header("User-Agent", "WoW-HD-Patcher")
        .send()
        .await;

    let remote_modified = response.ok()
        .and_then(|r| r.headers().get("Last-Modified").cloned())
        .and_then(|h| h.to_str().ok().map(|s| s.to_string()));

    // Determine if update is needed
    let needs_update = match (&local_modified, &remote_modified) {
        (Some(local), Some(remote)) => {
            // Parse and compare dates - this is a simple heuristic
            // In practice, we'd parse both dates properly
            local < remote
        }
        (None, Some(_)) => true, // Not installed, remote exists
        _ => false, // Can't determine
    };

    Ok(PatchFreshness {
        patch_id,
        remote_modified,
        local_modified,
        needs_update,
    })
}

/// Install DXVK from GitHub
#[tauri::command]
async fn install_dxvk(manager: State<'_, InstallManager>, version: Option<String>) -> Result<(), String> {
    let wow_path = manager.get_wow_path()
        .ok_or("WoW path not set")?;

    // Select version - default to 2.7.1 (recommended for NVIDIA)
    let ver = version.as_deref().unwrap_or("2.7.1");
    let download_url = match ver {
        "2.5.3" => "https://github.com/doitsujin/dxvk/releases/download/v2.5.3/dxvk-2.5.3.tar.gz",
        "2.7.1" | _ => "https://github.com/doitsujin/dxvk/releases/download/v2.7.1/dxvk-2.7.1.tar.gz",
    };

    let client = reqwest::Client::new();
    let response = client
        .get(download_url)
        .header("User-Agent", "WoW-HD-Patcher")
        .send()
        .await
        .map_err(|e| format!("Failed to download DXVK: {}", e))?;

    let bytes = response.bytes().await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Extract d3d9.dll from x32 folder in tar.gz
    use std::io::Read;
    let decoder = flate2::read::GzDecoder::new(&bytes[..]);
    let mut archive = tar::Archive::new(decoder);

    for entry in archive.entries().map_err(|e| format!("Failed to read archive: {}", e))? {
        let mut entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path().map_err(|e| format!("Invalid path: {}", e))?;

        // Look for x32/d3d9.dll
        if path.to_string_lossy().contains("x32/d3d9.dll") || path.to_string_lossy().contains("x32\\d3d9.dll") {
            let mut contents = Vec::new();
            entry.read_to_end(&mut contents)
                .map_err(|e| format!("Failed to read DLL: {}", e))?;

            let dest_path = wow_path.join("d3d9.dll");
            std::fs::write(&dest_path, contents)
                .map_err(|e| format!("Failed to write DLL: {}", e))?;

            return Ok(());
        }
    }

    Err("d3d9.dll not found in DXVK archive".to_string())
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

            // Initialize file logging
            let log_path = app_data.join("debug.log");
            if let Ok(log_file) = File::create(&log_path) {
                let _ = CombinedLogger::init(vec![
                    WriteLogger::new(LevelFilter::Debug, Config::default(), log_file),
                ]);
                log::info!("=== WoW HD Patcher started ===");
                log::info!("Log file: {:?}", log_path);
                log::info!("App data: {:?}", app_data);
                log::info!("Downloads: {:?}", downloads_path);
            }

            // Create and register InstallManager
            let install_manager = InstallManager::new(downloads_path);
            app.manage(install_manager);

            Ok(())
        })
        .manage(DownloadManager::new())
        .manage(PatchesCache::new())
        .invoke_handler(tauri::generate_handler![
            // Parser commands
            fetch_patches,
            validate_selection,
            auto_select_deps,
            get_conflicts,
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
            detect_installed_patches,
            uninstall_patches,
            load_saved_wow_path,
            check_requirements,
            install_vanilla_helpers,
            install_dxvk,
            // Update commands
            check_for_updates,
            check_patch_freshness,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
