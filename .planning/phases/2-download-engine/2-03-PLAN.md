---
phase: 2-download-engine
plan: 03
type: execute
wave: 2
depends_on: [2-01]
files_modified:
  - src-tauri/src/download/providers/mediafire.rs
autonomous: true
user_setup: []

must_haves:
  truths:
    - "Mediafire share URLs are converted to direct download URLs"
    - "Dynamic download subdomains (download1, download2, etc.) are handled"
    - "File name is extracted from the share page"
  artifacts:
    - path: "src-tauri/src/download/providers/mediafire.rs"
      provides: "Mediafire URL resolution"
      exports: ["MediafireProvider"]
  key_links:
    - from: "src-tauri/src/download/providers/mediafire.rs"
      to: "DownloadProvider trait"
      via: "impl DownloadProvider for MediafireProvider"
      pattern: "impl DownloadProvider"
    - from: "src-tauri/src/download/providers/mediafire.rs"
      to: "regex for download URL"
      via: "Extract download URL from page HTML"
      pattern: r"download\d+\.mediafire\.com"
---

<objective>
Implement the Mediafire provider that resolves share URLs to direct download URLs by parsing the share page HTML.

Purpose: Mediafire is one of the two primary hosts for HD Patch files. Unlike Google Drive, Mediafire uses dynamic numbered subdomains (download1, download2, etc.) that must be extracted from the share page.

Output: MediafireProvider implementing DownloadProvider trait, ready for use by the download engine.
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
  <name>Task 1: Implement Mediafire URL parsing and share page fetching</name>
  <files>
    src-tauri/src/download/providers/mediafire.rs
  </files>
  <action>
Create src-tauri/src/download/providers/mediafire.rs with:

1. MediafireProvider struct:
   ```rust
   pub struct MediafireProvider {
       client: reqwest::Client,
   }

   impl MediafireProvider {
       pub fn new(client: reqwest::Client) -> Self {
           Self { client }
       }
   }
   ```

2. URL validation using lazy_static regex:
   ```rust
   use lazy_static::lazy_static;
   use regex::Regex;

   lazy_static! {
       // Matches mediafire.com share URLs
       static ref MEDIAFIRE_SHARE_PATTERN: Regex = Regex::new(
           r"(?:www\.)?mediafire\.com/(?:file|view|download|folder)/([a-zA-Z0-9]+)"
       ).unwrap();

       // Matches the actual download URL with numbered subdomain
       static ref MEDIAFIRE_DOWNLOAD_URL: Regex = Regex::new(
           r#"https://download\d+\.mediafire\.com/[^'"<>\s]+"#
       ).unwrap();

       // Matches pre-download URL with dkey parameter (fallback)
       static ref MEDIAFIRE_DKEY_URL: Regex = Regex::new(
           r#"https?://(?:www\.)?mediafire\.com/(?:file|view|download)/[^'"\?]+\?dkey=[^'"<>\s]+"#
       ).unwrap();
   }

   impl MediafireProvider {
       /// Validate that URL is a Mediafire share URL
       pub fn is_mediafire_url(url: &str) -> bool {
           MEDIAFIRE_SHARE_PATTERN.is_match(url)
       }

       /// Extract the direct download URL from page HTML
       fn extract_download_url(html: &str) -> Option<String> {
           // Primary: Look for download URL with numbered subdomain
           if let Some(m) = MEDIAFIRE_DOWNLOAD_URL.find(html) {
               return Some(m.as_str().to_string());
           }
           None
       }

       /// Extract dkey URL as fallback (needs second fetch)
       fn extract_dkey_url(html: &str) -> Option<String> {
           if let Some(m) = MEDIAFIRE_DKEY_URL.find(html) {
               return Some(m.as_str().to_string());
           }
           None
       }
   }
   ```

3. Add unit tests for URL validation:
   - Test valid file URL: "https://www.mediafire.com/file/abc123/patch.rar"
   - Test valid folder URL: "https://mediafire.com/folder/abc123"
   - Test invalid URL: "https://example.com/file"
   - Test download URL extraction from sample HTML
  </action>
  <verify>
    cargo test --manifest-path src-tauri/Cargo.toml mediafire
    All URL validation tests pass.
  </verify>
  <done>
    - is_mediafire_url validates Mediafire share URLs
    - extract_download_url finds download URLs in HTML
    - 4+ unit tests pass
  </done>
</task>

<task type="auto">
  <name>Task 2: Implement URL resolution and DownloadProvider trait</name>
  <files>
    src-tauri/src/download/providers/mediafire.rs
    src-tauri/src/download/providers/mod.rs
  </files>
  <action>
