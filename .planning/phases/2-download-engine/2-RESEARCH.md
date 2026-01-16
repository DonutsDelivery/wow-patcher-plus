# Phase 2: Download Engine - Research

**Researched:** 2026-01-16
**Domain:** File downloading, HTTP streaming, progress tracking, Rust async
**Confidence:** MEDIUM (Google Drive/Mediafire patterns based on community tools and WebSearch; Tauri/reqwest from official docs)

## Summary

This research investigates how to download patch files from Google Drive and Mediafire with progress tracking, resume capability, and parallel download support. The download engine must handle two distinct hosting providers with different URL resolution strategies.

Google Drive requires extracting file IDs from share URLs and constructing direct download URLs, with special handling for large files (>100MB) that trigger virus scan warnings. Mediafire requires fetching the share page and extracting the actual download URL from the HTML, as direct links use dynamic numbered subdomains.

The standard approach uses reqwest's streaming API (`bytes_stream()`) for memory-efficient downloads with progress tracking, Tauri Channels for real-time progress reporting to the frontend, and tokio Semaphore for parallel download concurrency control.

**Primary recommendation:** Use reqwest with `stream` feature for downloads, Tauri Channel API for progress events (not regular events), and implement provider-specific URL resolvers that convert share URLs to direct download URLs before streaming.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| reqwest | 0.12.x | HTTP client with streaming | Already in project via tauri-plugin-http |
| tokio | 1.x | Async runtime | Already in project, provides Semaphore for concurrency |
| futures-util | 0.3.x | Stream utilities (StreamExt) | Required for `bytes_stream().next()` iteration |
| tauri (Channel) | 2.x | Progress events to frontend | Designed for streaming data, faster than events |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| scraper | 0.25.x | HTML parsing | Already in project, for Mediafire page parsing |
| tokio (fs) | 1.x | Async file I/O | Writing downloaded chunks to disk |
| uuid | 1.x | Download IDs | Generate unique identifiers for tracking |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Tauri Channel | Tauri Events | Events not designed for streaming data, higher latency |
| reqwest stream | reqwest bytes() | bytes() loads entire file to memory, unusable for large patches |
| tokio Semaphore | buffer_unordered | Semaphore provides finer control, works across spawn boundaries |

**Installation (add to existing Cargo.toml):**
```toml
[dependencies]
# Already present: tauri, tokio, scraper, reqwest (via tauri-plugin-http)
futures-util = "0.3"
uuid = { version = "1", features = ["v4"] }
```

**Note:** Enable `stream` feature on reqwest if using it directly (tauri-plugin-http may need verification).

## Architecture Patterns

### Recommended Module Structure
```
src-tauri/
├── src/
│   ├── download/
│   │   ├── mod.rs           # Module exports, DownloadManager
│   │   ├── engine.rs        # Core download logic with streaming
│   │   ├── providers/
│   │   │   ├── mod.rs       # Provider trait definition
│   │   │   ├── gdrive.rs    # Google Drive URL resolution
│   │   │   └── mediafire.rs # Mediafire URL resolution
│   │   ├── progress.rs      # Progress tracking, speed calculation
│   │   └── resume.rs        # Resume capability with Range headers
│   ├── models/
│   │   └── download.rs      # Already exists, extend with progress types
```

### Pattern 1: Provider Trait for URL Resolution
**What:** Abstract the difference between download providers
**When to use:** Always - providers have fundamentally different URL resolution
**Example:**
```rust
// Source: Derived from provider analysis
use async_trait::async_trait;

#[async_trait]
pub trait DownloadProvider {
    /// Resolve a share URL to a direct download URL
    async fn resolve_direct_url(&self, share_url: &str) -> Result<DirectDownloadInfo, DownloadError>;

    /// Check if this provider supports resume (Range headers)
    fn supports_resume(&self) -> bool;
}

pub struct DirectDownloadInfo {
    pub url: String,
    pub file_name: String,
    pub content_length: Option<u64>,
    pub supports_range: bool,
}
```

