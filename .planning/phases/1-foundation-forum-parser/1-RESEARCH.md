# Phase 1: Foundation & Forum Parser - Research

**Researched:** 2026-01-16
**Domain:** Web scraping, HTML parsing, Rust/Tauri integration
**Confidence:** MEDIUM (forum structure analysis based on phpBB patterns and web search; direct forum access unavailable)

## Summary

This research investigates how to parse the Turtle WoW HD Patch: Reforged forum post to discover available patches, download links, and module dependencies. The forum uses phpBB software with a standard HTML structure. The HD Patch system consists of multiple modules (A through V) with specific dependency relationships.

The standard approach is to use Tauri's HTTP plugin (which re-exports reqwest) for fetching the forum page, combined with the `scraper` crate for HTML parsing using CSS selectors. The forum post is publicly accessible (no authentication required), and the content structure follows phpBB's standard `div.postbody > div.content` pattern.

**Primary recommendation:** Use Tauri's HTTP plugin for requests and the `scraper` crate for HTML parsing, targeting phpBB's `div.content` class for post body extraction. Store module data in strongly-typed Rust structs with serde for serialization.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tauri-plugin-http | 2.x | HTTP requests with permission system | Official Tauri plugin, re-exports reqwest |
| scraper | 0.25.0 | HTML parsing with CSS selectors | Uses Servo's html5ever, browser-grade parsing |
| tokio | 1.x | Async runtime | Required by reqwest, Tauri standard |
| serde | 1.x | Serialization/deserialization | Rust standard for data structures |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| regex | 1.x | Pattern matching for URLs | Extract Google Drive/Mediafire file IDs |
| url | 2.x | URL parsing and normalization | Validate and normalize download links |
| thiserror | 1.x | Error type definitions | Custom error handling |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| scraper | select.rs | select.rs has jQuery-style syntax but scraper has better maintained Servo integration |
| scraper | html5ever directly | Lower level, more control but more code |
| tauri-plugin-http | raw reqwest | Would work but lose Tauri permission integration |

**Installation (Cargo.toml):**
```toml
[dependencies]
tauri-plugin-http = "2"
scraper = "0.25"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
regex = "1"
url = "2"
thiserror = "1"
```

## Architecture Patterns

### Recommended Project Structure
```
src-tauri/
├── src/
│   ├── lib.rs              # Tauri plugin initialization
│   ├── parser/
│   │   ├── mod.rs          # Parser module exports
│   │   ├── forum.rs        # Forum fetching and HTML parsing
│   │   ├── modules.rs      # Patch module definitions
│   │   ├── links.rs        # Download link extraction
│   │   └── dependencies.rs # Dependency resolution
│   └── models/
│       ├── mod.rs          # Model exports
│       ├── patch.rs        # Patch/Module data structures
│       └── download.rs     # Download link structures
```

### Pattern 1: Data Models for Patches
**What:** Strongly-typed Rust structs representing patch modules
**When to use:** Always - type safety is critical for dependency validation
**Example:**
```rust
// Source: Derived from forum post analysis
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchModule {
    pub id: PatchId,
    pub name: String,
    pub description: String,
    pub downloads: Vec<DownloadLink>,
    pub dependencies: Vec<PatchId>,
    pub file_size: Option<String>,
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatchId {
    A, // Player Characters & NPCs
    B, // Buildings (requires D, E)
    C, // Creatures
    D, // Doodads (requires B, E)
    E, // Environment (requires B, D)
    G, // Gear & Weapons
    I, // Interface
    L, // A Little Extra for Females (requires A)
    M, // Maps & Loading Screens
    N, // Darker Nights
    O, // Raid Visuals Mod (requires S)
    S, // Sounds & Music
    U, // Ultra HD (requires A, G)
    V, // Visual Effects for Spells
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadLink {
    pub provider: DownloadProvider,
    pub url: String,
    pub file_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadProvider {
    Mediafire,
    GoogleDrive,
    Unknown,
}
```

