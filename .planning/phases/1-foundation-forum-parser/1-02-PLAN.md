---
phase: 1-foundation-forum-parser
plan: 02
type: execute
wave: 2
depends_on: ["1-01"]
files_modified:
  - src-tauri/src/parser/forum.rs
  - src-tauri/src/parser/modules.rs
  - src-tauri/src/parser/links.rs
  - src-tauri/src/parser/dependencies.rs
  - src-tauri/src/lib.rs
autonomous: true

must_haves:
  truths:
    - "App can fetch HTML from the forum post URL"
    - "App can extract post content from phpBB HTML structure"
    - "App can identify download links (Mediafire and Google Drive)"
    - "App validates module dependencies correctly"
  artifacts:
    - path: "src-tauri/src/parser/forum.rs"
      provides: "Forum HTML fetching and parsing"
      contains: "fetch_forum_post"
    - path: "src-tauri/src/parser/links.rs"
      provides: "Download link extraction"
      contains: "MEDIAFIRE_FILE"
    - path: "src-tauri/src/parser/dependencies.rs"
      provides: "Dependency validation logic"
      contains: "validate_module_selection"
  key_links:
    - from: "src-tauri/src/parser/forum.rs"
      to: "tauri_plugin_http::reqwest"
      via: "HTTP client"
      pattern: "reqwest::Client"
    - from: "src-tauri/src/parser/forum.rs"
      to: "scraper"
      via: "HTML parsing"
      pattern: "Html::parse_document"
    - from: "src-tauri/src/parser/links.rs"
      to: "regex"
      via: "URL extraction"
      pattern: "Regex::new"
---

<objective>
Implement the complete forum parser that fetches the HD Patch forum post, extracts patch module information, identifies download links, and validates module dependencies.

Purpose: Enable the app to discover available patches and their download sources dynamically from the forum.
Output: Working parser that can be called from the frontend to get patch information.
</objective>

<execution_context>
@./.claude/get-shit-done/workflows/execute-plan.md
@./.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/phases/1-foundation-forum-parser/1-RESEARCH.md
@.planning/phases/1-foundation-forum-parser/1-01-SUMMARY.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Implement forum fetching and HTML parsing</name>
  <files>
    src-tauri/src/parser/forum.rs
  </files>
  <action>
Implement the forum fetcher and HTML parser in `src-tauri/src/parser/forum.rs`:

