mod parser;
mod models;

use std::collections::HashSet;
use models::{PatchModule, PatchId};
use parser::{
    forum::{fetch_forum_post_with_fallback, ForumParser, FORUM_URL},
    links::extract_download_links,
    modules::parse_modules,
    dependencies::{validate_module_selection, auto_select_dependencies},
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .invoke_handler(tauri::generate_handler![
            fetch_patches,
            validate_selection,
            auto_select_deps,
            get_forum_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
