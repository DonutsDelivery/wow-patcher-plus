pub mod patch;
pub mod download;

pub use patch::{PatchModule, PatchId, PatchGroup};
pub use download::{DownloadLink, DownloadProvider};

/// Parsed forum post content
#[derive(Debug)]
pub struct ParsedForumPost {
    pub content: String,
    pub links: Vec<String>,
}

/// Parser error types
#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("Failed to parse CSS selector: {0}")]
    SelectorParse(String),

    #[error("Forum post not found")]
    PostNotFound,

    #[error("HTTP request failed: {0}")]
    HttpError(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}
