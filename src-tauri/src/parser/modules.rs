//! Patch module parsing and metadata
//!
//! Note: This module is largely deprecated. Patch data is now fetched from
//! the remote patches.json file. These functions remain for potential
//! fallback/offline use.

use crate::models::{PatchId, PatchModule, ParserError, DownloadLink};

/// Get metadata for all known patch modules
/// Returns empty list - patches are now fetched from remote JSON
pub fn get_all_modules() -> Vec<PatchModule> {
    vec![]
}

/// Parse modules from forum post content
/// This is deprecated - modules come from the remote JSON now
pub fn parse_modules(_content: &str, _links: &[DownloadLink]) -> Result<Vec<PatchModule>, ParserError> {
    Ok(vec![])
}
