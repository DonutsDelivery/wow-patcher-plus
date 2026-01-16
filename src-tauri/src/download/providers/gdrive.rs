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
        format!("https://drive.google.com/uc?export=download&id={}", file_id)
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
}