### Pattern 2: CSS Selector-Based Parsing
**What:** Use CSS selectors to navigate phpBB HTML structure
**When to use:** For extracting content from forum posts
**Example:**
```rust
// Source: https://docs.rs/scraper
use scraper::{Html, Selector};

pub fn parse_forum_post(html: &str) -> Result<Vec<PatchModule>, ParserError> {
    let document = Html::parse_document(html);

    // phpBB post content is in div.postbody > div.content
    let content_selector = Selector::parse("div.postbody div.content")
        .map_err(|_| ParserError::SelectorParse)?;

    // Get the first post (main HD Patch info)
    let first_post = document
        .select(&content_selector)
        .next()
        .ok_or(ParserError::PostNotFound)?;

    // Extract text and links
    let post_html = first_post.inner_html();

    // Parse modules from post content
    parse_modules_from_content(&post_html)
}
```

### Pattern 3: Regex for Download Link Extraction
**What:** Use regex patterns to identify and extract download URLs
**When to use:** Extracting Google Drive and Mediafire links from post content
**Example:**
```rust
// Source: Community patterns for drive/mediafire URLs
use regex::Regex;

lazy_static::lazy_static! {
    // Google Drive patterns
    static ref GDRIVE_FILE: Regex = Regex::new(
        r"https://drive\.google\.com/file/d/([A-Za-z0-9_-]+)"
    ).unwrap();

    static ref GDRIVE_OPEN: Regex = Regex::new(
        r"https://drive\.google\.com/open\?id=([A-Za-z0-9_-]+)"
    ).unwrap();

    // Mediafire patterns
    static ref MEDIAFIRE_FILE: Regex = Regex::new(
        r"https://www\.mediafire\.com/file/([a-z0-9]+)/([^/\s\"]+)"
    ).unwrap();

    static ref MEDIAFIRE_FOLDER: Regex = Regex::new(
        r"https://(?:www\.)?(?:app\.)?mediafire\.com/folder/([a-z0-9]+)"
    ).unwrap();
}

pub fn extract_download_links(content: &str) -> Vec<DownloadLink> {
    let mut links = Vec::new();

    // Extract Google Drive links
    for cap in GDRIVE_FILE.captures_iter(content) {
        links.push(DownloadLink {
            provider: DownloadProvider::GoogleDrive,
            url: cap[0].to_string(),
            file_name: None,
        });
    }

    // Extract Mediafire links
    for cap in MEDIAFIRE_FILE.captures_iter(content) {
        links.push(DownloadLink {
            provider: DownloadProvider::Mediafire,
            url: cap[0].to_string(),
            file_name: Some(cap[2].to_string()),
        });
    }

    links
}
```

### Anti-Patterns to Avoid
- **String matching for HTML:** Never use string splitting or contains() for HTML parsing - use proper DOM parsing
- **Hardcoded URLs:** Store the forum URL in configuration, not hardcoded, as the domain has changed (turtle-wow.org -> turtlecraft.gg)
- **Synchronous requests in UI thread:** Always use async/await for HTTP requests
- **Ignoring rate limits:** Add delays between requests if scraping multiple pages

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTML parsing | String manipulation | scraper crate | Edge cases with malformed HTML, encodings |
| HTTP requests | std::net | reqwest via tauri-plugin-http | Connection pooling, TLS, redirects |
| URL validation | String checks | url crate | RFC compliance, encoding handling |
| CSS selectors | Manual DOM traversal | scraper's Selector | Browser-grade selector parsing |
| User-Agent rotation | Random strings | curated list | Realistic browser signatures matter |

**Key insight:** Forum HTML is messy and inconsistent. Browser-grade parsing (html5ever) handles edge cases that naive parsing will miss.

## Common Pitfalls

### Pitfall 1: Forum URL Domain Changes
**What goes wrong:** The forum has moved between domains (turtle-wow.org, turtlecraft.gg)
**Why it happens:** Private server communities occasionally rebrand or move domains
**How to avoid:** Store base URL in configuration, allow user override, handle redirects
**Warning signs:** 301/302 redirects, connection failures to hardcoded URLs

### Pitfall 2: Post Content Updates Breaking Parser
**What goes wrong:** Forum post author updates formatting, breaking CSS selectors or regex
**Why it happens:** The HD Patch is actively maintained, post is updated frequently (last: Dec 2025)
**How to avoid:**
- Use robust, general selectors (div.content not specific class chains)
- Add fallback parsing strategies
- Log parsing failures with HTML context for debugging
- Version the expected format and warn on changes
**Warning signs:** Parsing returns empty results, missing modules