### Pattern 2: Streaming Download with Channel Progress
**What:** Stream bytes from response, report progress via Tauri Channel
**When to use:** All downloads - memory efficient and responsive UI
**Example:**
```rust
// Source: https://v2.tauri.app/develop/calling-frontend/ (Channel section)
// Source: https://gist.github.com/Tapanhaz/096e299bf060607b572d700e89a62529
use tauri::ipc::Channel;
use futures_util::StreamExt;
use std::cmp::min;

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum DownloadEvent {
    Started { download_id: String, file_name: String, total_bytes: u64 },
    Progress { download_id: String, downloaded_bytes: u64, total_bytes: u64, speed_bps: u64 },
    Completed { download_id: String, file_path: String },
    Failed { download_id: String, error: String },
}

pub async fn download_file(
    client: &reqwest::Client,
    url: &str,
    dest_path: &std::path::Path,
    download_id: String,
    on_event: Channel<DownloadEvent>,
) -> Result<(), DownloadError> {
    let response = client.get(url).send().await?;
    let total_bytes = response.content_length().unwrap_or(0);

    on_event.send(DownloadEvent::Started {
        download_id: download_id.clone(),
        file_name: dest_path.file_name().unwrap().to_string_lossy().to_string(),
        total_bytes,
    })?;

    let mut file = tokio::fs::File::create(dest_path).await?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    let start_time = std::time::Instant::now();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
        downloaded += chunk.len() as u64;

        let elapsed = start_time.elapsed().as_secs_f64();
        let speed_bps = if elapsed > 0.0 { (downloaded as f64 / elapsed) as u64 } else { 0 };

        on_event.send(DownloadEvent::Progress {
            download_id: download_id.clone(),
            downloaded_bytes: downloaded,
            total_bytes,
            speed_bps,
        })?;
    }

    on_event.send(DownloadEvent::Completed {
        download_id,
        file_path: dest_path.to_string_lossy().to_string(),
    })?;

    Ok(())
}
```

### Pattern 3: Parallel Downloads with Semaphore
**What:** Limit concurrent downloads using tokio Semaphore
**When to use:** When downloading multiple patches simultaneously
**Example:**
```rust
// Source: https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html
use std::sync::Arc;
use tokio::sync::Semaphore;

const MAX_CONCURRENT_DOWNLOADS: usize = 3;

pub struct DownloadManager {
    client: reqwest::Client,
    semaphore: Arc<Semaphore>,
}

impl DownloadManager {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_DOWNLOADS)),
        }
    }

    pub async fn download(&self, url: String, path: String) -> Result<(), DownloadError> {
        // acquire_owned() allows permit to move into spawned task
        let permit = self.semaphore.clone().acquire_owned().await?;

        // Download logic here...

        drop(permit); // Release permit when done
        Ok(())
    }
}
```

### Anti-Patterns to Avoid
- **Using `response.bytes()` for large files:** Loads entire file into memory, will fail for 500MB+ patches
- **Using Tauri Events for progress:** Events are not designed for high-frequency streaming data
- **Spawning with `tokio::spawn` in event handlers:** Can panic in Tauri v2, use `tauri::async_runtime::spawn`
- **Chunked batch downloads:** Wastes time waiting for slowest download in batch, use Semaphore instead
- **Hardcoding download URLs:** Google Drive and Mediafire require dynamic URL resolution

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Download streaming | Manual byte reading | reqwest `bytes_stream()` | Handles chunking, backpressure, errors |
| Concurrency limiting | Thread pool | tokio Semaphore | Async-native, fair ordering |
| Progress events | Custom IPC | Tauri Channel | Optimized for streaming, type-safe |
| Speed calculation | Manual timing | Track bytes + elapsed time | Simple math, but use Instant not SystemTime |
| Resume detection | File stat syscalls | `std::fs::metadata()` | Cross-platform, handles missing files |

**Key insight:** The tricky parts are provider-specific URL resolution, not the download mechanics. reqwest and tokio handle HTTP complexity; focus effort on Google Drive/Mediafire quirks.

## Common Pitfalls

### Pitfall 1: Google Drive Large File Virus Scan Warning
**What goes wrong:** Downloads >100MB return an HTML warning page instead of the file
**Why it happens:** Google cannot virus-scan large files, shows confirmation dialog
**How to avoid:**
- After first request, check if response is HTML (not binary)
- Look for `download_warning_` cookie or `confirm=` parameter in response
- Make second request with extracted confirmation token
- Pattern: `https://drive.google.com/uc?export=download&confirm=TOKEN&id=FILE_ID`
**Warning signs:** Downloaded file is ~50KB HTML instead of expected size

### Pitfall 2: Mediafire Dynamic Download Domains
**What goes wrong:** Direct links to mediafire.com/file/... don't work
**Why it happens:** Mediafire uses numbered subdomains (download1, download2, etc.) for actual files
**How to avoid:**
- Fetch the share page HTML first
- Extract direct URL using regex: `https://download[0-9]+\.mediafire\.com/[^'"]+`
- Use extracted URL for actual download
**Warning signs:** 404 errors or redirects to ad pages

