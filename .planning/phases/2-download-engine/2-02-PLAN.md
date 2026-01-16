---
phase: 2-download-engine
plan: 02
type: execute
wave: 2
depends_on: [2-01]
files_modified:
  - src-tauri/src/download/providers/gdrive.rs
autonomous: true
user_setup: []

must_haves:
  truths:
    - "Google Drive share URLs are converted to direct download URLs"
    - "Large files (>100MB) with virus scan warning are handled"
    - "File ID is extracted from various Google Drive URL formats"
  artifacts:
    - path: "src-tauri/src/download/providers/gdrive.rs"
      provides: "Google Drive URL resolution"
      exports: ["GoogleDriveProvider"]
  key_links:
    - from: "src-tauri/src/download/providers/gdrive.rs"
      to: "DownloadProvider trait"
      via: "impl DownloadProvider for GoogleDriveProvider"
      pattern: "impl DownloadProvider"
    - from: "src-tauri/src/download/providers/gdrive.rs"
      to: "scraper crate"
      via: "HTML parsing for confirmation page"
      pattern: "Html::parse_document"
---

<objective>
Implement the Google Drive provider that resolves share URLs to direct download URLs, handling the virus scan confirmation page for large files.

Purpose: Google Drive is one of the two primary hosts for HD Patch files. The provider must handle various URL formats (file/d/, open?id=, uc?id=) and the confirmation dialog for files >100MB.

Output: GoogleDriveProvider implementing DownloadProvider trait, ready for use by the download engine.
</objective>

<execution_context>
@./.claude/get-shit-done/workflows/execute-plan.md
@./.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/phases/2-download-engine/2-RESEARCH.md

# From Plan 01 (will exist when this runs)
@src-tauri/src/download/providers/mod.rs
@src-tauri/src/download/mod.rs
</context>

<tasks>

<task type="auto">
  <name>Task 1: Implement Google Drive URL parsing and file ID extraction</name>
  <files>
    src-tauri/src/download/providers/gdrive.rs
  </files>
  <action>
Create src-tauri/src/download/providers/gdrive.rs with:

1. GoogleDriveProvider struct:
   ```rust
   pub struct GoogleDriveProvider {
       client: reqwest::Client,
   }

   impl GoogleDriveProvider {
       pub fn new(client: reqwest::Client) -> Self {
           Self { client }
       }
   }
   ```

2. File ID extraction function using lazy_static regex patterns:
   ```rust
   use lazy_static::lazy_static;
   use regex::Regex;

   lazy_static! {
       // Matches: drive.google.com/file/d/FILE_ID/...
       static ref FILE_D_PATTERN: Regex = Regex::new(r"drive\.google\.com/file/d/([a-zA-Z0-9_-]+)").unwrap();
       // Matches: drive.google.com/open?id=FILE_ID
       static ref OPEN_ID_PATTERN: Regex = Regex::new(r"drive\.google\.com/open\?id=([a-zA-Z0-9_-]+)").unwrap();
       // Matches: drive.google.com/uc?id=FILE_ID or ?export=download&id=FILE_ID
       static ref UC_ID_PATTERN: Regex = Regex::new(r"[?&]id=([a-zA-Z0-9_-]+)").unwrap();
   }

   impl GoogleDriveProvider {
       /// Extract file ID from various Google Drive URL formats
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
   ```

3. Add unit tests for file ID extraction:
   - Test file/d/ format: "https://drive.google.com/file/d/1ABC123/view?usp=sharing"
   - Test open?id= format: "https://drive.google.com/open?id=1ABC123"
   - Test uc?id= format: "https://drive.google.com/uc?id=1ABC123"
   - Test invalid URL returns None
  </action>
  <verify>
    cargo test --manifest-path src-tauri/Cargo.toml gdrive
    All file ID extraction tests pass.
  </verify>
  <done>
    - extract_file_id handles all three Google Drive URL formats
    - get_direct_url constructs proper uc?export=download URL
    - 4+ unit tests pass
  </done>
</task>

<task type="auto">
  <name>Task 2: Implement virus scan confirmation handling and DownloadProvider trait</name>
  <files>
    src-tauri/src/download/providers/gdrive.rs
    src-tauri/src/download/providers/mod.rs
  </files>
  <action>
