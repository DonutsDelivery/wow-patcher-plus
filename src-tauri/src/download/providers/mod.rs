//! Download provider abstraction
//!
//! This module defines the trait that download providers (Google Drive, Mediafire, etc.)
//! must implement to resolve share URLs to direct download URLs.

pub mod gdrive;
pub mod mediafire;
pub mod dropbox;
pub mod transfer;
pub mod mega;

pub use gdrive::GoogleDriveProvider;
pub use mediafire::MediafireProvider;
pub use dropbox::DropboxProvider;
pub use transfer::TransferProvider;
pub use mega::MegaProvider;

use async_trait::async_trait;
use crate::download::DownloadError;

/// Information about a resolved direct download URL
#[derive(Debug, Clone)]
pub struct DirectDownloadInfo {
    /// The resolved direct download URL
    pub url: String,
    /// File name extracted from headers or URL (if available)
    pub file_name: Option<String>,
    /// Total file size in bytes (if known from headers)
    pub content_length: Option<u64>,
    /// Whether the server supports Range headers for resume
    pub supports_range: bool,
}

/// Trait for download providers that resolve share URLs to direct download URLs
///
/// Different hosting providers (Google Drive, Mediafire) have different URL formats
/// and may require fetching intermediate pages or handling confirmation dialogs.
#[async_trait]
pub trait DownloadProvider: Send + Sync {
    /// Resolve a share/public URL to a direct download URL
    ///
    /// # Arguments
    /// * `share_url` - The public share URL (e.g., drive.google.com/file/d/...)
    ///
    /// # Returns
    /// Information about the direct download including URL and metadata
    async fn resolve_direct_url(&self, share_url: &str) -> Result<DirectDownloadInfo, DownloadError>;

    /// Check if this provider supports resumable downloads (Range headers)
    fn supports_resume(&self) -> bool;

    /// Get the provider's name for logging/display purposes
    fn name(&self) -> &'static str;
}
