//! WoW installation folder detection and validation

use std::path::{Path, PathBuf};

/// Detects and validates WoW installation folders
pub struct WowDetector;

impl WowDetector {
    /// Check if a path is a valid WoW installation
    ///
    /// Valid if it has:
    /// - One of: WoW.exe, WoWFoV.exe, turtle-wow.exe
    /// - A Data subdirectory
    pub fn is_valid_wow_folder(path: &Path) -> bool {
        let has_exe = path.join("WoW.exe").exists()
            || path.join("WoWFoV.exe").exists()
            || path.join("turtle-wow.exe").exists();

        let has_data = path.join("Data").exists() && path.join("Data").is_dir();

        has_exe && has_data
    }

    /// Try to auto-detect WoW folder from current directory or exe location
    pub fn auto_detect() -> Option<PathBuf> {
        // Check current working directory
        if let Ok(cwd) = std::env::current_dir() {
            if Self::is_valid_wow_folder(&cwd) {
                return Some(cwd);
            }
        }

        // Check executable directory
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                if Self::is_valid_wow_folder(exe_dir) {
                    return Some(exe_dir.to_path_buf());
                }
            }
        }

        None
    }

    /// Get the Data folder path from a WoW installation
    pub fn get_data_folder(wow_path: &Path) -> PathBuf {
        wow_path.join("Data")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_is_valid_wow_folder_returns_false_for_nonexistent_path() {
        let nonexistent = Path::new("/nonexistent/wow/path");
        assert!(!WowDetector::is_valid_wow_folder(nonexistent));
    }

    #[test]
    fn test_get_data_folder_returns_correct_path() {
        let wow_path = Path::new("/home/user/WoW");
        let data_folder = WowDetector::get_data_folder(wow_path);
        assert_eq!(data_folder, PathBuf::from("/home/user/WoW/Data"));
    }

    #[test]
    fn test_auto_detect_returns_none_in_dev_environment() {
        // In dev environment, we're not in a WoW folder
        let result = WowDetector::auto_detect();
        assert!(result.is_none());
    }
}