```rust
//! Forum fetching and HTML parsing

use scraper::{Html, Selector};
use tauri_plugin_http::reqwest;

use crate::models::{ParsedForumPost, ParserError};

/// Default forum URL for HD Patch: Reforged
pub const FORUM_URL: &str = "https://forum.turtlecraft.gg/viewtopic.php?t=21355";

/// Alternative forum URL (old domain, may redirect)
pub const FORUM_URL_ALT: &str = "https://forum.turtle-wow.org/viewtopic.php?t=21355";

pub struct ForumParser {
    content_selector: Selector,
    link_selector: Selector,
}

impl ForumParser {
    pub fn new() -> Result<Self, ParserError> {
        Ok(Self {
            // phpBB post content is in div.postbody div.content
            content_selector: Selector::parse("div.postbody div.content")
                .map_err(|e| ParserError::SelectorParse(format!("{:?}", e)))?,
            link_selector: Selector::parse("a[href]")
                .map_err(|e| ParserError::SelectorParse(format!("{:?}", e)))?,
        })
    }

    /// Parse forum HTML and extract the first post content
    pub fn parse(&self, html: &str) -> Result<ParsedForumPost, ParserError> {
        let document = Html::parse_document(html);

        // Get first post content (main HD Patch info)
        let content = document
            .select(&self.content_selector)
            .next()
            .map(|el| el.inner_html())
            .ok_or(ParserError::PostNotFound)?;

        // Extract all links from the document
        let links: Vec<String> = document
            .select(&self.link_selector)
            .filter_map(|el| el.value().attr("href"))
            .map(String::from)
            .collect();

        Ok(ParsedForumPost { content, links })
    }

    /// Extract text content (without HTML tags) from the first post
    pub fn extract_text(&self, html: &str) -> Result<String, ParserError> {
        let document = Html::parse_document(html);

        let text = document
            .select(&self.content_selector)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join(" "))
            .ok_or(ParserError::PostNotFound)?;

        Ok(text)
    }
}

impl Default for ForumParser {
    fn default() -> Self {
        Self::new().expect("Failed to create default ForumParser")
    }
}

/// Fetch the forum post HTML from the given URL
pub async fn fetch_forum_post(url: &str) -> Result<String, ParserError> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e| ParserError::HttpError(e.to_string()))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| ParserError::HttpError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(ParserError::HttpError(format!(
            "HTTP {} from {}",
            response.status(),
            url
        )));
    }

    response
        .text()
        .await
        .map_err(|e| ParserError::HttpError(e.to_string()))
}

/// Fetch forum post with fallback to alternate URL
pub async fn fetch_forum_post_with_fallback() -> Result<String, ParserError> {
    match fetch_forum_post(FORUM_URL).await {
        Ok(html) => Ok(html),
        Err(e) => {
            eprintln!("Primary URL failed: {}. Trying alternate...", e);
            fetch_forum_post(FORUM_URL_ALT).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_HTML: &str = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <div class="postbody">
            <div class="content">
                <p>HD Patch: Reforged</p>
                <p>Patch-A: Player Characters</p>
                <a href="https://www.mediafire.com/file/abc123/patch-a.7z">Download Patch-A</a>
                <a href="https://drive.google.com/file/d/xyz789/view">Google Drive Mirror</a>
            </div>
        </div>
    </body>
    </html>
    "#;

    #[test]
    fn test_parse_forum_html() {
        let parser = ForumParser::new().unwrap();
        let result = parser.parse(SAMPLE_HTML).unwrap();

        assert!(result.content.contains("HD Patch: Reforged"));
        assert!(result.content.contains("Patch-A"));
        assert_eq!(result.links.len(), 2);
        assert!(result.links.iter().any(|l| l.contains("mediafire.com")));
        assert!(result.links.iter().any(|l| l.contains("drive.google.com")));
    }

    #[test]
    fn test_extract_text() {
        let parser = ForumParser::new().unwrap();
        let text = parser.extract_text(SAMPLE_HTML).unwrap();

        assert!(text.contains("HD Patch: Reforged"));
        assert!(text.contains("Patch-A"));
        // Should not contain HTML tags
        assert!(!text.contains("<p>"));
        assert!(!text.contains("<a"));
    }

    #[test]
    fn test_missing_post_content() {
        let parser = ForumParser::new().unwrap();
        let empty_html = "<html><body></body></html>";
        let result = parser.parse(empty_html);

        assert!(matches!(result, Err(ParserError::PostNotFound)));
    }
}
```
  </action>
  <verify>
Run `cargo test` in src-tauri - forum.rs tests should pass.
Run `cargo check` - should compile without errors.
  </verify>
  <done>
Forum fetcher implemented with HTTP client, HTML parsing using scraper, and fallback URL support. Tests pass for parsing logic.
  </done>
</task>

<task type="auto">
  <name>Task 2: Implement download link extraction</name>
  <files>
    src-tauri/src/parser/links.rs
  </files>
  <action>
Implement link extraction with regex patterns in `src-tauri/src/parser/links.rs`:

```rust
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
        r"https://(?:www\.)?mediafire\.com/file/([a-z0-9]+)/([^/\s\"<>]+)"
    ).unwrap();

    static ref MEDIAFIRE_FOLDER: Regex = Regex::new(
        r"https://(?:www\.)?(?:app\.)?mediafire\.com/folder/([a-z0-9]+)"
    ).unwrap();

    static ref MEDIAFIRE_VIEW: Regex = Regex::new(
        r"https://(?:www\.)?mediafire\.com/view/([a-z0-9]+)/([^/\s\"<>]+)"
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
```
  </action>
  <verify>
Run `cargo test` in src-tauri - links.rs tests should pass.
  </verify>
  <done>
Link extraction implemented with regex patterns for Mediafire and Google Drive URLs. Deduplication and ID extraction utilities included.
  </done>
</task>