### Pitfall 3: Resume Broken by Changed File
**What goes wrong:** Resumed download produces corrupted file
**Why it happens:** Server file changed since partial download, but resume continues
**How to avoid:**
- Store ETag or Last-Modified from initial request
- Use `If-Range` header when resuming
- If server returns 200 (not 206), restart download from scratch
**Warning signs:** Final file hash doesn't match expected

### Pitfall 4: Progress Event Flooding
**What goes wrong:** UI becomes laggy, events back up
**Why it happens:** Sending progress event for every chunk (could be thousands/second)
**How to avoid:**
- Throttle progress updates (e.g., every 100ms or every 1% change)
- Use Tauri Channel (designed for this) not Events
- Consider sending only significant changes
**Warning signs:** UI freezes during download, memory usage climbs

### Pitfall 5: Tauri Async Runtime Mismatch
**What goes wrong:** Panic with "no reactor running" error
**Why it happens:** Using `tokio::spawn` instead of `tauri::async_runtime::spawn`
**How to avoid:**
- Always use `tauri::async_runtime::spawn` for Tauri commands
- Clone AppHandle before moving into async closures
- For long-running tasks, spawn in `setup()` closure
**Warning signs:** Panic on Windows, works on other platforms

## Code Examples

### Google Drive URL Resolution
```rust
// Source: https://getlate.dev/blog/google-drive-direct-download-urls-complete-guide
// Source: https://gist.github.com/yasirkula/d0ec0c07b138748e5feaecbd93b6223c

use regex::Regex;
use scraper::{Html, Selector};

pub struct GoogleDriveProvider {
    client: reqwest::Client,
}

impl GoogleDriveProvider {
    /// Convert share URL to direct download URL
    pub fn get_direct_url(file_id: &str) -> String {
        format!("https://drive.google.com/uc?export=download&id={}", file_id)
    }

    /// Handle large file confirmation (>100MB)
    pub async fn resolve_with_confirmation(&self, file_id: &str) -> Result<String, DownloadError> {
        let initial_url = Self::get_direct_url(file_id);
        let response = self.client.get(&initial_url).send().await?;

        // Check if we got the virus scan warning page
        let content_type = response.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if content_type.contains("text/html") {
            // Parse HTML to find confirmation link
            let html = response.text().await?;
            let document = Html::parse_document(&html);

            // Look for the "Download anyway" link with confirm parameter
            let link_selector = Selector::parse("a[href*='confirm=']").unwrap();
            if let Some(element) = document.select(&link_selector).next() {
                if let Some(href) = element.value().attr("href") {
                    // href might be relative, make it absolute
                    if href.starts_with("/") {
                        return Ok(format!("https://drive.google.com{}", href));
                    }
                    return Ok(href.to_string());
                }
            }

            // Alternative: extract from form action or JavaScript
            let confirm_regex = Regex::new(r"confirm=([0-9A-Za-z_-]+)").unwrap();
            if let Some(cap) = confirm_regex.captures(&html) {
                return Ok(format!(
                    "https://drive.google.com/uc?export=download&confirm={}&id={}",
                    &cap[1], file_id
                ));
            }

            return Err(DownloadError::ConfirmationFailed);
        }

        // No confirmation needed, original URL works
        Ok(initial_url)
    }
}
```

### Mediafire URL Resolution
```rust
// Source: https://raw.githubusercontent.com/Andrew-J-Larson-Alt/mediafire-direct/main/!-bash-script/mediafire-direct.sh

use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    // Matches the actual download URL from Mediafire page
    static ref MEDIAFIRE_DOWNLOAD_URL: Regex = Regex::new(
        r#"https://download[0-9]+\.mediafire\.com/[^'"<>\s]+"#
    ).unwrap();

    // Matches pre-download URL with dkey parameter
    static ref MEDIAFIRE_DKEY_URL: Regex = Regex::new(
        r#"https?://(?:www\.)?mediafire\.com/(?:file|view|download)/[^'"\?]+\?dkey=[^'"<>\s]+"#
    ).unwrap();
}

pub struct MediafireProvider {
    client: reqwest::Client,
}

impl MediafireProvider {
    pub async fn resolve_direct_url(&self, share_url: &str) -> Result<String, DownloadError> {
        // Step 1: Fetch the share page
        let response = self.client
            .get(share_url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .send()
            .await?;

        let html = response.text().await?;

        // Step 2: Try to find direct download URL
        if let Some(cap) = MEDIAFIRE_DOWNLOAD_URL.find(&html) {
            return Ok(cap.as_str().to_string());
        }

        // Step 3: If not found, look for dkey URL and follow it
        if let Some(cap) = MEDIAFIRE_DKEY_URL.find(&html) {
            let dkey_url = cap.as_str();

            // Small delay to avoid rate limiting (1.5s like the bash script)
            tokio::time::sleep(std::time::Duration::from_millis(1500)).await;

            let response2 = self.client
                .get(dkey_url)
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .send()
                .await?;

            let html2 = response2.text().await?;

            if let Some(cap2) = MEDIAFIRE_DOWNLOAD_URL.find(&html2) {
                return Ok(cap2.as_str().to_string());
            }
        }

        Err(DownloadError::DirectUrlNotFound)
    }
}
```

