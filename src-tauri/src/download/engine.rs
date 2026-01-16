//! Core streaming download engine
//!
//! Provides memory-efficient file downloads with progress tracking.

use std::path::Path;
use futures_util::StreamExt;
use tauri::ipc::Channel;
use tokio::io::AsyncWriteExt;

use crate::download::{DownloadError, progress::{DownloadEvent, ProgressTracker}};

/// Download a file from a URL with progress reporting
///
/// This function streams the download to avoid loading large files into memory.
/// Progress events are sent via the Tauri Channel at throttled intervals.
///
/// # Arguments
/// * `client` - HTTP client for making requests
/// * `url` - Direct download URL
/// * `dest_path` - Local path to save the file
/// * `download_id` - Unique identifier for this download
/// * `on_event` - Channel to send progress events to frontend
///
/// # Returns
/// Ok(()) on success, or DownloadError on failure
pub async fn download_file(
    _client: &reqwest::Client,
    _url: &str,
    _dest_path: &Path,
    _download_id: String,
    _on_event: Channel<DownloadEvent>,
) -> Result<(), DownloadError> {
    // Implementation will be added in Task 3
    todo!("download_file implementation")
}

/// Extract filename from response headers or URL
fn extract_filename(response: &reqwest::Response, url: &str) -> String {
    // Try Content-Disposition header first
    if let Some(cd) = response.headers().get("content-disposition") {
        if let Ok(cd_str) = cd.to_str() {
            // Look for filename="..." or filename*=UTF-8''...
            if let Some(start) = cd_str.find("filename=") {
                let name_part = &cd_str[start + 9..];
                let name = if name_part.starts_with('"') {
                    // Quoted filename
                    name_part[1..].split('"').next().unwrap_or("download")
                } else {
                    // Unquoted filename
                    name_part.split(';').next().unwrap_or("download").trim()
                };
                if !name.is_empty() {
                    return name.to_string();
                }
            }
        }
    }

    // Fall back to URL path's last segment
    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(segments) = parsed.path_segments() {
            if let Some(last) = segments.last() {
                if !last.is_empty() {
                    return last.to_string();
                }
            }
        }
    }

    // Default fallback
    "download".to_string()
}