<task type="auto">
  <name>Task 3: Implement dependency validation and Tauri commands</name>
  <files>
    src-tauri/src/parser/dependencies.rs
    src-tauri/src/parser/modules.rs
    src-tauri/src/lib.rs
  </files>
  <action>
1. Implement dependency validation in `src-tauri/src/parser/dependencies.rs`:

```rust
//! Module dependency validation

use std::collections::HashSet;
use crate::models::PatchId;

/// Validate that a selection of modules satisfies all dependency rules
///
/// Rules:
/// - B, D, E must be installed together (all or none)
/// - L requires A
/// - U requires A and G
/// - O requires S
pub fn validate_module_selection(selected: &HashSet<PatchId>) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // B, D, E must be together or none
    let bde_group = [PatchId::B, PatchId::D, PatchId::E];
    let bde_selected: Vec<_> = bde_group.iter().filter(|p| selected.contains(p)).collect();
    if !bde_selected.is_empty() && bde_selected.len() != 3 {
        let missing: Vec<_> = bde_group
            .iter()
            .filter(|p| !selected.contains(p))
            .map(|p| format!("{:?}", p))
            .collect();
        errors.push(format!(
            "Patches B, D, and E must be installed together. Missing: {}",
            missing.join(", ")
        ));
    }

    // L requires A
    if selected.contains(&PatchId::L) && !selected.contains(&PatchId::A) {
        errors.push("Patch L (A Little Extra for Females) requires Patch A (Player Characters)".into());
    }

    // U requires A and G
    if selected.contains(&PatchId::U) {
        if !selected.contains(&PatchId::A) {
            errors.push("Patch U (Ultra HD) requires Patch A (Player Characters)".into());
        }
        if !selected.contains(&PatchId::G) {
            errors.push("Patch U (Ultra HD) requires Patch G (Gear & Weapons)".into());
        }
    }

    // O requires S
    if selected.contains(&PatchId::O) && !selected.contains(&PatchId::S) {
        errors.push("Patch O (Raid Visuals Mod) requires Patch S (Sounds & Music)".into());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Get the dependencies for a given patch
pub fn get_dependencies(patch: PatchId) -> Vec<PatchId> {
    match patch {
        PatchId::B => vec![PatchId::D, PatchId::E],
        PatchId::D => vec![PatchId::B, PatchId::E],
        PatchId::E => vec![PatchId::B, PatchId::D],
        PatchId::L => vec![PatchId::A],
        PatchId::U => vec![PatchId::A, PatchId::G],
        PatchId::O => vec![PatchId::S],
        _ => vec![],
    }
}

/// Auto-select dependencies for a given selection
/// Returns a new set with all required dependencies added
pub fn auto_select_dependencies(selected: &HashSet<PatchId>) -> HashSet<PatchId> {
    let mut result = selected.clone();

    for patch in selected.iter() {
        for dep in get_dependencies(*patch) {
            result.insert(dep);
        }
    }

    // Recurse until no more changes (handles transitive deps)
    if result.len() > selected.len() {
        auto_select_dependencies(&result)
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_selection_empty() {
        let selected = HashSet::new();
        assert!(validate_module_selection(&selected).is_ok());
    }

    #[test]
    fn test_valid_selection_single() {
        let mut selected = HashSet::new();
        selected.insert(PatchId::A);
        selected.insert(PatchId::C);
        selected.insert(PatchId::G);
        assert!(validate_module_selection(&selected).is_ok());
    }

    #[test]
    fn test_valid_bde_together() {
        let mut selected = HashSet::new();
        selected.insert(PatchId::B);
        selected.insert(PatchId::D);
        selected.insert(PatchId::E);
        assert!(validate_module_selection(&selected).is_ok());
    }

    #[test]
    fn test_invalid_bde_partial() {
        let mut selected = HashSet::new();
        selected.insert(PatchId::B);
        selected.insert(PatchId::D);
        // Missing E
        let result = validate_module_selection(&selected);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("B, D, and E must be installed together"));
    }

    #[test]
    fn test_invalid_l_without_a() {
        let mut selected = HashSet::new();
        selected.insert(PatchId::L);
        let result = validate_module_selection(&selected);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("Patch L"));
        assert!(errors[0].contains("requires Patch A"));
    }

    #[test]
    fn test_invalid_u_without_a_and_g() {
        let mut selected = HashSet::new();
        selected.insert(PatchId::U);
        let result = validate_module_selection(&selected);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2); // Missing both A and G
    }

    #[test]
    fn test_invalid_o_without_s() {
        let mut selected = HashSet::new();
        selected.insert(PatchId::O);
        let result = validate_module_selection(&selected);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("Patch O"));
    }

    #[test]
    fn test_auto_select_dependencies() {
        let mut selected = HashSet::new();
        selected.insert(PatchId::L);

        let result = auto_select_dependencies(&selected);
        assert!(result.contains(&PatchId::L));
        assert!(result.contains(&PatchId::A)); // Auto-added
    }

    #[test]
    fn test_auto_select_bde() {
        let mut selected = HashSet::new();
        selected.insert(PatchId::B);

        let result = auto_select_dependencies(&selected);
        assert!(result.contains(&PatchId::B));
        assert!(result.contains(&PatchId::D));
        assert!(result.contains(&PatchId::E));
    }
}
```

