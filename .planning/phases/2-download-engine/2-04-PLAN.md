---
phase: 2-download-engine
plan: 04
type: execute
wave: 3
depends_on: [2-02, 2-03]
files_modified:
  - src-tauri/src/download/resume.rs
  - src-tauri/src/download/manager.rs
  - src-tauri/src/download/mod.rs
  - src-tauri/src/lib.rs
  - src-tauri/capabilities/default.json
autonomous: true
user_setup: []

must_haves:
  truths:
    - "Interrupted downloads can resume from where they left off"
    - "Multiple downloads can run in parallel with concurrency limit"
    - "Frontend can start downloads via Tauri commands"
    - "Frontend receives progress events during downloads"
  artifacts:
    - path: "src-tauri/src/download/resume.rs"
      provides: "Resume capability with Range headers"
      exports: ["download_with_resume"]
    - path: "src-tauri/src/download/manager.rs"
      provides: "Download manager with parallel execution"
      exports: ["DownloadManager"]
    - path: "src-tauri/src/lib.rs"
      provides: "Tauri command registration"
      contains: "start_download"
  key_links:
    - from: "src-tauri/src/download/manager.rs"
      to: "tokio::sync::Semaphore"
      via: "Concurrency limiting"
      pattern: "Semaphore::new"
    - from: "src-tauri/src/lib.rs"
      to: "download::manager::DownloadManager"
      via: "Tauri state management"
      pattern: "manage.*DownloadManager"
    - from: "Tauri commands"
      to: "tauri::ipc::Channel"
      via: "Progress event streaming"
      pattern: "Channel<DownloadEvent>"
---

<objective>
Implement resume capability, parallel download manager with Semaphore concurrency, and Tauri commands for frontend integration.

Purpose: This plan completes the download engine by adding resume support, parallel download management, and exposing everything to the frontend via Tauri commands with Channel-based progress reporting.

Output: Complete download system ready for frontend consumption with resume, parallel downloads, and real-time progress.
</objective>

<execution_context>
@./.claude/get-shit-done/workflows/execute-plan.md
@./.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/phases/2-download-engine/2-RESEARCH.md

# From prior plans (will exist when this runs)
@src-tauri/src/download/mod.rs
@src-tauri/src/download/engine.rs
@src-tauri/src/download/progress.rs
@src-tauri/src/download/providers/mod.rs
@src-tauri/src/download/providers/gdrive.rs
@src-tauri/src/download/providers/mediafire.rs
@src-tauri/src/models/download.rs
</context>

<tasks>

<task type="auto">
  <name>Task 1: Implement resume capability with Range headers</name>
  <files>
    src-tauri/src/download/resume.rs
    src-tauri/src/download/mod.rs
  </files>
  <action>
Create src-tauri/src/download/resume.rs with:

1. Resume download function:
   ```rust
   use reqwest::header::{RANGE, CONTENT_RANGE, ACCEPT_RANGES};
   use tokio::fs::OpenOptions;
   use tokio::io::{AsyncSeekExt, AsyncWriteExt};
   use futures_util::StreamExt;
   use std::path::Path;

   use crate::download::{DownloadError, progress::{DownloadEvent, ProgressTracker}};

   /// Download with resume support
   /// If dest_path exists, attempts to resume from where it left off
   pub async fn download_with_resume(
       client: &reqwest::Client,
       url: &str,
       dest_path: &Path,
       download_id: String,
       on_event: tauri::ipc::Channel<DownloadEvent>,
   ) -> Result<(), DownloadError> {
       // Check for existing partial file
       let start_pos = if dest_path.exists() {
           tokio::fs::metadata(dest_path).await
               .map(|m| m.len())
               .unwrap_or(0)
       } else {
           0
       };

       // Build request with Range header if resuming
       let mut request = client.get(url);
       if start_pos > 0 {
           request = request.header(RANGE, format!("bytes={}-", start_pos));
       }

       let response = request.send().await.map_err(DownloadError::RequestError)?;
       let status = response.status();

       // Determine actual start position and total size
       let (actual_start, total_size) = if status == reqwest::StatusCode::PARTIAL_CONTENT {
           // Server accepted Range request (206 response)
           let total = parse_content_range_total(response.headers())?;
           (start_pos, total)
       } else if status == reqwest::StatusCode::OK {
           // Server doesn't support Range, or file changed - start fresh
           let total = response.content_length().unwrap_or(0);
           (0, total)
       } else if status == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
           // File already complete (416 response)
           let total = start_pos;
           on_event.send(DownloadEvent::Completed {
               download_id,
               file_path: dest_path.to_string_lossy().to_string(),
           }).map_err(|e| DownloadError::ChannelError(e.to_string()))?;
           return Ok(());
       } else {
           return Err(DownloadError::HttpError(status));
       };

       // Extract filename from response
       let file_name = extract_filename(&response, url);

       // Send started event
       on_event.send(DownloadEvent::Started {
           download_id: download_id.clone(),
           file_name,
           total_bytes: total_size,
       }).map_err(|e| DownloadError::ChannelError(e.to_string()))?;

       // Open file in appropriate mode
       let mut file = if actual_start > 0 {
           let mut f = OpenOptions::new()
               .write(true)
               .open(dest_path)
               .await
               .map_err(DownloadError::IoError)?;
           f.seek(std::io::SeekFrom::End(0)).await.map_err(DownloadError::IoError)?;
           f
       } else {
           tokio::fs::File::create(dest_path).await.map_err(DownloadError::IoError)?
       };

       // Create progress tracker starting from actual_start
       let mut tracker = ProgressTracker::new(download_id.clone(), total_size);
       tracker.set_downloaded(actual_start);

       // Stream download
       let mut stream = response.bytes_stream();

       while let Some(chunk_result) = stream.next().await {
           let chunk = chunk_result.map_err(DownloadError::RequestError)?;
           file.write_all(&chunk).await.map_err(DownloadError::IoError)?;

           if let Some(event) = tracker.update(chunk.len() as u64) {
               let _ = on_event.send(event); // Ignore throttled sends
           }
       }

       // Ensure final progress is sent
       file.flush().await.map_err(DownloadError::IoError)?;

       on_event.send(DownloadEvent::Completed {
           download_id,
           file_path: dest_path.to_string_lossy().to_string(),
       }).map_err(|e| DownloadError::ChannelError(e.to_string()))?;

       Ok(())
   }

   fn parse_content_range_total(headers: &reqwest::header::HeaderMap) -> Result<u64, DownloadError> {
       // Content-Range: bytes 1000-1999/5000
       if let Some(range) = headers.get(CONTENT_RANGE) {
           let s = range.to_str().map_err(|_| DownloadError::ProviderError("Invalid Content-Range header".to_string()))?;
           if let Some(total) = s.split('/').last() {
               if total != "*" {
                   return total.parse().map_err(|_| DownloadError::ProviderError("Invalid Content-Range total".to_string()));
               }
           }
       }
       // If no Content-Range or unknown total, return 0
       Ok(0)
   }

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
       url.split('/').last()
           .map(|s| s.split('?').next().unwrap_or(s))
           .filter(|s| !s.is_empty())
           .map(|s| s.to_string())
           .unwrap_or_else(|| "download".to_string())
   }
   ```

2. Add set_downloaded method to ProgressTracker in progress.rs:
   ```rust
   impl ProgressTracker {
       pub fn set_downloaded(&mut self, bytes: u64) {
           self.downloaded_bytes = bytes;
       }
   }
   ```

3. Update download/mod.rs to export resume module:
   - Add: pub mod resume;
   - Add re-export: pub use resume::download_with_resume;

4. Add unit tests:
   - Test parse_content_range_total with valid header
   - Test extract_filename from Content-Disposition
   - Test extract_filename from URL fallback
  </action>
  <verify>
    cargo check --manifest-path src-tauri/Cargo.toml
    cargo test --manifest-path src-tauri/Cargo.toml resume
    All tests pass.
  </verify>
  <done>
    - download_with_resume checks for existing file and sends Range header
    - 206 Partial Content response handled correctly
    - 200 OK response restarts download from scratch
    - 416 Range Not Satisfiable treated as already complete
    - Progress tracker can be initialized with existing bytes
  </done>
</task>

<task type="auto">
  <name>Task 2: Implement DownloadManager with parallel execution</name>
  <files>
    src-tauri/src/download/manager.rs
    src-tauri/src/download/mod.rs
  </files>
  <action>
Create src-tauri/src/download/manager.rs with:

1. DownloadManager struct:
   ```rust
   use std::sync::Arc;
   use std::path::PathBuf;
   use tokio::sync::Semaphore;
   use tauri::ipc::Channel;

   use crate::download::{
       DownloadError,
       progress::DownloadEvent,
       resume::download_with_resume,
       providers::{DownloadProvider, GoogleDriveProvider, MediafireProvider, DirectDownloadInfo},
   };
   use crate::models::DownloadProvider as ProviderType;

   const MAX_CONCURRENT_DOWNLOADS: usize = 3;

   pub struct DownloadManager {
       client: reqwest::Client,
       semaphore: Arc<Semaphore>,
   }

   impl Default for DownloadManager {
       fn default() -> Self {
           Self::new()
       }
   }

   impl DownloadManager {
       pub fn new() -> Self {
           let client = reqwest::Client::builder()
               .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
               .redirect(reqwest::redirect::Policy::limited(10))
               .build()
               .expect("Failed to create HTTP client");

           Self {
               client,
               semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_DOWNLOADS)),
           }
       }

       /// Start a download, acquiring semaphore permit for concurrency limiting
       pub async fn download(
           &self,
           share_url: String,
           provider_type: ProviderType,
           dest_dir: PathBuf,
           download_id: String,
           on_event: Channel<DownloadEvent>,
       ) -> Result<String, DownloadError> {
           // Acquire semaphore permit (blocks if MAX_CONCURRENT reached)
           let _permit = self.semaphore.clone().acquire_owned().await
               .map_err(|e| DownloadError::ProviderError(format!("Semaphore error: {}", e)))?;

           // Resolve direct URL based on provider
           let info = self.resolve_url(&share_url, provider_type).await?;

           // Determine destination path
           let file_name = info.file_name.unwrap_or_else(|| {
               share_url.split('/').last()
                   .map(|s| s.split('?').next().unwrap_or(s))
                   .filter(|s| !s.is_empty())
                   .map(|s| s.to_string())
                   .unwrap_or_else(|| format!("{}.download", download_id))
           });
           let dest_path = dest_dir.join(&file_name);

           // Perform download with resume support
           download_with_resume(
               &self.client,
               &info.url,
               &dest_path,
               download_id,
               on_event,
           ).await?;

           Ok(dest_path.to_string_lossy().to_string())
       }

       /// Resolve share URL to direct download URL
       async fn resolve_url(&self, share_url: &str, provider_type: ProviderType) -> Result<DirectDownloadInfo, DownloadError> {
           match provider_type {
               ProviderType::GoogleDrive => {
                   let provider = GoogleDriveProvider::new(self.client.clone());
                   provider.resolve_direct_url(share_url).await
               }
               ProviderType::Mediafire => {
                   let provider = MediafireProvider::new(self.client.clone());
                   provider.resolve_direct_url(share_url).await
               }
               ProviderType::Unknown => {
                   Err(DownloadError::ProviderError("Unknown download provider".to_string()))
               }
           }
       }

       /// Get current number of active downloads
       pub fn active_downloads(&self) -> usize {
           MAX_CONCURRENT_DOWNLOADS - self.semaphore.available_permits()
       }
   }
   ```

2. Update download/mod.rs:
   - Add: pub mod manager;
   - Add re-export: pub use manager::DownloadManager;

3. Add basic tests:
   - Test DownloadManager::new() creates successfully
   - Test active_downloads starts at 0
  </action>
  <verify>
    cargo check --manifest-path src-tauri/Cargo.toml
    cargo test --manifest-path src-tauri/Cargo.toml manager
    All tests pass.
  </verify>
  <done>
    - DownloadManager created with shared reqwest::Client
    - Semaphore limits concurrent downloads to 3
    - resolve_url dispatches to correct provider based on ProviderType
    - download method acquires permit, resolves URL, downloads with resume
    - active_downloads reports current count
  </done>
</task>

<task type="auto">
  <name>Task 3: Add Tauri commands and HTTP permissions</name>
  <files>
    src-tauri/src/lib.rs
    src-tauri/capabilities/default.json
  </files>
  <action>
1. Add Tauri commands to src-tauri/src/lib.rs:
   ```rust
   use tauri::{Manager, State, ipc::Channel};
   use std::path::PathBuf;

   use crate::download::{DownloadManager, progress::DownloadEvent, DownloadError};
   use crate::models::DownloadProvider;

   /// Start a download for a patch module
   #[tauri::command]
   pub async fn start_download(
       manager: State<'_, DownloadManager>,
       share_url: String,
       provider: String,
       dest_dir: String,
       on_progress: Channel<DownloadEvent>,
   ) -> Result<String, String> {
       let download_id = uuid::Uuid::new_v4().to_string();

       let provider_type = match provider.to_lowercase().as_str() {
           "googledrive" | "google_drive" | "gdrive" => DownloadProvider::GoogleDrive,
           "mediafire" => DownloadProvider::Mediafire,
           _ => DownloadProvider::Unknown,
       };

       let dest_path = PathBuf::from(dest_dir);

       // Clone values for spawned task
       let manager_clone = manager.inner().clone();
       let download_id_clone = download_id.clone();

       tauri::async_runtime::spawn(async move {
           let result = manager_clone.download(
               share_url,
               provider_type,
               dest_path,
               download_id_clone.clone(),
               on_progress.clone(),
           ).await;

           if let Err(e) = result {
               let _ = on_progress.send(DownloadEvent::Failed {
                   download_id: download_id_clone,
                   error: e.to_string(),
               });
           }
       });

       Ok(download_id)
   }

   /// Get current active download count
   #[tauri::command]
   pub fn get_active_downloads(manager: State<'_, DownloadManager>) -> usize {
       manager.active_downloads()
   }
   ```

