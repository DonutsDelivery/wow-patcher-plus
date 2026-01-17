//! Mediafire provider implementation
//!
//! Resolves Mediafire share URLs to direct download URLs by parsing the share page HTML.
//! Handles dynamic numbered subdomains (download1, download2, etc.) that Mediafire uses.

use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::Regex;
use scraper::{Html, Selector};

use super::{DirectDownloadInfo, DownloadProvider};
use crate::download::DownloadError;

lazy_static! {
    /// Matches mediafire.com share URLs
    static ref MEDIAFIRE_SHARE_PATTERN: Regex = Regex::new(
        r"(?:www\.)?mediafire\.com/(?:file|view|download|folder)/([a-zA-Z0-9]+)"
    ).unwrap();

    /// Matches the actual download URL with numbered subdomain
    static ref MEDIAFIRE_DOWNLOAD_URL: Regex = Regex::new(
        r#"https://download\d+\.mediafire\.com/[^'"<>\s]+"#
    ).unwrap();

    /// Matches pre-download URL with dkey parameter (fallback)
    static ref MEDIAFIRE_DKEY_URL: Regex = Regex::new(
        r#"https?://(?:www\.)?mediafire\.com/(?:file|view|download)/[^'"\?]+\?dkey=[^'"<>\s]+"#
    ).unwrap();
}

/// Mediafire download provider
///
/// Resolves Mediafire share URLs by fetching the share page and extracting
/// the direct download URL with its numbered subdomain.
pub struct MediafireProvider {
    client: reqwest::Client,
}

impl MediafireProvider {
    /// Create a new Mediafire provider with the given HTTP client
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    /// Validate that URL is a Mediafire share URL
    pub fn is_mediafire_url(url: &str) -> bool {
        MEDIAFIRE_SHARE_PATTERN.is_match(url)
    }

    /// Extract the direct download URL from page HTML
    fn extract_download_url(html: &str) -> Option<String> {
        // Primary: Look for download URL with numbered subdomain
        if let Some(m) = MEDIAFIRE_DOWNLOAD_URL.find(html) {
            return Some(m.as_str().to_string());
        }
        None
    }

    /// Extract dkey URL as fallback (needs second fetch)
    fn extract_dkey_url(html: &str) -> Option<String> {
        if let Some(m) = MEDIAFIRE_DKEY_URL.find(html) {
            return Some(m.as_str().to_string());
        }
        None
    }

    /// Extract filename from Mediafire page HTML
    fn extract_filename_from_page(html: &str) -> Option<String> {
        let document = Html::parse_document(html);

        // Try: div.filename
        if let Ok(filename_selector) = Selector::parse("div.filename") {
            if let Some(elem) = document.select(&filename_selector).next() {
                let text = elem.text().collect::<String>().trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }

        // Try: download button text or title attribute
        if let Ok(download_selector) = Selector::parse("a.input[aria-label*='Download']") {
            if let Some(elem) = document.select(&download_selector).next() {
                if let Some(title) = elem.value().attr("title") {
                    return Some(title.to_string());
                }
            }
        }

        None
    }

    /// Fetch the share page and extract the direct download URL
    async fn resolve_from_share_page(
        &self,
        share_url: &str,
    ) -> Result<DirectDownloadInfo, DownloadError> {
        log::info!("[MediaFire] Fetching share page: {}", share_url);

        // Step 1: Fetch the share page with browser-like headers
        let response = self
            .client
            .get(share_url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            )
            .header(
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            )
            .send()
            .await
            .map_err(DownloadError::RequestError)?;

        log::info!("[MediaFire] Share page response status: {}", response.status());

        if !response.status().is_success() {
            log::info!("[MediaFire] Share page fetch failed!");
            return Err(DownloadError::HttpError(response.status()));
        }

        let html = response
            .text()
            .await
            .map_err(DownloadError::RequestError)?;

        log::info!("[MediaFire] Got HTML ({} bytes)", html.len());

        // Step 2: Try to find direct download URL
        if let Some(url) = Self::extract_download_url(&html) {
            let file_name = Self::extract_filename_from_page(&html);
            log::info!("[MediaFire] Found direct URL: {}", url);
            log::info!("[MediaFire] Extracted filename: {:?}", file_name);
            return Ok(DirectDownloadInfo {
                url,
                file_name,
                content_length: None, // Will be determined during download
                supports_range: true, // Mediafire generally supports Range
            });
        }

        log::info!("[MediaFire] No direct URL found, trying dkey fallback...");

        // Step 3: If not found, try dkey URL and follow it
        if let Some(dkey_url) = Self::extract_dkey_url(&html) {
            // Small delay to avoid rate limiting (as per research)
            tokio::time::sleep(std::time::Duration::from_millis(1500)).await;

            let response2 = self
                .client
                .get(&dkey_url)
                .header(
                    "User-Agent",
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
                )
                .send()
                .await
                .map_err(DownloadError::RequestError)?;

            let html2 = response2
                .text()
                .await
                .map_err(DownloadError::RequestError)?;

            if let Some(url) = Self::extract_download_url(&html2) {
                let file_name = Self::extract_filename_from_page(&html2)
                    .or_else(|| Self::extract_filename_from_page(&html));
                return Ok(DirectDownloadInfo {
                    url,
                    file_name,
                    content_length: None,
                    supports_range: true,
                });
            }
        }

        Err(DownloadError::DirectUrlNotFound)
    }
}

#[async_trait]
impl DownloadProvider for MediafireProvider {
    async fn resolve_direct_url(&self, share_url: &str) -> Result<DirectDownloadInfo, DownloadError> {
        if !Self::is_mediafire_url(share_url) {
            return Err(DownloadError::ProviderError(
                "Not a valid Mediafire URL".to_string(),
            ));
        }

        self.resolve_from_share_page(share_url).await
    }