### Resume Download with Range Headers
```rust
// Source: https://rust-lang-nursery.github.io/rust-cookbook/web/clients/download.html

use reqwest::header::{RANGE, CONTENT_LENGTH, ACCEPT_RANGES, IF_RANGE};
use tokio::fs::OpenOptions;
use tokio::io::AsyncSeekExt;

pub async fn download_with_resume(
    client: &reqwest::Client,
    url: &str,
    dest_path: &std::path::Path,
    on_progress: impl Fn(u64, u64),
) -> Result<(), DownloadError> {
    // Check for existing partial file
    let start_pos = if dest_path.exists() {
        tokio::fs::metadata(dest_path).await?.len()
    } else {
        0
    };

    // Build request with Range header if resuming
    let mut request = client.get(url);
    if start_pos > 0 {
        request = request.header(RANGE, format!("bytes={}-", start_pos));
    }

    let response = request.send().await?;
    let status = response.status();

    // Check response status
    let (actual_start, total_size) = if status == reqwest::StatusCode::PARTIAL_CONTENT {
        // Server accepted Range request
        let total = parse_content_range_total(response.headers())?;
        (start_pos, total)
    } else if status == reqwest::StatusCode::OK {
        // Server doesn't support Range, or file changed - start fresh
        let total = response.content_length().unwrap_or(0);
        (0, total)
    } else {
        return Err(DownloadError::HttpError(status));
    };

    // Open file in appropriate mode
    let mut file = if actual_start > 0 {
        let mut f = OpenOptions::new().append(true).open(dest_path).await?;
        f.seek(std::io::SeekFrom::End(0)).await?;
        f
    } else {
        tokio::fs::File::create(dest_path).await?
    };

    // Stream download
    let mut downloaded = actual_start;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
        downloaded += chunk.len() as u64;
        on_progress(downloaded, total_size);
    }

    Ok(())
}

fn parse_content_range_total(headers: &reqwest::header::HeaderMap) -> Result<u64, DownloadError> {
    // Content-Range: bytes 0-999/1234
    if let Some(range) = headers.get("content-range") {
        let s = range.to_str().map_err(|_| DownloadError::InvalidHeader)?;
        if let Some(total) = s.split('/').last() {
            return total.parse().map_err(|_| DownloadError::InvalidHeader);
        }
    }
    Err(DownloadError::InvalidHeader)
}
```

### Tauri Command with Channel Progress
```rust
// Source: https://v2.tauri.app/develop/calling-frontend/

use tauri::{AppHandle, ipc::Channel};

#[tauri::command]
pub async fn start_download(
    app: AppHandle,
    module_id: String,
    download_url: String,
    dest_dir: String,
    on_progress: Channel<DownloadEvent>,
) -> Result<String, String> {
    let download_id = uuid::Uuid::new_v4().to_string();

    // Clone what we need for the spawned task
    let download_id_clone = download_id.clone();

    tauri::async_runtime::spawn(async move {
        let result = perform_download(
            &download_url,
            &dest_dir,
            &download_id_clone,
            on_progress,
        ).await;

        if let Err(e) = result {
            // Error already sent via channel
            log::error!("Download failed: {}", e);
        }
    });

    Ok(download_id)
}
```

### Frontend TypeScript
```typescript
// Source: https://v2.tauri.app/develop/calling-frontend/
import { invoke, Channel } from '@tauri-apps/api/core';

type DownloadEvent =
  | { event: 'started'; data: { downloadId: string; fileName: string; totalBytes: number } }
  | { event: 'progress'; data: { downloadId: string; downloadedBytes: number; totalBytes: number; speedBps: number } }
  | { event: 'completed'; data: { downloadId: string; filePath: string } }
  | { event: 'failed'; data: { downloadId: string; error: string } };

export async function downloadPatch(
  moduleId: string,
  url: string,
  destDir: string,
  onProgress: (event: DownloadEvent) => void
): Promise<string> {
  const channel = new Channel<DownloadEvent>();
  channel.onmessage = onProgress;

  return await invoke<string>('start_download', {
    moduleId,
    downloadUrl: url,
    destDir,
    onProgress: channel,
  });
}

// Usage in React component
function DownloadButton({ patch }: { patch: Patch }) {
  const [progress, setProgress] = useState(0);
  const [speed, setSpeed] = useState(0);

  const handleDownload = async () => {
    await downloadPatch(patch.id, patch.url, '/path/to/dest', (event) => {
      if (event.event === 'progress') {
        setProgress((event.data.downloadedBytes / event.data.totalBytes) * 100);
        setSpeed(event.data.speedBps);
      }
    });
  };

  return <button onClick={handleDownload}>Download ({progress.toFixed(1)}%)</button>;
}
```

