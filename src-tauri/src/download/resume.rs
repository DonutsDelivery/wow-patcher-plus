//! Resume capability for interrupted downloads
//!
//! Provides download functionality with HTTP Range header support,
//! allowing interrupted downloads to resume from where they left off.

use futures_util::StreamExt;
use reqwest::header::{CONTENT_RANGE, RANGE};
use std::path::Path;
use tauri::ipc::Channel;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

use crate::download::{
    progress::{DownloadEvent, ProgressTracker},
    DownloadError,
};

/// Download a file with resume support
///
/// If `dest_path` exists with partial content, this function attempts to resume
/// the download from where it left off using HTTP Range headers.
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
///
/// # Resume Behavior
/// - If file exists, sends `Range: bytes=<size>-` header
/// - 206 Partial Content: Resume from existing position
/// - 200 OK: Server doesn't support Range, restart from scratch
/// - 416 Range Not Satisfiable: File already complete
pub async fn download_with_resume(
    client: &reqwest::Client,
    url: &str,
    dest_path: &Path,
    download_id: String,
    on_event: Channel<DownloadEvent>,
) -> Result<(), DownloadError> {
    // Check for existing partial file
    let start_pos = if dest_path.exists() {
        tokio::fs::metadata(dest_path)
            .await
            .map(|m| m.len())
            .unwrap_or(0)
    } else {
        0
    };

    log::info!("[Resume] Downloading from URL: {}", url);
    log::info!("[Resume] Start position: {}", start_pos);

    // Build request with Range header if resuming
    let mut request = client.get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("Accept", "*/*")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Connection", "keep-alive");

    if start_pos > 0 {
        request = request.header(RANGE, format!("bytes={}-", start_pos));
        log::info!("[Resume] Adding Range header: bytes={}-", start_pos);
    }

    log::info!("[Resume] Sending request...");
    let response = match request.send().await {
        Ok(r) => r,
        Err(e) => {
            log::info!("[Resume] Request failed: {:?}", e);
            return Err(DownloadError::RequestError(e));
        }
    };
    let status = response.status();
    log::info!("[Resume] Response status: {}", status);

    // Determine actual start position and total size based on response
    let (actual_start, total_size, response) = if status == reqwest::StatusCode::PARTIAL_CONTENT {
        // Server accepted Range request (206 response)
        let total = parse_content_range_total(response.headers())?;
        (start_pos, total, response)
    } else if status == reqwest::StatusCode::OK {
        // Server doesn't support Range, or file changed - start fresh
        let total = response.content_length().unwrap_or(0);
        (0, total, response)
    } else if status == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
        // File already complete (416 response)
        let _total = start_pos;
        on_event
            .send(DownloadEvent::Completed {
                download_id,
                file_path: dest_path.to_string_lossy().to_string(),
            })
            .map_err(|e| DownloadError::ChannelError(e.to_string()))?;
        return Ok(());
    } else if status == reqwest::StatusCode::BAD_REQUEST && start_pos > 0 {
        // 400 error while resuming - URL likely expired (MediaFire, etc.)
        // Delete partial file and retry from scratch
        log::info!("[Resume] Got 400 while resuming, deleting partial file and retrying fresh");
        drop(response);
        let _ = tokio::fs::remove_file(dest_path).await;

        // Retry without Range header
        let retry_request = client.get(url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("Accept", "*/*")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Connection", "keep-alive");

        log::info!("[Resume] Retrying download from start...");
        let retry_response = retry_request.send().await.map_err(DownloadError::RequestError)?;
        let retry_status = retry_response.status();
        log::info!("[Resume] Retry response status: {}", retry_status);

        if !retry_status.is_success() {
            return Err(DownloadError::HttpError(retry_status));
        }

        let total = retry_response.content_length().unwrap_or(0);
        (0, total, retry_response)
    } else {
        return Err(DownloadError::HttpError(status));
    };

    // Extract filename from response
    let file_name = extract_filename(&response, url);

    log::info!("[Resume] Total size: {} bytes", total_size);
    log::info!("[Resume] Actual start position: {}", actual_start);

    // Send started event
    on_event
        .send(DownloadEvent::Started {
            download_id: download_id.clone(),
            file_name,
            total_bytes: total_size,
        })
        .map_err(|e| DownloadError::ChannelError(e.to_string()))?;

    // Create parent directories if they don't exist
    if let Some(parent) = dest_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Open file in appropriate mode
    let mut file = if actual_start > 0 {
        let mut f = OpenOptions::new()
            .write(true)
            .open(dest_path)
            .await
            .map_err(DownloadError::IoError)?;
        f.seek(std::io::SeekFrom::End(0))
            .await
            .map_err(DownloadError::IoError)?;
        f
    } else {
        tokio::fs::File::create(dest_path)
            .await
            .map_err(DownloadError::IoError)?
    };

    // Create progress tracker starting from actual_start
    let mut tracker = ProgressTracker::new(download_id.clone(), total_size);
    tracker.set_downloaded(actual_start);

    // Stream download
    let mut stream = response.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(DownloadError::RequestError)?;
        file.write_all(&chunk)
            .await
            .map_err(DownloadError::IoError)?;

        if let Some(event) = tracker.update(chunk.len() as u64) {
            let _ = on_event.send(event); // Ignore throttled sends
        }
    }

    // Ensure final progress is sent
    file.flush().await.map_err(DownloadError::IoError)?;

    on_event
        .send(DownloadEvent::Completed {
            download_id,
            file_path: dest_path.to_string_lossy().to_string(),
        })
        .map_err(|e| DownloadError::ChannelError(e.to_string()))?;

    Ok(())
}

