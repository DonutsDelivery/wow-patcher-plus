//! Google Drive provider implementation
//!
//! Resolves Google Drive share URLs to direct download URLs, handling:
//! - Various URL formats (file/d/, open?id=, uc?id=)
//! - Virus scan confirmation page for large files (>100MB)

use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::Regex;
use scraper::{Html, Selector};

use super::{DirectDownloadInfo, DownloadProvider};
use crate::download::DownloadError;

lazy_static! {
    /// Matches: drive.google.com/file/d/FILE_ID/...
    static ref FILE_D_PATTERN: Regex = Regex::new(r"drive\.google\.com/file/d/([a-zA-Z0-9_-]+)").unwrap();
    /// Matches: drive.google.com/open?id=FILE_ID
    static ref OPEN_ID_PATTERN: Regex = Regex::new(r"drive\.google\.com/open\?id=([a-zA-Z0-9_-]+)").unwrap();
    /// Matches: drive.google.com/uc?id=FILE_ID or ?export=download&id=FILE_ID
    static ref UC_ID_PATTERN: Regex = Regex::new(r"[?&]id=([a-zA-Z0-9_-]+)").unwrap();
    /// Matches confirm token in virus scan warning page
    static ref CONFIRM_REGEX: Regex = Regex::new(r"confirm=([0-9A-Za-z_-]+)").unwrap();
}

/// Google Drive download provider
///
/// Handles resolving Google Drive share URLs to direct download URLs,
/// including handling the virus scan confirmation dialog for large files.
pub struct GoogleDriveProvider {
    client: reqwest::Client,
}

impl GoogleDriveProvider {
    /// Create a new Google Drive provider with the given HTTP client
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    /// Extract file ID from various Google Drive URL formats
    ///
    /// Supports:
    /// - `https://drive.google.com/file/d/FILE_ID/view?usp=sharing`
    /// - `https://drive.google.com/open?id=FILE_ID`
    /// - `https://drive.google.com/uc?id=FILE_ID`
    /// - `https://drive.google.com/uc?export=download&id=FILE_ID`
    pub fn extract_file_id(url: &str) -> Option<String> {
        // Try each pattern in order of specificity
        if let Some(cap) = FILE_D_PATTERN.captures(url) {
            return Some(cap[1].to_string());
        }
        if let Some(cap) = OPEN_ID_PATTERN.captures(url) {
            return Some(cap[1].to_string());
        }
        if let Some(cap) = UC_ID_PATTERN.captures(url) {
            return Some(cap[1].to_string());
        }
        None
    }

    /// Convert file ID to direct download URL
    pub fn get_direct_url(file_id: &str) -> String {
        format!(
            "https://drive.google.com/uc?export=download&id={}",
            file_id
        )
    }

    /// Handle large file confirmation (>100MB virus scan warning)
    /// Returns the final direct download URL after any required confirmation
    async fn resolve_with_confirmation(
        &self,
        file_id: &str,
    ) -> Result<DirectDownloadInfo, DownloadError> {
        let initial_url = Self::get_direct_url(file_id);

        let response = self.client.get(&initial_url).send().await?;

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        // If we got HTML, it's the virus scan warning page
        if content_type.contains("text/html") {
            let html = response.text().await?;
            return self.parse_confirmation_page(&html, file_id);
        }

        // No confirmation needed - extract info from response
        let content_length = response.content_length();
        let file_name = Self::extract_filename_from_headers(response.headers());

        // Check if Range requests are supported
        let supports_range = response
            .headers()
            .get("accept-ranges")
            .map(|v| v.to_str().unwrap_or("") != "none")
            .unwrap_or(false);

        Ok(DirectDownloadInfo {
            url: initial_url,
            file_name,
            content_length,
            supports_range,
        })
    }

