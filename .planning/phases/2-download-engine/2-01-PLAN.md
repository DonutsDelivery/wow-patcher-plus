---
phase: 2-download-engine
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src-tauri/Cargo.toml
  - src-tauri/src/download/mod.rs
  - src-tauri/src/download/engine.rs
  - src-tauri/src/download/progress.rs
  - src-tauri/src/download/providers/mod.rs
autonomous: true
user_setup: []

must_haves:
  truths:
    - "Download module exists with proper structure"
    - "Provider trait defines URL resolution contract"
    - "Download engine streams bytes without loading full file to memory"
    - "Progress events can be sent during download"
  artifacts:
    - path: "src-tauri/src/download/mod.rs"
      provides: "Module exports and DownloadManager struct"
      exports: ["DownloadManager", "DownloadEvent", "DownloadError"]
    - path: "src-tauri/src/download/engine.rs"
      provides: "Core streaming download logic"
      exports: ["download_file"]
    - path: "src-tauri/src/download/progress.rs"
      provides: "Progress tracking types and speed calculation"
      exports: ["DownloadEvent", "ProgressTracker"]
    - path: "src-tauri/src/download/providers/mod.rs"
      provides: "Provider trait definition"
      exports: ["DownloadProvider", "DirectDownloadInfo"]
  key_links:
    - from: "src-tauri/src/download/engine.rs"
      to: "reqwest bytes_stream()"
      via: "StreamExt iteration"
      pattern: "bytes_stream.*next.*await"
    - from: "src-tauri/src/download/engine.rs"
      to: "tauri::ipc::Channel"
      via: "Progress event sending"
      pattern: "Channel.*DownloadEvent"
---

<objective>
Create the core download infrastructure with streaming downloads, progress tracking, and a provider trait for URL resolution abstraction.

Purpose: Establishes the foundation that Google Drive and Mediafire providers will implement. The streaming download engine ensures memory-efficient handling of large patch files (500MB+).

Output: Download module with Provider trait, streaming engine, and progress types ready for provider implementations.
</objective>

<execution_context>
@./.claude/get-shit-done/workflows/execute-plan.md
@./.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/2-download-engine/2-RESEARCH.md

# Existing types from Phase 1
@src-tauri/src/models/download.rs
@src-tauri/src/models/patch.rs
@src-tauri/Cargo.toml
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add dependencies and create module structure</name>
  <files>
    src-tauri/Cargo.toml
    src-tauri/src/download/mod.rs
    src-tauri/src/download/providers/mod.rs
  </files>
  <action>
1. Update Cargo.toml to add required dependencies:
   - futures-util = "0.3" (for StreamExt on bytes_stream)
   - uuid = { version = "1", features = ["v4"] } (for download IDs)
   - async-trait = "0.1" (for async trait definitions)
   Note: reqwest and tokio already present, but verify reqwest has "stream" feature enabled

