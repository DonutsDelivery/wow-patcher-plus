//! Repair functionality for patch installations
//!
//! Repair works by re-copying from the downloads folder.
//! If the download is missing, the repair fails (requires re-download first).

use std::path::Path;
use tauri::ipc::Channel;

use super::copier::{install_mpq, get_mpq_filename, InstallEvent};
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