    /// Parse the virus scan warning page to extract the confirmation URL
    fn parse_confirmation_page(
        &self,
        html: &str,
        file_id: &str,
    ) -> Result<DirectDownloadInfo, DownloadError> {
        let document = Html::parse_document(html);

        // Method 1: Look for download link with confirm parameter
        let link_selector = Selector::parse("a[href*='confirm=']").unwrap();
        if let Some(element) = document.select(&link_selector).next() {
            if let Some(href) = element.value().attr("href") {
                let url = if href.starts_with('/') {
                    format!("https://drive.google.com{}", href)
                } else if href.starts_with("http") {
                    href.to_string()
                } else {
                    return Err(DownloadError::ConfirmationFailed);
                };

                return Ok(DirectDownloadInfo {
                    url,
                    file_name: None,
                    content_length: None,
                    supports_range: true, // Assume yes after confirmation
                });
            }
        }

        // Method 2: Look for form with action containing download URL
        let form_selector = Selector::parse("form[action*='/uc']").unwrap();
        if let Some(form) = document.select(&form_selector).next() {
            if let Some(action) = form.value().attr("action") {
                // Check for hidden input fields with confirm parameter
                let input_selector = Selector::parse("input[name='confirm']").unwrap();
                if let Some(confirm_input) = document.select(&input_selector).next() {
                    if let Some(confirm_value) = confirm_input.value().attr("value") {
                        let url = format!(
                            "https://drive.google.com/uc?export=download&confirm={}&id={}",
                            confirm_value, file_id
                        );
                        return Ok(DirectDownloadInfo {
                            url,
                            file_name: None,
                            content_length: None,
                            supports_range: true,
                        });
                    }
                }

                // Fallback: use form action directly
                let url = if action.starts_with('/') {
                    format!("https://drive.google.com{}", action)
                } else {
                    action.to_string()
                };

                return Ok(DirectDownloadInfo {
                    url,
                    file_name: None,
                    content_length: None,
                    supports_range: true,
                });
            }
        }

        // Method 3: Extract confirm token from HTML via regex
        if let Some(cap) = CONFIRM_REGEX.captures(html) {
            let url = format!(
                "https://drive.google.com/uc?export=download&confirm={}&id={}",
                &cap[1], file_id
            );
            return Ok(DirectDownloadInfo {
                url,
                file_name: None,
                content_length: None,
                supports_range: true,
            });
        }

        Err(DownloadError::ConfirmationFailed)
    }

    /// Extract filename from Content-Disposition header
    fn extract_filename_from_headers(
        headers: &reqwest::header::HeaderMap,
    ) -> Option<String> {
        headers
            .get("content-disposition")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| {
                // Parse: attachment; filename="patch-A.rar" or filename*=UTF-8''patch-A.rar
                if let Some(start) = v.find("filename=") {
                    let rest = &v[start + 9..];
                    let filename = rest
                        .split(';')
                        .next()
                        .unwrap_or(rest)
                        .trim()
                        .trim_matches('"');
                    if !filename.is_empty() {
                        return Some(filename.to_string());
                    }
                }
                None
            })
    }
}

#[async_trait]
impl DownloadProvider for GoogleDriveProvider {
    /// Resolve a Google Drive share URL to a direct download URL
    async fn resolve_direct_url(
        &self,
        share_url: &str,
    ) -> Result<DirectDownloadInfo, DownloadError> {
        let file_id = Self::extract_file_id(share_url).ok_or_else(|| {
            DownloadError::ProviderError(
                "Could not extract Google Drive file ID from URL".to_string(),
            )
        })?;

        self.resolve_with_confirmation(&file_id).await
    }

    /// Google Drive generally supports Range headers for resumable downloads
    fn supports_resume(&self) -> bool {
        true
    }

    /// Provider name for logging and display
    fn name(&self) -> &'static str {
        "Google Drive"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_file_id_file_d_format() {
        let url = "https://drive.google.com/file/d/1ABC123xyz_-/view?usp=sharing";
        assert_eq!(
            GoogleDriveProvider::extract_file_id(url),
            Some("1ABC123xyz_-".to_string())
        );
    }

    #[test]
    fn test_extract_file_id_file_d_no_view() {
        let url = "https://drive.google.com/file/d/1ABC123xyz_-";
        assert_eq!(
            GoogleDriveProvider::extract_file_id(url),
            Some("1ABC123xyz_-".to_string())
        );
    }

    #[test]
    fn test_extract_file_id_open_id_format() {
        let url = "https://drive.google.com/open?id=1ABC123xyz_-";
        assert_eq!(
            GoogleDriveProvider::extract_file_id(url),
            Some("1ABC123xyz_-".to_string())
        );
    }