### Pitfall 3: Download Link Expiration
**What goes wrong:** Google Drive and Mediafire links become invalid
**Why it happens:** Files are re-uploaded with new IDs, storage quotas exceeded
**How to avoid:**
- Don't cache download URLs for long periods
- Validate links before presenting to user
- Parse fresh on each use
**Warning signs:** 404 errors when accessing download links

### Pitfall 4: Module Dependency Ordering
**What goes wrong:** User installs incompatible module combinations
**Why it happens:** Complex dependency rules (B+D+E together, L needs A, U needs A+G)
**How to avoid:**
- Model dependencies as a graph
- Validate selections before download
- Auto-select dependencies when user picks a module
**Warning signs:** Crash reports mentioning specific module combinations

### Pitfall 5: phpBB Session/Cookie Requirements
**What goes wrong:** Some phpBB boards require cookies even for public posts
**Why it happens:** Bot protection, session management
**How to avoid:**
- Test with clean requests first
- If needed, implement cookie jar
- Set realistic User-Agent header
**Warning signs:** Redirects to login, different content than browser shows

## Code Examples

Verified patterns from official sources:

### Fetching Forum Page with Tauri HTTP Plugin
```rust
// Source: https://v2.tauri.app/plugin/http-client/
use tauri_plugin_http::reqwest;

pub async fn fetch_forum_post(url: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;

    let response = client
        .get(url)
        .send()
        .await?;

    response.text().await
}
```

### Complete Parsing Flow
```rust
// Source: https://docs.rs/scraper/0.25.0/scraper/
use scraper::{Html, Selector, ElementRef};

pub struct ForumParser {
    content_selector: Selector,
    link_selector: Selector,
}

impl ForumParser {
    pub fn new() -> Self {
        Self {
            content_selector: Selector::parse("div.postbody div.content").unwrap(),
            link_selector: Selector::parse("a[href]").unwrap(),
        }
    }

    pub fn parse(&self, html: &str) -> ParsedForumPost {
        let document = Html::parse_document(html);

        // Get first post content
        let content = document
            .select(&self.content_selector)
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();

        // Extract all links
        let links: Vec<String> = document
            .select(&self.link_selector)
            .filter_map(|el| el.value().attr("href"))
            .map(String::from)
            .collect();

        ParsedForumPost { content, links }
    }
}
```

### Dependency Validation
```rust
// Source: Derived from forum post dependency rules
use std::collections::HashSet;

pub fn validate_module_selection(selected: &HashSet<PatchId>) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // B, D, E must be together or none
    let bde_group = [PatchId::B, PatchId::D, PatchId::E];
    let bde_selected: Vec<_> = bde_group.iter().filter(|p| selected.contains(p)).collect();
    if !bde_selected.is_empty() && bde_selected.len() != 3 {
        errors.push("Patches B, D, and E must be installed together".into());
    }

    // L requires A
    if selected.contains(&PatchId::L) && !selected.contains(&PatchId::A) {
        errors.push("Patch L requires Patch A".into());
    }

    // U requires A and G
    if selected.contains(&PatchId::U) {
        if !selected.contains(&PatchId::A) {
            errors.push("Patch U requires Patch A".into());
        }
        if !selected.contains(&PatchId::G) {
            errors.push("Patch U requires Patch G".into());
        }
    }

    // O requires S
    if selected.contains(&PatchId::O) && !selected.contains(&PatchId::S) {
        errors.push("Patch O requires Patch S".into());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
```

## HD Patch Module Reference

Based on forum research, here are the known modules:

| Module | Name | Description | Dependencies |
|--------|------|-------------|--------------|
| A | Player Characters & NPCs | HD models and textures for players/NPCs | None |
| B | Buildings | HD building models | Must install with D, E |
| C | Creatures | HD creature models | None |
| D | Doodads | HD doodad models | Must install with B, E |
| E | Environment | HD environment textures | Must install with B, D |
| G | Gear & Weapons | HD equipment models | None |
| I | Interface | HD UI elements | None |
| L | A Little Extra for Females | Additional female model options | Requires A |
| M | Maps & Loading Screens | HD loading screens | None |
| N | Darker Nights | Night ambiance enhancement | None |
| O | Raid Visuals Mod | Enhanced raid effects | Requires S |
| S | Sounds & Music | HD audio | None |
| U | Ultra HD | 4K character textures | Requires A, G |
| V | Visual Effects | HD spell effects | None |

