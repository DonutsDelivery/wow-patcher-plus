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