    #[test]
    fn test_extract_file_id_uc_id_format() {
        let url = "https://drive.google.com/uc?id=1ABC123xyz_-";
        assert_eq!(
            GoogleDriveProvider::extract_file_id(url),
            Some("1ABC123xyz_-".to_string())
        );
    }

    #[test]
    fn test_extract_file_id_uc_export_download() {
        let url = "https://drive.google.com/uc?export=download&id=1ABC123xyz_-";
        assert_eq!(
            GoogleDriveProvider::extract_file_id(url),
            Some("1ABC123xyz_-".to_string())
        );
    }

    #[test]
    fn test_extract_file_id_invalid_url() {
        let url = "https://example.com/file.txt";
        assert_eq!(GoogleDriveProvider::extract_file_id(url), None);
    }

    #[test]
    fn test_extract_file_id_empty_url() {
        let url = "";
        assert_eq!(GoogleDriveProvider::extract_file_id(url), None);
    }

    #[test]
    fn test_get_direct_url() {
        let file_id = "1ABC123xyz_-";
        let expected = "https://drive.google.com/uc?export=download&id=1ABC123xyz_-";
        assert_eq!(GoogleDriveProvider::get_direct_url(file_id), expected);
    }

    #[test]
    fn test_parse_confirmation_page_with_link() {
        let client = reqwest::Client::new();
        let provider = GoogleDriveProvider::new(client);

        let html = r#"
        <html>
        <body>
            <a href="https://drive.google.com/uc?export=download&confirm=ABC123&id=FILE123">Download anyway</a>
        </body>
        </html>
        "#;

        let result = provider.parse_confirmation_page(html, "FILE123");
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(info.url.contains("confirm=ABC123"));
    }

    #[test]
    fn test_parse_confirmation_page_with_relative_link() {
        let client = reqwest::Client::new();
        let provider = GoogleDriveProvider::new(client);

        let html = r#"
        <html>
        <body>
            <a href="/uc?export=download&confirm=XYZ789&id=FILE123">Download anyway</a>
        </body>
        </html>
        "#;

        let result = provider.parse_confirmation_page(html, "FILE123");
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(
            info.url,
            "https://drive.google.com/uc?export=download&confirm=XYZ789&id=FILE123"
        );
    }

    #[test]
    fn test_parse_confirmation_page_with_form() {
        let client = reqwest::Client::new();
        let provider = GoogleDriveProvider::new(client);

        let html = r#"
        <html>
        <body>
            <form action="/uc?export=download" method="POST">
                <input type="hidden" name="confirm" value="TOKEN123">
                <input type="hidden" name="id" value="FILE123">
            </form>
        </body>
        </html>
        "#;

        let result = provider.parse_confirmation_page(html, "FILE123");
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(info.url.contains("confirm=TOKEN123"));
    }

    #[test]
    fn test_parse_confirmation_page_regex_fallback() {
        let client = reqwest::Client::new();
        let provider = GoogleDriveProvider::new(client);

        let html = r#"
        <html>
        <body>
            <script>
                var url = "https://drive.google.com/uc?export=download&confirm=REGEX_TOKEN&id=FILE123";
            </script>
        </body>
        </html>
        "#;

        let result = provider.parse_confirmation_page(html, "FILE123");
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(info.url.contains("confirm=REGEX_TOKEN"));
    }

    #[test]
    fn test_parse_confirmation_page_failure() {
        let client = reqwest::Client::new();
        let provider = GoogleDriveProvider::new(client);

        let html = r#"
        <html>
        <body>
            <p>No download link here</p>
        </body>
        </html>
        "#;

        let result = provider.parse_confirmation_page(html, "FILE123");
        assert!(result.is_err());
    }

    #[test]
    fn test_provider_name() {
        let client = reqwest::Client::new();
        let provider = GoogleDriveProvider::new(client);
        assert_eq!(provider.name(), "Google Drive");
    }

    #[test]
    fn test_supports_resume() {
        let client = reqwest::Client::new();
        let provider = GoogleDriveProvider::new(client);
        assert!(provider.supports_resume());
    }
}