2. Make DownloadManager cloneable by wrapping internal state:
   Update manager.rs to use Arc internally:
   ```rust
   #[derive(Clone)]
   pub struct DownloadManager {
       client: reqwest::Client,
       semaphore: Arc<Semaphore>,
   }
   ```
   (reqwest::Client is already Clone)

3. Register DownloadManager as Tauri state in lib.rs run() function:
   ```rust
   pub fn run() {
       tauri::Builder::default()
           .plugin(tauri_plugin_opener::init())
           .plugin(tauri_plugin_http::init())
           .manage(DownloadManager::new())
           .invoke_handler(tauri::generate_handler![
               // Existing commands
               fetch_patches,
               validate_selection,
               auto_select_deps,
               get_forum_url,
               // New download commands
               start_download,
               get_active_downloads,
           ])
           .run(tauri::generate_context!())
           .expect("error while running tauri application");
   }
   ```

4. Update src-tauri/capabilities/default.json to add HTTP permissions for download hosts:
   ```json
   {
     "$schema": "../gen/schemas/desktop-schema.json",
     "identifier": "default",
     "description": "Capability for the main window",
     "windows": ["main"],
     "permissions": [
       "core:default",
       "opener:default",
       {
         "identifier": "http:default",
         "allow": [
           { "url": "https://forum.turtlecraft.gg/*" },
           { "url": "https://forum.turtle-wow.org/*" },
           { "url": "https://drive.google.com/*" },
           { "url": "https://*.googleusercontent.com/*" },
           { "url": "https://www.mediafire.com/*" },
           { "url": "https://mediafire.com/*" },
           { "url": "https://download*.mediafire.com/*" }
         ]
       }
     ]
   }
   ```

5. Add DownloadError Display implementation if not already present:
   ```rust
   impl std::fmt::Display for DownloadError {
       fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
           match self {
               Self::HttpError(status) => write!(f, "HTTP error: {}", status),
               Self::IoError(e) => write!(f, "IO error: {}", e),
               Self::RequestError(e) => write!(f, "Request error: {}", e),
               Self::ProviderError(msg) => write!(f, "Provider error: {}", msg),
               Self::ChannelError(msg) => write!(f, "Channel error: {}", msg),
               Self::ConfirmationFailed => write!(f, "Failed to get download confirmation"),
               Self::DirectUrlNotFound => write!(f, "Could not find direct download URL"),
           }
       }
   }
   ```
  </action>
  <verify>
    cargo build --manifest-path src-tauri/Cargo.toml
    Verify build succeeds with Tauri commands registered.
    Verify capabilities JSON is valid.
  </verify>
  <done>
    - start_download Tauri command accepts share_url, provider, dest_dir, and Channel
    - get_active_downloads returns current count
    - DownloadManager registered as Tauri state
    - HTTP permissions added for Google Drive and Mediafire domains
    - Commands registered in invoke_handler
  </done>
</task>

</tasks>

<verification>
After all tasks complete:

1. Full build succeeds:
   ```bash
   cargo build --manifest-path src-tauri/Cargo.toml
   ```

2. All tests pass:
   ```bash
   cargo test --manifest-path src-tauri/Cargo.toml
   ```

3. Tauri commands are registered (check lib.rs invoke_handler)

4. HTTP capabilities include download host domains

5. Module structure complete:
   ```
   src-tauri/src/download/
   ├── mod.rs           (exports)
   ├── engine.rs        (basic download)
   ├── progress.rs      (DownloadEvent, ProgressTracker)
   ├── resume.rs        (download_with_resume)
   ├── manager.rs       (DownloadManager with Semaphore)
   └── providers/
       ├── mod.rs       (DownloadProvider trait)
       ├── gdrive.rs    (GoogleDriveProvider)
       └── mediafire.rs (MediafireProvider)
   ```
</verification>

<success_criteria>
- [ ] download_with_resume sends Range header for existing files
- [ ] 206/200/416 status codes handled correctly
- [ ] DownloadManager limits concurrent downloads with Semaphore
- [ ] DownloadManager dispatches to correct provider
- [ ] start_download Tauri command spawns download task
- [ ] get_active_downloads Tauri command returns count
- [ ] DownloadManager registered as Tauri state
- [ ] HTTP permissions allow Google Drive and Mediafire domains
- [ ] Full cargo build succeeds
- [ ] All tests pass (20+ across all modules)
</success_criteria>

<output>
After completion, create `.planning/phases/2-download-engine/2-04-SUMMARY.md`
</output>
