//! Download link data structures

use serde::{Deserialize, Serialize};

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