## Provider Comparison

| Feature | Google Drive | Mediafire |
|---------|--------------|-----------|
| Direct URL | Convert with file ID | Scrape from page |
| Large file handling | Virus scan confirmation | None needed |
| Range header support | YES | Likely YES (needs testing) |
| Rate limiting | Quota per API key | Unknown, use delays |
| Auth required | No (public files) | No |
| URL stability | File ID permanent | Dynamic download domain |

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| reqwest blocking | reqwest async + stream | 2020+ | Non-blocking downloads |
| Tauri Events | Tauri Channels | Tauri v2 | Better streaming performance |
| Manual concurrency | tokio Semaphore | tokio 1.0 | Fair, bounded parallelism |
| Full file in memory | Chunk streaming | Always best practice | Handles any file size |

**Deprecated/outdated:**
- `reqwest::blocking` - Don't use in Tauri async commands
- `app.emit()` for progress - Use Channel instead
- `tokio::spawn` in Tauri v2 commands - Use `tauri::async_runtime::spawn`

## Open Questions

1. **Mediafire Range header support**
   - What we know: HTTP spec, most servers support it
   - What's unclear: Mediafire dynamic domains might not support Range
   - Recommendation: Test with actual Mediafire download, fall back to restart if 200

2. **Google Drive quota limits**
   - What we know: Exists for API access
   - What's unclear: Limits for direct download URLs without API key
   - Recommendation: Implement retry with exponential backoff, warn user if persistent

3. **Optimal progress update frequency**
   - What we know: Every chunk is too frequent, causes lag
   - What's unclear: Best balance for responsive UI
   - Recommendation: Start with 100ms minimum interval, tune based on testing

4. **File integrity verification**
   - What we know: MD5/SHA hashes may be in forum post
   - What's unclear: How consistently hashes are provided
   - Recommendation: Implement hash verification if available, skip if not

## Sources

### Primary (HIGH confidence)
- [Tauri v2 Calling Frontend](https://v2.tauri.app/develop/calling-frontend/) - Channel API for progress
- [reqwest Response docs](https://docs.rs/reqwest/latest/reqwest/struct.Response.html) - bytes_stream()
- [tokio Semaphore](https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html) - Concurrency limiting
- [Rust Cookbook Downloads](https://rust-lang-nursery.github.io/rust-cookbook/web/clients/download.html) - Range headers

### Secondary (MEDIUM confidence)
- [Google Drive Direct Download Guide](https://getlate.dev/blog/google-drive-direct-download-urls-complete-guide) - URL format
- [mediafire-direct bash script](https://github.com/Andrew-J-Larson-Alt/mediafire-direct) - Regex patterns
- [reqwest progress gist](https://gist.github.com/Tapanhaz/096e299bf060607b572d700e89a62529) - Streaming pattern
- [Tauri v2 async tasks blog](https://sneakycrow.dev/blog/2024-05-12-running-async-tasks-in-tauri-v2) - Spawn pattern

### Tertiary (LOW confidence - needs validation)
- Google Drive large file confirmation token extraction - Based on community patterns, may need updates
- Mediafire exact regex patterns - Mediafire may change HTML structure
- Range header support on both providers - Needs testing with actual files

## Metadata

**Confidence breakdown:**
- Download streaming: HIGH - Official reqwest/tokio docs
- Tauri integration: HIGH - Official Tauri v2 docs
- Google Drive resolution: MEDIUM - Community patterns, may change
- Mediafire resolution: MEDIUM - Bash script analysis, HTML may change
- Resume support: MEDIUM - HTTP spec known, provider support needs testing

**Research date:** 2026-01-16
**Valid until:** 2026-02-01 (14 days - provider HTML structures may change)

**Key limitation:** Google Drive and Mediafire URL resolution patterns are based on community tools and may break if providers change their HTML structure. Implement robust error handling and fallback to user notification if resolution fails.