**Note:** VanillaHelpers mod is a mandatory prerequisite for all HD patches (external to the module system).

## Forum Post Details

| Property | Value |
|----------|-------|
| Primary URL | https://forum.turtlecraft.gg/viewtopic.php?t=21355 |
| Alternate URL | https://forum.turtle-wow.org/viewtopic.php?t=21355 |
| Thread ID | 21355 |
| Last Updated | December 22, 2025 |
| Download Folder | https://app.mediafire.com/folder/gzjmrha0cbjn9 |

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| turtle-wow.org domain | turtlecraft.gg domain | Late 2025 | URLs need updating |
| Manual downloads | HD Patch Manager auto-updater | 2025 | Existing tool to reference |
| Single thread | Multi-page thread with updates | Ongoing | First post has current info |

**Deprecated/outdated:**
- Old forum domain (turtle-wow.org) may still redirect but turtlecraft.gg is current
- Previous versions of patches (pre-Reforged) are superseded

## Open Questions

Things that couldn't be fully resolved:

1. **Exact HTML structure of current forum post**
   - What we know: phpBB uses div.postbody > div.content pattern
   - What's unclear: Exact formatting of module listings, whether spoiler tags are used
   - Recommendation: Implement adaptive parsing, test against actual forum response

2. **Rate limiting on forum**
   - What we know: No explicit rate limit documentation found
   - What's unclear: Whether aggressive scraping is blocked
   - Recommendation: Add configurable delay between requests, use caching

3. **Mediafire folder structure**
   - What we know: Folder URL exists, contains patch files
   - What's unclear: How files are organized, naming conventions
   - Recommendation: May need to also parse Mediafire folder page or use API

4. **Google Drive link formats used**
   - What we know: Multiple URL patterns exist for GDrive
   - What's unclear: Which formats the forum post uses
   - Recommendation: Support all common patterns in regex

## Existing Tools Reference

An existing tool exists: [Wow-HD-Patch-Manager](https://github.com/Thx4riposte/Wow-HD-Patch-Manager) by Thx4riposte. This could be referenced for:
- Understanding expected behavior
- Known issues to avoid
- User expectations

However, examining its implementation was limited as the repo structure suggests a simple utility. Our Tauri-based approach will be more comprehensive.

## Sources

### Primary (HIGH confidence)
- [Tauri HTTP Plugin](https://v2.tauri.app/plugin/http-client/) - Official documentation
- [scraper crate](https://docs.rs/scraper/latest/scraper/) - v0.25.0 documentation
- [reqwest crate](https://docs.rs/reqwest/latest/reqwest/) - v0.13.1 documentation

### Secondary (MEDIUM confidence)
- [Turtle WoW Forum Thread](https://forum.turtlecraft.gg/viewtopic.php?t=21355) - Module list and dependencies (via WebSearch)
- [phpBB Post Structure](https://www.phpbb.com/community/viewtopic.php?t=2137844) - HTML structure patterns
- [Rust Cookbook - Web Scraping](https://rust-lang-nursery.github.io/rust-cookbook/web/scraping.html) - Link extraction patterns

### Tertiary (LOW confidence - needs validation)
- phpBB CSS class names (div.postbody, div.content) - Based on general phpBB documentation, specific forum may vary
- Download link regex patterns - Community patterns, need testing against actual links
- Module list completeness - Based on WebSearch results, may be incomplete

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Official Tauri and crates.io documentation
- Architecture: MEDIUM - Based on Rust best practices and Tauri patterns
- Module data: MEDIUM - Forum research via WebSearch, not direct access
- Pitfalls: MEDIUM - Based on common web scraping issues and forum posts

**Research date:** 2026-01-16
**Valid until:** 2026-02-16 (30 days - stable technology, active forum)

**Key limitation:** Direct forum access was unavailable during research. Parser implementation should include robust error handling and logging to debug any structural differences from expected phpBB patterns.
