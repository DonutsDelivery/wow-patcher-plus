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
    ///
    /// Automatically clears the WDB cache folder before installing.
    pub async fn install_patches(
        &self,
        patch_ids: &[&str],
        on_event: Channel<InstallEvent>,
    ) -> Vec<Result<PathBuf, InstallError>> {
        // Clear WDB folder before installing mods (required for mods to work properly)
        if let Err(e) = self.clear_wdb_folder().await {
            log::warn!("[Install] Failed to clear WDB folder: {:?}", e);
        }

        let mut results = Vec::with_capacity(patch_ids.len());

        for id in patch_ids {
            let result = self.install_patch(id, on_event.clone()).await;
            results.push(result);
        }

        results
    }

    /// Clear the WDB cache folder and replace with empty file
    ///
    /// The WDB folder contains cached data that must be cleared when mods are
    /// installed or updated. Per the official guide:
    /// 1. Delete the existing WDB folder
    /// 2. Create an empty file named "WDB" (prevents WoW from recreating cache)
    pub async fn clear_wdb_folder(&self) -> Result<(), InstallError> {
        // Get path without holding lock across await
        let wdb_path = {
            let wow_path = self.wow_path.read().unwrap();
            let path = wow_path.as_ref().ok_or(InstallError::WowPathNotSet)?;
            path.join("WDB")
        };

        // Remove WDB whether it's a folder or file
        if wdb_path.exists() {
            log::info!("[Install] Removing WDB: {:?}", wdb_path);
            if wdb_path.is_dir() {
                tokio::fs::remove_dir_all(&wdb_path).await?;
            } else {
                tokio::fs::remove_file(&wdb_path).await?;
            }
        }

        // Create empty file named WDB (prevents WoW from recreating cache folder)
        log::info!("[Install] Creating empty WDB file to prevent cache recreation");
        tokio::fs::File::create(&wdb_path).await?;
        log::info!("[Install] WDB cleared and replaced with empty file");

        Ok(())
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
    ///
    /// Automatically clears the WDB cache folder before repairing/updating.
    pub async fn repair_patches(
        &self,
        patch_ids: &[&str],
        on_event: Channel<InstallEvent>,
    ) -> Result<Vec<RepairResult>, InstallError> {
        // Clear WDB folder before updating mods (required for mods to work properly)
        if let Err(e) = self.clear_wdb_folder().await {
            log::warn!("[Install] Failed to clear WDB folder: {:?}", e);
        }

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
