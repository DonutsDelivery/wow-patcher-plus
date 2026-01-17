//! MEGA.nz provider implementation
//!
//! Downloads files from MEGA.nz shared links using the mega crate.
//! Handles URL parsing, decryption, and streaming downloads.

use async_trait::async_trait;
use std::path::Path;
use tokio::fs::File;
use tokio_util::compat::TokioAsyncWriteCompatExt;

use super::{DirectDownloadInfo, DownloadProvider};
use crate::download::DownloadError;

/// MEGA.nz download provider
pub struct MegaProvider {
    client: reqwest::Client,
}

impl MegaProvider {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    /// Download a file from MEGA to the specified path
    pub async fn download_to_file(
        &self,
        url: &str,
        dest_path: &Path,
    ) -> Result<u64, DownloadError> {
        log::info!("[MEGA] download_to_file called");
        log::info!("[MEGA] URL: {}", url);
        log::info!("[MEGA] Dest: {:?}", dest_path);

        // Create MEGA client with HTTP client
        log::info!("[MEGA] Creating MEGA client...");
        let mut mega_client = mega::Client::builder()
            .build(self.client.clone())
            .map_err(|e| DownloadError::ProviderError(format!("Failed to create MEGA client: {}", e)))?;

        // Fetch public nodes from the URL
        log::info!("[MEGA] Fetching public nodes...");
        let nodes = mega_client
            .fetch_public_nodes(url)
            .await
            .map_err(|e| {
                log::info!("[MEGA] Failed to fetch nodes: {}", e);
                DownloadError::ProviderError(format!("Failed to fetch MEGA file info: {}", e))
            })?;

        // Get the first file node
        let node = nodes
            .roots()
            .find(|n| n.kind().is_file())
            .ok_or_else(|| DownloadError::ProviderError("No file found at MEGA URL".to_string()))?;

        let file_size = node.size();
        log::info!("[MEGA] Found file: {} ({} bytes)", node.name(), file_size);

        // Create destination file
        let file = File::create(dest_path)
            .await?;

        // Create a pipe for streaming
        let (reader, writer) = sluice::pipe::pipe();

        // Spawn task to write data to file
        let handle = tokio::spawn(async move {
            futures_util::io::copy(reader, &mut file.compat_write()).await
        });

        // Download the node to the writer
        mega_client
            .download_node(node, writer)
            .await
            .map_err(|e| DownloadError::ProviderError(format!("Failed to download from MEGA: {}", e)))?;

        // Wait for file write to complete
        handle
            .await
            .map_err(|e| DownloadError::ProviderError(format!("Task join error: {}", e)))?
            .map_err(|e| DownloadError::ProviderError(format!("Copy error: {}", e)))?;

        Ok(file_size)
    }

    /// Get file info from a MEGA URL without downloading
    pub async fn get_file_info(&self, url: &str) -> Result<(String, u64), DownloadError> {
        let mut mega_client = mega::Client::builder()
            .build(self.client.clone())
            .map_err(|e| DownloadError::ProviderError(format!("Failed to create MEGA client: {}", e)))?;

        let nodes = mega_client
            .fetch_public_nodes(url)
            .await
            .map_err(|e| DownloadError::ProviderError(format!("Failed to fetch MEGA file info: {}", e)))?;

        let node = nodes
            .roots()
            .find(|n| n.kind().is_file())
            .ok_or_else(|| DownloadError::ProviderError("No file found at MEGA URL".to_string()))?;

        Ok((node.name().to_string(), node.size()))
    }
}

#[async_trait]
impl DownloadProvider for MegaProvider {
    async fn resolve_direct_url(&self, share_url: &str) -> Result<DirectDownloadInfo, DownloadError> {
        log::info!("[MEGA] resolve_direct_url: {}", share_url);
        let (file_name, file_size) = self.get_file_info(share_url).await?;
        log::info!("[MEGA] Resolved: {} ({} bytes)", file_name, file_size);

        // For MEGA, we return the original URL since we handle download specially
        Ok(DirectDownloadInfo {
            url: share_url.to_string(),
            file_name: Some(file_name),
            content_length: Some(file_size),
            supports_range: false, // MEGA handles its own chunking
        })
    }

    fn supports_resume(&self) -> bool {
        false // MEGA handles downloads differently
    }

    fn name(&self) -> &'static str {
        "MEGA"
    }
}
