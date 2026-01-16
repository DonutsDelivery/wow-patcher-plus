//! Installation engine for HD patch files
//!
//! This module provides:
//! - WoW folder detection and validation
//! - Settings persistence for WoW path
//! - MPQ file copy operations (added in later plans)

pub mod detector;
pub mod settings;

// Re-exports
pub use detector::WowDetector;
pub use settings::{Settings, SettingsError};