1. Add virus scan confirmation handling to GoogleDriveProvider:
   ```rust
   use scraper::{Html, Selector};

   impl GoogleDriveProvider {
       /// Handle large file confirmation (>100MB virus scan warning)
       /// Returns the final direct download URL after any required confirmation
       async fn resolve_with_confirmation(&self, file_id: &str) -> Result<DirectDownloadInfo, DownloadError> {
           let initial_url = Self::get_direct_url(file_id);

           let response = self.client
               .get(&initial_url)
               .send()
               .await
               .map_err(DownloadError::RequestError)?;

           let content_type = response.headers()
               .get("content-type")
               .and_then(|v| v.to_str().ok())
               .unwrap_or("");

           // If we got HTML, it's the virus scan warning page
           if content_type.contains("text/html") {
               let html = response.text().await.map_err(DownloadError::RequestError)?;
               return self.parse_confirmation_page(&html, file_id);
           }

           // No confirmation needed - extract info from response
           let content_length = response.content_length();
           let file_name = Self::extract_filename_from_headers(response.headers());

           // Check if Range requests are supported
           let supports_range = response.headers()
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

       fn parse_confirmation_page(&self, html: &str, file_id: &str) -> Result<DirectDownloadInfo, DownloadError> {
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

           // Method 2: Look for form with confirm parameter
           let form_selector = Selector::parse("form[action*='confirm']").unwrap();
           if let Some(form) = document.select(&form_selector).next() {
               if let Some(action) = form.value().attr("action") {
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
           lazy_static! {
               static ref CONFIRM_REGEX: Regex = Regex::new(r"confirm=([0-9A-Za-z_-]+)").unwrap();
           }
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

       fn extract_filename_from_headers(headers: &reqwest::header::HeaderMap) -> Option<String> {
           headers.get("content-disposition")
               .and_then(|v| v.to_str().ok())
               .and_then(|v| {
                   // Parse: attachment; filename="patch-A.rar"
                   v.split("filename=").nth(1)
                       .map(|s| s.trim_matches('"').to_string())
               })
       }
   }
   ```

2. Implement DownloadProvider trait for GoogleDriveProvider:
   ```rust
   use async_trait::async_trait;

   #[async_trait]
   impl DownloadProvider for GoogleDriveProvider {
       async fn resolve_direct_url(&self, share_url: &str) -> Result<DirectDownloadInfo, DownloadError> {
           let file_id = Self::extract_file_id(share_url)
               .ok_or_else(|| DownloadError::ProviderError(
                   "Could not extract Google Drive file ID from URL".to_string()
               ))?;

           self.resolve_with_confirmation(&file_id).await
       }

       fn supports_resume(&self) -> bool {
           true // Google Drive generally supports Range headers
       }

       fn name(&self) -> &'static str {
           "Google Drive"
       }
   }
   ```

3. Update src-tauri/src/download/providers/mod.rs:
   - Uncomment or add: pub mod gdrive;
   - Add re-export: pub use gdrive::GoogleDriveProvider;

4. Add integration test (can be ignored if no network):
   - Test with a known small public Google Drive file
   - Mark test with #[ignore] for CI environments
  </action>
  <verify>
    cargo check --manifest-path src-tauri/Cargo.toml
    cargo test --manifest-path src-tauri/Cargo.toml gdrive
    All tests pass, trait implementation compiles.
  </verify>
  <done>
    - GoogleDriveProvider implements DownloadProvider trait
    - Virus scan confirmation page is parsed and handled
    - resolve_direct_url returns DirectDownloadInfo with URL, filename, size
    - Provider is exported from providers module
  </done>
</task>

</tasks>

<verification>
After all tasks complete:

1. Google Drive provider exists and compiles:
   ```bash
   cargo check --manifest-path src-tauri/Cargo.toml
   ```

2. File ID extraction works for all URL formats:
   ```bash
   cargo test --manifest-path src-tauri/Cargo.toml gdrive::tests
   ```

3. Provider implements DownloadProvider trait correctly:
   - resolve_direct_url is async and returns Result<DirectDownloadInfo, DownloadError>
   - supports_resume returns bool
   - name returns static str
</verification>

<success_criteria>
- [ ] GoogleDriveProvider struct created with reqwest::Client
- [ ] extract_file_id handles file/d/, open?id=, and uc?id= URL formats
- [ ] resolve_with_confirmation handles virus scan warning page
- [ ] HTML parsing extracts confirmation URL from link, form, or regex
- [ ] DownloadProvider trait implemented for GoogleDriveProvider
- [ ] Provider exported from providers/mod.rs
- [ ] All unit tests pass (6+ tests)
</success_criteria>

<output>
After completion, create `.planning/phases/2-download-engine/2-02-SUMMARY.md`
</output>