/// Parse total file size from Content-Range header
///
/// Format: `Content-Range: bytes 1000-1999/5000`
/// Returns the total (5000 in the example), or 0 if unknown.
fn parse_content_range_total(
    headers: &reqwest::header::HeaderMap,
) -> Result<u64, DownloadError> {
    if let Some(range) = headers.get(CONTENT_RANGE) {
        let s = range
            .to_str()
            .map_err(|_| DownloadError::ProviderError("Invalid Content-Range header".to_string()))?;
        if let Some(total) = s.split('/').last() {
            if total != "*" {
                return total.parse().map_err(|_| {
                    DownloadError::ProviderError("Invalid Content-Range total".to_string())
                });
            }
        }
    }
    // If no Content-Range or unknown total, return 0
    Ok(0)
}

/// Extract filename from response headers or URL
fn extract_filename(response: &reqwest::Response, url: &str) -> String {
    // Try Content-Disposition header first
    if let Some(cd) = response.headers().get("content-disposition") {
        if let Ok(s) = cd.to_str() {
            if let Some(name) = s.split("filename=").nth(1) {
                let clean = name.trim_matches(|c| c == '"' || c == '\'').to_string();
                if !clean.is_empty() {
                    return clean;
                }
            }
        }
    }

    // Fall back to URL path
    url.split('/')
        .last()
        .map(|s| s.split('?').next().unwrap_or(s))
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "download".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderMap, HeaderValue};

    #[test]
    fn test_parse_content_range_total_valid() {
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_RANGE,
            HeaderValue::from_static("bytes 1000-1999/5000"),
        );
        let result = parse_content_range_total(&headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 5000);
    }

    #[test]
    fn test_parse_content_range_total_unknown() {
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_RANGE,
            HeaderValue::from_static("bytes 1000-1999/*"),
        );
        let result = parse_content_range_total(&headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_parse_content_range_total_missing() {
        let headers = HeaderMap::new();
        let result = parse_content_range_total(&headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_extract_filename_from_url() {
        // Simple filename from URL path
        assert_eq!(
            extract_filename_from_url("https://example.com/path/to/file.zip"),
            "file.zip"
        );
    }

    #[test]
    fn test_extract_filename_from_url_with_query() {
        // Filename with query params stripped
        assert_eq!(
            extract_filename_from_url("https://example.com/file.zip?token=abc"),
            "file.zip"
        );
    }

    #[test]
    fn test_extract_filename_from_url_empty_path() {
        // Fallback for empty path
        assert_eq!(
            extract_filename_from_url("https://example.com/"),
            "download"
        );
    }

    /// Helper to test URL-based filename extraction without Response
    fn extract_filename_from_url(url: &str) -> String {
        url.split('/')
            .last()
            .map(|s| s.split('?').next().unwrap_or(s))
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "download".to_string())
    }
}
