//! Download manager with parallel execution
//!
//! Provides a centralized download manager that limits concurrent downloads
//! using a Semaphore, resolves URLs via providers, and downloads with resume support.

use std::path::PathBuf;
use std::sync::Arc;
use tauri::ipc::Channel;
use tokio::sync::Semaphore;

use crate::download::{
    progress::DownloadEvent,
    providers::{GoogleDriveProvider, MediafireProvider, DirectDownloadInfo, DownloadProvider},
    resume::download_with_resume,
    DownloadError,
};
use crate::models::DownloadProvider as ProviderType;

/// Maximum number of concurrent downloads
const MAX_CONCURRENT_DOWNLOADS: usize = 3;

/// Centralized download manager with concurrency limiting
///
/// This struct manages parallel downloads with:
/// - Shared HTTP client for connection pooling
/// - Semaphore-based concurrency limiting
/// - Provider-based URL resolution
/// - Resume support for interrupted downloads
#[derive(Clone)]
pub struct DownloadManager {
    client: reqwest::Client,
    semaphore: Arc<Semaphore>,
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DownloadManager {
    /// Create a new download manager
    ///
    /// Initializes with:
    /// - HTTP client with browser-like User-Agent
    /// - 10-hop redirect policy
    /// - Semaphore for 3 concurrent downloads
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_DOWNLOADS)),
        }
    }

    /// Start a download, acquiring semaphore permit for concurrency limiting
    ///
    /// # Arguments
    /// * `share_url` - The share URL to download from (Google Drive or Mediafire)
    /// * `provider_type` - Which provider to use for URL resolution
    /// * `dest_dir` - Directory to save the downloaded file
    /// * `download_id` - Unique identifier for this download
    /// * `on_event` - Channel to send progress events
    ///
    /// # Returns
    /// The full path to the downloaded file on success
    pub async fn download(
        &self,
        share_url: String,
        provider_type: ProviderType,
        dest_dir: PathBuf,
        download_id: String,
        on_event: Channel<DownloadEvent>,
    ) -> Result<String, DownloadError> {
        // Acquire semaphore permit (blocks if MAX_CONCURRENT reached)
        let _permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| DownloadError::ProviderError(format!("Semaphore error: {}", e)))?;

        // Resolve direct URL based on provider
        let info = self.resolve_url(&share_url, provider_type).await?;

        // Determine destination path
        let file_name = info.file_name.unwrap_or_else(|| {
            share_url
                .split('/')
                .last()
                .map(|s| s.split('?').next().unwrap_or(s))
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("{}.download", download_id))
        });
        let dest_path = dest_dir.join(&file_name);

        // Perform download with resume support
        download_with_resume(&self.client, &info.url, &dest_path, download_id, on_event).await?;

        Ok(dest_path.to_string_lossy().to_string())
    }

    /// Resolve share URL to direct download URL
    ///
    /// Dispatches to the appropriate provider based on provider_type.
    async fn resolve_url(
        &self,
        share_url: &str,
        provider_type: ProviderType,
    ) -> Result<DirectDownloadInfo, DownloadError> {
        match provider_type {
            ProviderType::GoogleDrive => {
                let provider = GoogleDriveProvider::new(self.client.clone());
                provider.resolve_direct_url(share_url).await
            }
            ProviderType::Mediafire => {
                let provider = MediafireProvider::new(self.client.clone());
                provider.resolve_direct_url(share_url).await
            }
            ProviderType::Unknown => Err(DownloadError::ProviderError(
                "Unknown download provider".to_string(),
            )),
        }
    }

    /// Get current number of active downloads
    ///
    /// This is calculated as MAX_CONCURRENT - available permits.
    pub fn active_downloads(&self) -> usize {
        MAX_CONCURRENT_DOWNLOADS - self.semaphore.available_permits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_manager_new() {
        let manager = DownloadManager::new();
        assert_eq!(manager.active_downloads(), 0);
    }

    #[test]
    fn test_download_manager_default() {
        let manager = DownloadManager::default();
        assert_eq!(manager.active_downloads(), 0);
    }

    #[test]
    fn test_download_manager_clone() {
        let manager = DownloadManager::new();
        let cloned = manager.clone();
        // Both should share the same semaphore via Arc
        assert_eq!(cloned.active_downloads(), 0);
    }

    #[test]
    fn test_active_downloads_starts_at_zero() {
        let manager = DownloadManager::new();
        // No downloads started, should be 0
        assert_eq!(manager.active_downloads(), 0);
    }
}
