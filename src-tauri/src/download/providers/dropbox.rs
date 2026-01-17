//! Dropbox provider implementation
//!
//! Converts Dropbox share URLs to direct download URLs by changing dl=0 to dl=1

use async_trait::async_trait;
use super::{DirectDownloadInfo, DownloadProvider};
use crate::download::DownloadError;

/// Dropbox download provider
pub struct DropboxProvider {
    client: reqwest::Client,
}

impl DropboxProvider {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    /// Convert Dropbox share URL to direct download URL
    pub fn get_direct_url(share_url: &str) -> String {
        let mut url = share_url.to_string();

        // Replace dl=0 with dl=1 if present
        if url.contains("dl=0") {
            url = url.replace("dl=0", "dl=1");
        } else if !url.contains("dl=1") {
            // Add dl=1 parameter
            if url.contains('?') {
                url.push_str("&dl=1");
            } else {
                url.push_str("?dl=1");
            }
        }

        url
    }
}

#[async_trait]
impl DownloadProvider for DropboxProvider {
    async fn resolve_direct_url(&self, share_url: &str) -> Result<DirectDownloadInfo, DownloadError> {
        let url = Self::get_direct_url(share_url);

        // Make a HEAD request to get file info
        let response = self.client.head(&url).send().await?;

        let content_length = response.content_length();
        let supports_range = response
            .headers()
            .get("accept-ranges")
            .map(|v| v.to_str().unwrap_or("") != "none")
            .unwrap_or(false);

        Ok(DirectDownloadInfo {
            url,
            file_name: None,
            content_length,
            supports_range,
        })
    }

    fn supports_resume(&self) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        "Dropbox"
    }
}
