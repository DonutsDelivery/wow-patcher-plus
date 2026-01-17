//! Download link data structures

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadLink {
    pub provider: DownloadProvider,
    pub url: String,
    pub file_name: Option<String>,
    /// Variant name for patches with multiple options (e.g., "Regular Version", "Less Thicc Version")
    pub variant: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadProvider {
    Mediafire,
    GoogleDrive,
    Unknown,
}