2. Update `src-tauri/src/parser/modules.rs` with module metadata:

```rust
//! Patch module parsing and metadata

use crate::models::{PatchId, PatchModule, ParserError, DownloadLink};
use crate::parser::dependencies::get_dependencies;

/// Get metadata for all known patch modules
pub fn get_all_modules() -> Vec<PatchModule> {
    vec![
        PatchModule {
            id: PatchId::A,
            name: "Player Characters & NPCs".into(),
            description: "HD models and textures for players and NPCs".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::A),
            file_size: None,
            last_updated: None,
        },
        PatchModule {
            id: PatchId::B,
            name: "Buildings".into(),
            description: "HD building models".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::B),
            file_size: None,
            last_updated: None,
        },
        PatchModule {
            id: PatchId::C,
            name: "Creatures".into(),
            description: "HD creature models".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::C),
            file_size: None,
            last_updated: None,
        },
        PatchModule {
            id: PatchId::D,
            name: "Doodads".into(),
            description: "HD doodad models".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::D),
            file_size: None,
            last_updated: None,
        },
        PatchModule {
            id: PatchId::E,
            name: "Environment".into(),
            description: "HD environment textures".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::E),
            file_size: None,
            last_updated: None,
        },
        PatchModule {
            id: PatchId::G,
            name: "Gear & Weapons".into(),
            description: "HD equipment models".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::G),
            file_size: None,
            last_updated: None,
        },
        PatchModule {
            id: PatchId::I,
            name: "Interface".into(),
            description: "HD UI elements".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::I),
            file_size: None,
            last_updated: None,
        },
        PatchModule {
            id: PatchId::L,
            name: "A Little Extra for Females".into(),
            description: "Additional female model options".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::L),
            file_size: None,
            last_updated: None,
        },
        PatchModule {
            id: PatchId::M,
            name: "Maps & Loading Screens".into(),
            description: "HD loading screens".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::M),
            file_size: None,
            last_updated: None,
        },
        PatchModule {
            id: PatchId::N,
            name: "Darker Nights".into(),
            description: "Night ambiance enhancement".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::N),
            file_size: None,
            last_updated: None,
        },
        PatchModule {
            id: PatchId::O,
            name: "Raid Visuals Mod".into(),
            description: "Enhanced raid effects".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::O),
            file_size: None,
            last_updated: None,
        },
        PatchModule {
            id: PatchId::S,
            name: "Sounds & Music".into(),
            description: "HD audio".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::S),
            file_size: None,
            last_updated: None,
        },
        PatchModule {
            id: PatchId::U,
            name: "Ultra HD".into(),
            description: "4K character textures".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::U),
            file_size: None,
            last_updated: None,
        },
        PatchModule {
            id: PatchId::V,
            name: "Visual Effects".into(),
            description: "HD spell effects".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::V),
            file_size: None,
            last_updated: None,
        },
    ]
}

/// Parse modules from forum post content
/// This enriches the known module list with download links found in the content
pub fn parse_modules(content: &str, links: &[DownloadLink]) -> Result<Vec<PatchModule>, ParserError> {
    let mut modules = get_all_modules();

    // Associate links with modules based on filename patterns
    for link in links {
        if let Some(filename) = &link.file_name {
            let filename_lower = filename.to_lowercase();

            // Try to match filename to module
            let module_id = if filename_lower.contains("patch-a") || filename_lower.contains("patch_a") {
                Some(PatchId::A)
            } else if filename_lower.contains("patch-b") || filename_lower.contains("patch_b") {
                Some(PatchId::B)
            } else if filename_lower.contains("patch-c") || filename_lower.contains("patch_c") {
                Some(PatchId::C)
            } else if filename_lower.contains("patch-d") || filename_lower.contains("patch_d") {
                Some(PatchId::D)
            } else if filename_lower.contains("patch-e") || filename_lower.contains("patch_e") {
                Some(PatchId::E)
            } else if filename_lower.contains("patch-g") || filename_lower.contains("patch_g") {
                Some(PatchId::G)
            } else if filename_lower.contains("patch-i") || filename_lower.contains("patch_i") {
                Some(PatchId::I)
            } else if filename_lower.contains("patch-l") || filename_lower.contains("patch_l") {
                Some(PatchId::L)
            } else if filename_lower.contains("patch-m") || filename_lower.contains("patch_m") {
                Some(PatchId::M)
            } else if filename_lower.contains("patch-n") || filename_lower.contains("patch_n") {
                Some(PatchId::N)
            } else if filename_lower.contains("patch-o") || filename_lower.contains("patch_o") {
                Some(PatchId::O)
            } else if filename_lower.contains("patch-s") || filename_lower.contains("patch_s") {
                Some(PatchId::S)
            } else if filename_lower.contains("patch-u") || filename_lower.contains("patch_u") {
                Some(PatchId::U)
            } else if filename_lower.contains("patch-v") || filename_lower.contains("patch_v") {
                Some(PatchId::V)
            } else {
                None
            };

            if let Some(id) = module_id {
                if let Some(module) = modules.iter_mut().find(|m| m.id == id) {
                    module.downloads.push(link.clone());
                }
            }
        }
    }

    Ok(modules)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DownloadProvider;

    #[test]
    fn test_get_all_modules() {
        let modules = get_all_modules();
        assert_eq!(modules.len(), 14);

        // Check specific modules exist
        assert!(modules.iter().any(|m| m.id == PatchId::A));
        assert!(modules.iter().any(|m| m.id == PatchId::U));
    }

    #[test]
    fn test_parse_modules_with_links() {
        let links = vec![
            DownloadLink {
                provider: DownloadProvider::Mediafire,
                url: "https://mediafire.com/file/xyz/patch-a.7z".into(),
                file_name: Some("patch-a.7z".into()),
            },
            DownloadLink {
                provider: DownloadProvider::GoogleDrive,
                url: "https://drive.google.com/file/d/abc/view".into(),
                file_name: Some("Patch-G-v2.rar".into()),
            },
        ];

        let modules = parse_modules("", &links).unwrap();

        let module_a = modules.iter().find(|m| m.id == PatchId::A).unwrap();
        assert_eq!(module_a.downloads.len(), 1);

        let module_g = modules.iter().find(|m| m.id == PatchId::G).unwrap();
        assert_eq!(module_g.downloads.len(), 1);
    }
}
```

