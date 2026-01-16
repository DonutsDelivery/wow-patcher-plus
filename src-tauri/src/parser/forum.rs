//! Forum fetching and HTML parsing

use crate::models::{ParsedForumPost, ParserError};

pub struct ForumParser {
    // Will hold selectors
}

impl ForumParser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse(&self, _html: &str) -> Result<ParsedForumPost, ParserError> {
        // TODO: Implement in Plan 02
        todo!("Forum parsing not yet implemented")
    }
}

impl Default for ForumParser {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn fetch_forum_post(_url: &str) -> Result<String, ParserError> {
    // TODO: Implement in Plan 02
    todo!("Forum fetching not yet implemented")
}
