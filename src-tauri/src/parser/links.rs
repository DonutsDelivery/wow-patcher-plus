//! Download link extraction using regex

use lazy_static::lazy_static;
use regex::Regex;

use crate::models::{DownloadLink, DownloadProvider};

lazy_static! {
    // Google Drive patterns
    static ref GDRIVE_FILE: Regex = Regex::new(
        r"https://drive\.google\.com/file/d/([A-Za-z0-9_-]+)"
    ).unwrap();

    static ref GDRIVE_OPEN: Regex = Regex::new(
        r"https://drive\.google\.com/open\?id=([A-Za-z0-9_-]+)"
    ).unwrap();

    static ref GDRIVE_UC: Regex = Regex::new(
        r"https://drive\.google\.com/uc\?.*id=([A-Za-z0-9_-]+)"
    ).unwrap();

    // Mediafire patterns
    static ref MEDIAFIRE_FILE: Regex = Regex::new(
        r#"https://(?:www\.)?mediafire\.com/file/([a-z0-9]+)/([^/\s"<>]+)"#
    ).unwrap();

    static ref MEDIAFIRE_FOLDER: Regex = Regex::new(
        r"https://(?:www\.)?(?:app\.)?mediafire\.com/folder/([a-z0-9]+)"
    ).unwrap();

    static ref MEDIAFIRE_VIEW: Regex = Regex::new(
        r#"https://(?:www\.)?mediafire\.com/view/([a-z0-9]+)/([^/\s"<>]+)"#
    ).unwrap();
}

/// Extract all download links from content (HTML or plain text)
pub fn extract_download_links(content: &str) -> Vec<DownloadLink> {
    let mut links = Vec::new();

    // Extract Google Drive file links
    for cap in GDRIVE_FILE.captures_iter(content) {
        links.push(DownloadLink {
            provider: DownloadProvider::GoogleDrive,
            url: cap[0].to_string(),
            file_name: None,
        });
    }

    // Extract Google Drive open links
    for cap in GDRIVE_OPEN.captures_iter(content) {
        links.push(DownloadLink {
            provider: DownloadProvider::GoogleDrive,
            url: cap[0].to_string(),
            file_name: None,
        });
    }

    // Extract Google Drive uc links
    for cap in GDRIVE_UC.captures_iter(content) {
        links.push(DownloadLink {
            provider: DownloadProvider::GoogleDrive,
            url: cap[0].to_string(),
            file_name: None,
        });
    }

    // Extract Mediafire file links (with filename)
    for cap in MEDIAFIRE_FILE.captures_iter(content) {
        links.push(DownloadLink {
            provider: DownloadProvider::Mediafire,
            url: cap[0].to_string(),
            file_name: Some(cap[2].to_string()),
        });
    }

    // Extract Mediafire view links (with filename)
    for cap in MEDIAFIRE_VIEW.captures_iter(content) {
        links.push(DownloadLink {
            provider: DownloadProvider::Mediafire,
            url: cap[0].to_string(),
            file_name: Some(cap[2].to_string()),
        });
    }

    // Extract Mediafire folder links
    for cap in MEDIAFIRE_FOLDER.captures_iter(content) {
        links.push(DownloadLink {
            provider: DownloadProvider::Mediafire,
            url: cap[0].to_string(),
            file_name: None,
        });
    }

    // Deduplicate by URL
    links.sort_by(|a, b| a.url.cmp(&b.url));
    links.dedup_by(|a, b| a.url == b.url);

    links
}

/// Extract Google Drive file ID from a URL
pub fn extract_gdrive_id(url: &str) -> Option<String> {
    if let Some(cap) = GDRIVE_FILE.captures(url) {
        return Some(cap[1].to_string());
    }
    if let Some(cap) = GDRIVE_OPEN.captures(url) {
        return Some(cap[1].to_string());
    }
    if let Some(cap) = GDRIVE_UC.captures(url) {
        return Some(cap[1].to_string());
    }
    None
}

/// Extract Mediafire file ID from a URL
pub fn extract_mediafire_id(url: &str) -> Option<String> {
    if let Some(cap) = MEDIAFIRE_FILE.captures(url) {
        return Some(cap[1].to_string());
    }
    if let Some(cap) = MEDIAFIRE_VIEW.captures(url) {
        return Some(cap[1].to_string());
    }
    if let Some(cap) = MEDIAFIRE_FOLDER.captures(url) {
        return Some(cap[1].to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_mediafire_file_links() {
        let content = r#"
            Download here: https://www.mediafire.com/file/abc123xyz/patch-a.7z
            Also: https://mediafire.com/file/def456/patch-b.rar/file
        "#;

        let links = extract_download_links(content);

        assert_eq!(links.len(), 2);
        assert!(links.iter().all(|l| l.provider == DownloadProvider::Mediafire));
        assert!(links.iter().any(|l| l.file_name.as_deref() == Some("patch-a.7z")));
    }

    #[test]
    fn test_extract_gdrive_file_links() {
        let content = r#"
            Google Drive: https://drive.google.com/file/d/1ABC-def_GHI/view?usp=sharing
            Alt link: https://drive.google.com/open?id=2XYZ_abc-123
        "#;

        let links = extract_download_links(content);

        assert_eq!(links.len(), 2);
        assert!(links.iter().all(|l| l.provider == DownloadProvider::GoogleDrive));
    }

    #[test]
    fn test_extract_mixed_links() {
        let content = r#"
            Mediafire: https://www.mediafire.com/file/abc123/patch.7z
            Google: https://drive.google.com/file/d/XYZ789/view
            Invalid: https://example.com/download
        "#;

        let links = extract_download_links(content);

        assert_eq!(links.len(), 2); // Should not include example.com
        assert!(links.iter().any(|l| l.provider == DownloadProvider::Mediafire));
        assert!(links.iter().any(|l| l.provider == DownloadProvider::GoogleDrive));
    }

    #[test]
    fn test_extract_gdrive_id() {
        assert_eq!(
            extract_gdrive_id("https://drive.google.com/file/d/1ABC-def_GHI/view"),
            Some("1ABC-def_GHI".to_string())
        );
        assert_eq!(
            extract_gdrive_id("https://drive.google.com/open?id=XYZ123"),
            Some("XYZ123".to_string())
        );
        assert_eq!(extract_gdrive_id("https://example.com"), None);
    }

    #[test]
    fn test_extract_mediafire_id() {
        assert_eq!(
            extract_mediafire_id("https://www.mediafire.com/file/abc123/file.zip"),
            Some("abc123".to_string())
        );
        assert_eq!(extract_mediafire_id("https://example.com"), None);
    }

    #[test]
    fn test_deduplication() {
        let content = r#"
            https://www.mediafire.com/file/abc123/patch.7z
            https://www.mediafire.com/file/abc123/patch.7z
            https://www.mediafire.com/file/abc123/patch.7z
        "#;

        let links = extract_download_links(content);
        assert_eq!(links.len(), 1); // Should be deduplicated
    }
}
