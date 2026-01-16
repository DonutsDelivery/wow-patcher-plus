//! Installation engine for HD patch files
//!
//! This module provides:
//! - WoW folder detection and validation
//! - Settings persistence for WoW path
//! - MPQ file copy operations with progress events
//! - Installation verification

pub mod copier;
pub mod detector;
pub mod settings;

// Re-exports
pub use copier::{install_mpq, InstallEvent, InstallError, get_mpq_filename};
pub use detector::WowDetector;
pub use settings::{Settings, SettingsError};
