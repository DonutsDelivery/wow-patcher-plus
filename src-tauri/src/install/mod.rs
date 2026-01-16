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
pub mod verifier;

// Re-exports
pub use copier::{install_mpq, InstallEvent, InstallError, get_mpq_filename};
pub use detector::WowDetector;
pub use settings::{Settings, SettingsError};
pub use verifier::{verify_patch, verify_all, VerifyResult, is_patch_installed, get_installed_patches};