3. Update `src-tauri/src/lib.rs` with Tauri commands:

```rust
mod parser;
mod models;

use std::collections::HashSet;
use models::{PatchModule, PatchId};
use parser::{
    forum::{fetch_forum_post_with_fallback, ForumParser, FORUM_URL},
    links::extract_download_links,
    modules::parse_modules,
    dependencies::{validate_module_selection, auto_select_dependencies},
};

#[tauri::command]
async fn fetch_patches() -> Result<Vec<PatchModule>, String> {
    // Fetch forum HTML
    let html = fetch_forum_post_with_fallback()
        .await
        .map_err(|e| e.to_string())?;

    // Parse the HTML
    let parser = ForumParser::new().map_err(|e| e.to_string())?;
    let parsed = parser.parse(&html).map_err(|e| e.to_string())?;

    // Extract download links
    let links = extract_download_links(&parsed.content);

    // Also extract from raw link URLs
    let mut all_links = links;
    for url in &parsed.links {
        all_links.extend(extract_download_links(url));
    }

    // Build module list with links
    let modules = parse_modules(&parsed.content, &all_links).map_err(|e| e.to_string())?;

    Ok(modules)
}

#[tauri::command]
fn validate_selection(selected: Vec<String>) -> Result<(), Vec<String>> {
    let patch_ids: HashSet<PatchId> = selected
        .iter()
        .filter_map(|s| match s.as_str() {
            "A" => Some(PatchId::A),
            "B" => Some(PatchId::B),
            "C" => Some(PatchId::C),
            "D" => Some(PatchId::D),
            "E" => Some(PatchId::E),
            "G" => Some(PatchId::G),
            "I" => Some(PatchId::I),
            "L" => Some(PatchId::L),
            "M" => Some(PatchId::M),
            "N" => Some(PatchId::N),
            "O" => Some(PatchId::O),
            "S" => Some(PatchId::S),
            "U" => Some(PatchId::U),
            "V" => Some(PatchId::V),
            _ => None,
        })
        .collect();

    validate_module_selection(&patch_ids)
}

#[tauri::command]
fn auto_select_deps(selected: Vec<String>) -> Vec<String> {
    let patch_ids: HashSet<PatchId> = selected
        .iter()
        .filter_map(|s| match s.as_str() {
            "A" => Some(PatchId::A),
            "B" => Some(PatchId::B),
            "C" => Some(PatchId::C),
            "D" => Some(PatchId::D),
            "E" => Some(PatchId::E),
            "G" => Some(PatchId::G),
            "I" => Some(PatchId::I),
            "L" => Some(PatchId::L),
            "M" => Some(PatchId::M),
            "N" => Some(PatchId::N),
            "O" => Some(PatchId::O),
            "S" => Some(PatchId::S),
            "U" => Some(PatchId::U),
            "V" => Some(PatchId::V),
            _ => None,
        })
        .collect();

    let with_deps = auto_select_dependencies(&patch_ids);

    with_deps
        .iter()
        .map(|id| format!("{:?}", id))
        .collect()
}

#[tauri::command]
fn get_forum_url() -> String {
    FORUM_URL.to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .invoke_handler(tauri::generate_handler![
            fetch_patches,
            validate_selection,
            auto_select_deps,
            get_forum_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```
  </action>
  <verify>