    fn supports_resume(&self) -> bool {
        true // Mediafire generally supports Range headers
    }

    fn name(&self) -> &'static str {
        "Mediafire"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_mediafire_url_valid_file() {
        assert!(MediafireProvider::is_mediafire_url(
            "https://www.mediafire.com/file/abc123/patch.rar"
        ));
    }

    #[test]
    fn test_is_mediafire_url_valid_folder() {
        assert!(MediafireProvider::is_mediafire_url(
            "https://mediafire.com/folder/abc123"
        ));
    }

    #[test]
    fn test_is_mediafire_url_valid_view() {
        assert!(MediafireProvider::is_mediafire_url(
            "https://www.mediafire.com/view/xyz789/file.txt"
        ));
    }

    #[test]
    fn test_is_mediafire_url_valid_download() {
        assert!(MediafireProvider::is_mediafire_url(
            "https://mediafire.com/download/abc123"
        ));
    }

    #[test]
    fn test_is_mediafire_url_invalid() {
        assert!(!MediafireProvider::is_mediafire_url(
            "https://example.com/file"
        ));
        assert!(!MediafireProvider::is_mediafire_url(
            "https://drive.google.com/file/d/abc123"
        ));
    }

    #[test]
    fn test_extract_download_url_numbered_subdomain() {
        let html = r#"
            <html>
                <a href="https://download1502.mediafire.com/abc123/patch.rar">Download</a>
            </html>
        "#;
        let result = MediafireProvider::extract_download_url(html);
        assert!(result.is_some());
        assert!(result.unwrap().starts_with("https://download1502.mediafire.com"));
    }

    #[test]
    fn test_extract_download_url_different_subdomain() {
        let html = r#"
            <script>
                var downloadUrl = "https://download42.mediafire.com/xyz789/file.zip";
            </script>
        "#;
        let result = MediafireProvider::extract_download_url(html);
        assert!(result.is_some());
        assert!(result.unwrap().contains("download42.mediafire.com"));
    }

    #[test]
    fn test_extract_download_url_not_found() {
        let html = r#"
            <html>
                <a href="https://www.mediafire.com/file/abc123">Share Link</a>
            </html>
        "#;
        let result = MediafireProvider::extract_download_url(html);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_dkey_url() {
        let html = r#"
            <html>
                <a href="https://www.mediafire.com/file/abc123/file.rar?dkey=xyz789token">
            </html>
        "#;
        let result = MediafireProvider::extract_dkey_url(html);
        assert!(result.is_some());
        assert!(result.unwrap().contains("dkey="));
    }

    #[test]
    fn test_extract_filename_from_page() {
        let html = r#"
            <html>
                <div class="filename">patch-A.rar</div>
            </html>
        "#;
        assert_eq!(
            MediafireProvider::extract_filename_from_page(html),
            Some("patch-A.rar".to_string())
        );
    }

    #[test]
    fn test_extract_filename_from_page_not_found() {
        let html = r#"
            <html>
                <div class="something-else">Not a filename</div>
            </html>
        "#;
        assert_eq!(MediafireProvider::extract_filename_from_page(html), None);
    }
}
