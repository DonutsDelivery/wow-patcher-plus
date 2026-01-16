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
    client: &reqwest::Client,
    url: &str,
    dest_path: &Path,
    download_id: String,
    on_event: Channel<DownloadEvent>,
) -> Result<(), DownloadError> {
    // Send GET request
    let response = client.get(url).send().await?;

    // Check for HTTP error status
    let status = response.status();
    if !status.is_success() {
        let error_msg = format!("HTTP {} - {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown"));
        let _ = on_event.send(DownloadEvent::Failed {
            download_id: download_id.clone(),
            error: error_msg.clone(),
        });
        return Err(DownloadError::HttpError(status));
    }

    // Extract content length and filename
    let total_bytes = response.content_length().unwrap_or(0);
    let file_name = extract_filename(&response, url);

    // Create progress tracker
    let mut progress_tracker = ProgressTracker::new(download_id.clone(), total_bytes);

    // Send Started event
    let started_event = progress_tracker.started_event(file_name);
    if on_event.send(started_event).is_err() {
        return Err(DownloadError::ChannelError("Failed to send Started event".to_string()));
    }

    // Create parent directories if they don't exist
    if let Some(parent) = dest_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Create destination file
    let mut file = match tokio::fs::File::create(dest_path).await {
        Ok(f) => f,
        Err(e) => {
            let _ = on_event.send(progress_tracker.failed_event(e.to_string()));
            return Err(DownloadError::IoError(e));
        }
    };

    // Stream the response body
    let mut stream = response.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                // Write chunk to file
                if let Err(e) = file.write_all(&chunk).await {
                    let _ = on_event.send(progress_tracker.failed_event(e.to_string()));
                    return Err(DownloadError::IoError(e));
                }

                // Update progress and send event if throttle allows
                if let Some(progress_event) = progress_tracker.update(chunk.len() as u64) {
                    let _ = on_event.send(progress_event);
                }
            }
            Err(e) => {
                let _ = on_event.send(progress_tracker.failed_event(e.to_string()));
                return Err(DownloadError::RequestError(e));
            }
        }
    }

    // Ensure all data is flushed to disk
    if let Err(e) = file.flush().await {
        let _ = on_event.send(progress_tracker.failed_event(e.to_string()));
        return Err(DownloadError::IoError(e));
    }

    // Send Completed event
    let completed_event = progress_tracker.completed_event(dest_path.to_string_lossy().to_string());
    let _ = on_event.send(completed_event);

    Ok(())
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
                    // URL decode the filename
                    return urlencoding_decode(last);
                }
            }
        }
    }

    // Default fallback
    "download".to_string()
}

/// Simple URL decoding for filenames
fn urlencoding_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            // Try to decode percent-encoded sequence
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                    continue;
                }
            }
            // If decoding failed, keep the original
            result.push('%');
            result.push_str(&hex);
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urlencoding_decode() {
        assert_eq!(urlencoding_decode("hello%20world"), "hello world");
        assert_eq!(urlencoding_decode("file+name.txt"), "file name.txt");
        assert_eq!(urlencoding_decode("simple.txt"), "simple.txt");
        assert_eq!(urlencoding_decode("%2Fpath%2Fto%2Ffile"), "/path/to/file");
    }

    #[test]
    fn test_extract_filename_from_url() {
        // URL-based filename extraction (no response headers to test here)
        let url = "https://example.com/path/to/myfile.zip";
        let parsed = url::Url::parse(url).unwrap();
        let segments: Vec<_> = parsed.path_segments().unwrap().collect();
        assert_eq!(segments.last().unwrap(), &"myfile.zip");
    }

    #[test]
    fn test_extract_filename_encoded_url() {
        let encoded = "my%20file%20name.zip";
        assert_eq!(urlencoding_decode(encoded), "my file name.zip");
    }
}