2. Create src-tauri/src/download/mod.rs with:
   - Module declarations: pub mod engine; pub mod progress; pub mod providers;
   - Re-exports of key types
   - DownloadError enum using thiserror with variants:
     - HttpError(reqwest::StatusCode)
     - IoError(#[from] std::io::Error)
     - RequestError(#[from] reqwest::Error)
     - ProviderError(String) - for provider-specific failures
     - ChannelError(String) - for progress channel failures
     - ConfirmationFailed - for Google Drive virus scan
     - DirectUrlNotFound - for Mediafire resolution failure

3. Create src-tauri/src/download/providers/mod.rs with:
   - DownloadProvider async trait with:
     - async fn resolve_direct_url(&self, share_url: &str) -> Result<DirectDownloadInfo, DownloadError>
     - fn supports_resume(&self) -> bool
     - fn name(&self) -> &'static str
   - DirectDownloadInfo struct with fields:
     - url: String
     - file_name: Option<String>
     - content_length: Option<u64>
     - supports_range: bool
   - Module declarations for future providers: pub mod gdrive; pub mod mediafire;
  </action>
  <verify>
    cargo check --manifest-path src-tauri/Cargo.toml
    Verify no compilation errors. Module structure should be recognized.
  </verify>
  <done>
    - Cargo.toml has futures-util, uuid, async-trait dependencies
    - download/mod.rs exists with DownloadError enum
    - download/providers/mod.rs exists with DownloadProvider trait and DirectDownloadInfo struct
  </done>
</task>

<task type="auto">
  <name>Task 2: Implement progress tracking types</name>
  <files>
    src-tauri/src/download/progress.rs
  </files>
  <action>
Create src-tauri/src/download/progress.rs with:

1. DownloadEvent enum (for Tauri Channel):
   ```rust
   #[derive(Clone, serde::Serialize)]
   #[serde(rename_all = "camelCase", tag = "event", content = "data")]
   pub enum DownloadEvent {
       Started { download_id: String, file_name: String, total_bytes: u64 },
       Progress { download_id: String, downloaded_bytes: u64, total_bytes: u64, speed_bps: u64, percent: f32 },
       Completed { download_id: String, file_path: String },
       Failed { download_id: String, error: String },
   }
   ```
   Note: Use serde tag/content for discriminated union in TypeScript

2. ProgressTracker struct for throttled progress reporting:
   ```rust
   pub struct ProgressTracker {
       download_id: String,
       total_bytes: u64,
       downloaded_bytes: u64,
       start_time: std::time::Instant,
       last_report_time: std::time::Instant,
       min_report_interval: std::time::Duration,
   }
   ```

3. ProgressTracker impl with:
   - new(download_id: String, total_bytes: u64) -> Self
     Set min_report_interval to 100ms to avoid UI flooding
   - update(&mut self, chunk_size: u64) -> Option<DownloadEvent>
     Returns Some(Progress event) only if min_report_interval elapsed since last report
     Calculate speed_bps as downloaded_bytes / elapsed_secs
     Calculate percent as (downloaded_bytes as f32 / total_bytes as f32) * 100.0
   - started_event(&self, file_name: String) -> DownloadEvent
   - completed_event(&self, file_path: String) -> DownloadEvent
   - failed_event(&self, error: String) -> DownloadEvent

4. Add unit tests for ProgressTracker:
   - Test speed calculation
   - Test throttling (rapid updates don't all produce events)
   - Test percent calculation
  </action>
  <verify>
    cargo test --manifest-path src-tauri/Cargo.toml progress
    All progress tests pass.
  </verify>
  <done>
    - DownloadEvent enum serializes correctly for Tauri Channel
    - ProgressTracker throttles updates to 100ms intervals
    - Speed and percent calculations are accurate
    - 3+ unit tests pass
  </done>
</task>

<task type="auto">
  <name>Task 3: Implement streaming download engine</name>
  <files>
    src-tauri/src/download/engine.rs
    src-tauri/src/lib.rs
  </files>
  <action>
Create src-tauri/src/download/engine.rs with:

1. download_file async function:
   ```rust
   pub async fn download_file(
       client: &reqwest::Client,
       url: &str,
       dest_path: &std::path::Path,
       download_id: String,
       on_event: tauri::ipc::Channel<DownloadEvent>,
   ) -> Result<(), DownloadError>
   ```

2. Implementation:
   - Send GET request to url
   - Extract content_length from response (default to 0 if not present)
   - Extract file_name from Content-Disposition header or URL path
   - Send Started event via channel
   - Create ProgressTracker with download_id and total_bytes
   - Create destination file with tokio::fs::File::create
   - Get bytes_stream() from response
   - Use futures_util::StreamExt to iterate: while let Some(chunk) = stream.next().await
   - Write each chunk with tokio::io::AsyncWriteExt::write_all
   - Call progress_tracker.update(chunk.len()) and send event if Some returned
   - After loop completes, send Completed event
   - If any error occurs, send Failed event before returning Err

3. Helper function to extract filename:
   ```rust
   fn extract_filename(response: &reqwest::Response, url: &str) -> String
   ```
   - First try Content-Disposition header
   - Fall back to URL path's last segment
   - Default to "download" if all else fails

4. Update src-tauri/src/lib.rs:
   - Add: mod download;
   - No Tauri commands yet (will be added in plan 04)

5. Add integration test that downloads a small test file:
   - Use httpbin.org/bytes/1024 or similar public endpoint
   - Verify file is created with correct size
   - Verify progress events are sent
  </action>
  <verify>
    cargo check --manifest-path src-tauri/Cargo.toml
    cargo test --manifest-path src-tauri/Cargo.toml download
    All tests pass, no warnings about unused code.
  </verify>
  <done>
    - download_file function streams bytes without loading full file to memory
    - Progress events sent via Channel with throttling
    - File created at destination path
    - Started, Progress, Completed events all fire correctly
    - download module registered in lib.rs
  </done>
</task>

</tasks>

<verification>
After all tasks complete:

1. Module structure exists:
   ```
   src-tauri/src/download/
   ├── mod.rs           (exports, DownloadError)
   ├── engine.rs        (download_file)
   ├── progress.rs      (DownloadEvent, ProgressTracker)
   └── providers/
       └── mod.rs       (DownloadProvider trait, DirectDownloadInfo)
   ```

2. Cargo builds without errors:
   ```bash
   cargo build --manifest-path src-tauri/Cargo.toml
   ```

3. All tests pass:
   ```bash
   cargo test --manifest-path src-tauri/Cargo.toml
   ```

4. Provider trait is properly defined with async_trait
</verification>

<success_criteria>
- [ ] futures-util, uuid, async-trait added to Cargo.toml
- [ ] DownloadProvider trait defined with resolve_direct_url and supports_resume
- [ ] DirectDownloadInfo struct captures URL resolution results
- [ ] DownloadEvent enum serializes for Tauri Channel consumption
- [ ] ProgressTracker throttles updates to 100ms minimum interval
- [ ] download_file streams response without memory issues
- [ ] Progress events include speed_bps and percent
- [ ] All unit tests pass
- [ ] download module integrated into lib.rs
</success_criteria>

<output>
After completion, create `.planning/phases/2-download-engine/2-01-SUMMARY.md`
</output>
