//! Download infrastructure for patch files
//!
//! This module provides:
//! - Streaming download engine for memory-efficient large file downloads
//! - Progress tracking with throttled event emission
//! - Provider abstraction for different download hosts (Google Drive, Mediafire)

pub mod engine;
pub mod manager;
pub mod progress;
pub mod providers;
pub mod resume;

use thiserror::Error;

// Re-export key types
pub use engine::download_file;
pub use progress::{DownloadEvent, ProgressTracker};
pub use providers::{DirectDownloadInfo, DownloadProvider};
pub use resume::download_with_resume;
pub use manager::DownloadManager;

/// Errors that can occur during download operations
#[derive(Debug, Error)]
pub enum DownloadError {
    /// HTTP request returned an error status code
    #[error("HTTP error: {0}")]
    HttpError(reqwest::StatusCode),

    /// I/O error during file operations
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Error from the HTTP client
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),

    /// Provider-specific error (e.g., rate limiting, unavailable)
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// Failed to send progress event via channel
    #[error("Channel error: {0}")]
    ChannelError(String),

    /// Google Drive large file confirmation failed
    #[error("Failed to extract download confirmation token")]
    ConfirmationFailed,

    /// Mediafire direct URL could not be resolved
    #[error("Direct download URL not found in provider page")]
    DirectUrlNotFound,
}
