//! Transfer.it provider implementation
//!
//! Handles transfer.it download URLs

use async_trait::async_trait;
use super::{DirectDownloadInfo, DownloadProvider};
use crate::download::DownloadError;

/// Transfer.it download provider
pub struct TransferProvider {
    client: reqwest::Client,
}

impl TransferProvider {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl DownloadProvider for TransferProvider {
    async fn resolve_direct_url(&self, share_url: &str) -> Result<DirectDownloadInfo, DownloadError> {
        // Transfer.it URLs are typically direct or redirect to direct download
        // Follow redirects to get the final URL
        let response = self.client.head(share_url).send().await?;

        let final_url = response.url().to_string();
        let content_length = response.content_length();
        let supports_range = response
            .headers()
            .get("accept-ranges")
            .map(|v| v.to_str().unwrap_or("") != "none")
            .unwrap_or(false);

        Ok(DirectDownloadInfo {
            url: final_url,
            file_name: None,
            content_length,
            supports_range,
        })
    }

    fn supports_resume(&self) -> bool {
        false // Transfer.it links are temporary
    }

    fn name(&self) -> &'static str {
        "Transfer.it"
    }
}