Run `cargo test` in src-tauri - all tests should pass (forum, links, dependencies, modules).
Run `cargo check` - should compile without errors.
  </verify>
  <done>
Dependency validation implemented with B/D/E group rule, L->A, U->A+G, O->S dependencies. Tauri commands exposed for frontend: fetch_patches, validate_selection, auto_select_deps, get_forum_url.
  </done>
</task>

</tasks>

<verification>
After all tasks complete:

1. Run full test suite: `cd "/home/user/Programs/wow patcher/src-tauri" && cargo test`
   - All parser tests should pass
   - All dependency validation tests should pass
   - All link extraction tests should pass

2. Run `npm run tauri dev` and test from browser console:
   ```js
   // In Tauri app dev tools console:
   await window.__TAURI__.core.invoke('get_forum_url')
   // Should return: "https://forum.turtlecraft.gg/viewtopic.php?t=21355"

   await window.__TAURI__.core.invoke('validate_selection', { selected: ['L'] })
   // Should return error about missing Patch A

   await window.__TAURI__.core.invoke('auto_select_deps', { selected: ['L'] })
   // Should return ['A', 'L']
   ```

3. Test live forum fetch (requires internet):
   ```js
   await window.__TAURI__.core.invoke('fetch_patches')
   // Should return array of patch modules (may have empty downloads if forum structure changed)
   ```
</verification>

<success_criteria>
- Forum HTML fetching works with fallback to alternate URL
- HTML parsing extracts post content using phpBB selectors
- Download link extraction identifies Mediafire and Google Drive URLs
- Dependency validation correctly enforces all rules (B/D/E, L->A, U->A+G, O->S)
- All unit tests pass
- Tauri commands callable from frontend
- Live fetch returns patch module data
</success_criteria>

<output>
After completion, create `.planning/phases/1-foundation-forum-parser/1-02-SUMMARY.md`
</output>
