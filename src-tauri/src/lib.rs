mod parser;
mod models;
mod download;

use std::collections::HashSet;
use std::path::PathBuf;
use tauri::{ipc::Channel, State};

use models::{PatchModule, PatchId, DownloadProvider as ProviderType};
use parser::{
    forum::{fetch_forum_post_with_fallback, ForumParser, FORUM_URL},
    links::extract_download_links,
    modules::parse_modules,
    dependencies::{validate_module_selection, auto_select_dependencies},
};
use download::{DownloadManager, progress::DownloadEvent};

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .manage(DownloadManager::new())
        .invoke_handler(tauri::generate_handler![
            fetch_patches,
            validate_selection,
            auto_select_deps,
            get_forum_url,
            start_download,
            get_active_downloads,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