1. Add full URL resolution to MediafireProvider:
   ```rust
   use scraper::{Html, Selector};
   use async_trait::async_trait;

   impl MediafireProvider {
       /// Fetch the share page and extract the direct download URL
       async fn resolve_from_share_page(&self, share_url: &str) -> Result<DirectDownloadInfo, DownloadError> {
           // Step 1: Fetch the share page with browser-like headers
           let response = self.client
               .get(share_url)
               .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
               .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
               .send()
               .await
               .map_err(DownloadError::RequestError)?;

           if !response.status().is_success() {
               return Err(DownloadError::HttpError(response.status()));
           }

           let html = response.text().await.map_err(DownloadError::RequestError)?;

           // Step 2: Try to find direct download URL
           if let Some(url) = Self::extract_download_url(&html) {
               let file_name = Self::extract_filename_from_page(&html);
               return Ok(DirectDownloadInfo {
                   url,
                   file_name,
                   content_length: None, // Will be determined during download
                   supports_range: true, // Mediafire generally supports Range
               });
           }

           // Step 3: If not found, try dkey URL and follow it
           if let Some(dkey_url) = Self::extract_dkey_url(&html) {
               // Small delay to avoid rate limiting (as per research)
               tokio::time::sleep(std::time::Duration::from_millis(1500)).await;

               let response2 = self.client
                   .get(&dkey_url)
                   .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                   .send()
                   .await
                   .map_err(DownloadError::RequestError)?;

               let html2 = response2.text().await.map_err(DownloadError::RequestError)?;

               if let Some(url) = Self::extract_download_url(&html2) {
                   let file_name = Self::extract_filename_from_page(&html2)
                       .or_else(|| Self::extract_filename_from_page(&html));
                   return Ok(DirectDownloadInfo {
                       url,
                       file_name,
                       content_length: None,
                       supports_range: true,
                   });
               }
           }

           Err(DownloadError::DirectUrlNotFound)
       }

       /// Extract filename from Mediafire page HTML
       fn extract_filename_from_page(html: &str) -> Option<String> {
           let document = Html::parse_document(html);

           // Try: div.filename
           let filename_selector = Selector::parse("div.filename").ok()?;
           if let Some(elem) = document.select(&filename_selector).next() {
               let text = elem.text().collect::<String>().trim().to_string();
               if !text.is_empty() {
                   return Some(text);
               }
           }

           // Try: download button text or title attribute
           let download_selector = Selector::parse("a.input[aria-label*='Download']").ok()?;
           if let Some(elem) = document.select(&download_selector).next() {
               if let Some(title) = elem.value().attr("title") {
                   return Some(title.to_string());
               }
           }

           None
       }
   }
   ```

2. Implement DownloadProvider trait:
   ```rust
   #[async_trait]
   impl DownloadProvider for MediafireProvider {
       async fn resolve_direct_url(&self, share_url: &str) -> Result<DirectDownloadInfo, DownloadError> {
           if !Self::is_mediafire_url(share_url) {
               return Err(DownloadError::ProviderError(
                   "Not a valid Mediafire URL".to_string()
               ));
           }

           self.resolve_from_share_page(share_url).await
       }

       fn supports_resume(&self) -> bool {
           true // Mediafire generally supports Range headers
       }

       fn name(&self) -> &'static str {
           "Mediafire"
       }
   }
   ```

3. Update src-tauri/src/download/providers/mod.rs:
   - Add: pub mod mediafire;
   - Add re-export: pub use mediafire::MediafireProvider;

4. Add unit test for filename extraction with sample HTML:
   ```rust
   #[test]
   fn test_extract_filename_from_page() {
       let html = r#"
           <html>
               <div class="filename">patch-A.rar</div>
           </html>
       "#;
       assert_eq!(
           MediafireProvider::extract_filename_from_page(html),
           Some("patch-A.rar".to_string())
       );
   }
   ```
  </action>
  <verify>
    cargo check --manifest-path src-tauri/Cargo.toml
    cargo test --manifest-path src-tauri/Cargo.toml mediafire
    All tests pass, trait implementation compiles.
  </verify>
  <done>
    - MediafireProvider implements DownloadProvider trait
    - resolve_from_share_page fetches page and extracts download URL
    - Fallback to dkey URL when direct URL not found
    - Filename extraction from page HTML
    - Provider exported from providers module
  </done>
</task>

</tasks>

<verification>
After all tasks complete:

1. Mediafire provider exists and compiles:
   ```bash
   cargo check --manifest-path src-tauri/Cargo.toml
   ```

2. URL validation and extraction work:
   ```bash
   cargo test --manifest-path src-tauri/Cargo.toml mediafire::tests
   ```

3. Provider implements DownloadProvider trait correctly:
   - resolve_direct_url is async and returns Result<DirectDownloadInfo, DownloadError>
   - supports_resume returns bool
   - name returns static str

4. Both providers now exported:
   ```rust
   use crate::download::providers::{GoogleDriveProvider, MediafireProvider};
   ```
</verification>

<success_criteria>
- [ ] MediafireProvider struct created with reqwest::Client
- [ ] is_mediafire_url validates share URLs
- [ ] extract_download_url finds download URLs with numbered subdomains
- [ ] resolve_from_share_page fetches page and extracts URL
- [ ] Fallback to dkey URL with 1.5s delay implemented
- [ ] Filename extraction from page HTML works
- [ ] DownloadProvider trait implemented for MediafireProvider
- [ ] Provider exported from providers/mod.rs
- [ ] All unit tests pass (5+ tests)
</success_criteria>

<output>
After completion, create `.planning/phases/2-download-engine/2-03-SUMMARY.md`
</output>
