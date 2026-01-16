//! Installation verification for patch files
//!
//! Verifies installed patches by checking:
//! - File existence in WoW Data folder
//! - File size matches downloaded version (if download exists)

use std::path::{Path, PathBuf};
use tokio::fs;
use serde::Serialize;

use super::copier::get_mpq_filename;

/// Result of verifying a single patch installation
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum VerifyResult {
    /// Patch is not installed
    NotInstalled,

    /// Patch is installed but download is missing (cannot verify size)
    Installed { verified: bool },

    /// Patch installed but size doesn't match download
    SizeMismatch {
        installed_size: u64,
        expected_size: u64
    },

    /// Error during verification
    Error { message: String },
}

/// Verify a single patch installation
///
/// Checks that the MPQ file exists in the Data folder and
/// optionally compares size with the downloaded file.
pub async fn verify_patch(
    patch_id: &str,
    data_folder: &Path,
    downloads_folder: &Path,
) -> VerifyResult {
    let mpq_name = get_mpq_filename(patch_id);
    let installed_path = data_folder.join(&mpq_name);
    let download_path = downloads_folder.join(&mpq_name);

    // Check if installed
    if !installed_path.exists() {
        return VerifyResult::NotInstalled;
    }

    // Get installed file size
    let installed_size = match fs::metadata(&installed_path).await {
        Ok(m) => m.len(),
        Err(e) => return VerifyResult::Error {
            message: format!("Failed to read installed file: {}", e)
        },
    };

    // Check if download exists for comparison
    if !download_path.exists() {
        // Can't verify size, but file exists
        return VerifyResult::Installed { verified: false };
    }

    // Compare sizes
    let download_size = match fs::metadata(&download_path).await {
        Ok(m) => m.len(),
        Err(e) => return VerifyResult::Error {
            message: format!("Failed to read download file: {}", e)
        },
    };

    if installed_size != download_size {
        return VerifyResult::SizeMismatch {
            installed_size,
            expected_size: download_size,
        };
    }

    VerifyResult::Installed { verified: true }
}

/// Verify multiple patches at once
///
/// Returns a vector of (patch_id, result) tuples.
pub async fn verify_all(
    patch_ids: &[&str],
    data_folder: &Path,
    downloads_folder: &Path,
) -> Vec<(String, VerifyResult)> {
    let mut results = Vec::with_capacity(patch_ids.len());

    for id in patch_ids {
        let result = verify_patch(id, data_folder, downloads_folder).await;
        results.push((id.to_string(), result));
    }

    results
}

/// Check if a specific patch is installed (quick existence check)
pub async fn is_patch_installed(patch_id: &str, data_folder: &Path) -> bool {
    let mpq_name = get_mpq_filename(patch_id);
    let installed_path = data_folder.join(&mpq_name);
    installed_path.exists()
}

/// Get paths for installed patches
///
/// Returns only patches that exist in the Data folder.
pub async fn get_installed_patches(
    patch_ids: &[&str],
    data_folder: &Path,
) -> Vec<PathBuf> {
    let mut installed = Vec::new();

    for id in patch_ids {
        let mpq_name = get_mpq_filename(id);
        let path = data_folder.join(&mpq_name);
        if path.exists() {
            installed.push(path);
        }
    }

    installed
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::io::AsyncWriteExt;

    /// Helper to create a test file with content
    async fn create_test_file(path: &Path, content: &[u8]) {
        let mut f = fs::File::create(path).await.unwrap();
        f.write_all(content).await.unwrap();
    }

    #[tokio::test]
    async fn test_verify_not_installed() {
        let data_dir = tempdir().unwrap();
        let downloads_dir = tempdir().unwrap();

        let result = verify_patch("A", data_dir.path(), downloads_dir.path()).await;
        assert!(matches!(result, VerifyResult::NotInstalled));
    }

    #[tokio::test]
    async fn test_verify_installed_no_download() {
        let data_dir = tempdir().unwrap();
        let downloads_dir = tempdir().unwrap();

        // Create installed file
        let installed = data_dir.path().join("Patch-A.mpq");
        create_test_file(&installed, b"test content").await;

        let result = verify_patch("A", data_dir.path(), downloads_dir.path()).await;
        assert!(matches!(result, VerifyResult::Installed { verified: false }));
    }

    #[tokio::test]
    async fn test_verify_installed_size_match() {
        let data_dir = tempdir().unwrap();
        let downloads_dir = tempdir().unwrap();
        let content = b"test content for matching";

        // Create installed file
        let installed = data_dir.path().join("Patch-B.mpq");
        create_test_file(&installed, content).await;

        // Create download with same content
        let download = downloads_dir.path().join("Patch-B.mpq");
        create_test_file(&download, content).await;

        let result = verify_patch("B", data_dir.path(), downloads_dir.path()).await;
        assert!(matches!(result, VerifyResult::Installed { verified: true }));
    }

    #[tokio::test]
    async fn test_verify_size_mismatch() {
        let data_dir = tempdir().unwrap();
        let downloads_dir = tempdir().unwrap();

        // Create installed file
        let installed = data_dir.path().join("Patch-C.mpq");
        create_test_file(&installed, b"short").await;

        // Create download with different size
        let download = downloads_dir.path().join("Patch-C.mpq");
        create_test_file(&download, b"much longer content here").await;

        let result = verify_patch("C", data_dir.path(), downloads_dir.path()).await;
        assert!(matches!(result, VerifyResult::SizeMismatch { .. }));
    }

    #[tokio::test]
    async fn test_verify_all() {
        let data_dir = tempdir().unwrap();
        let downloads_dir = tempdir().unwrap();

        // Create one installed file
        let installed = data_dir.path().join("Patch-A.mpq");
        create_test_file(&installed, b"test").await;

        let results = verify_all(&["A", "B"], data_dir.path(), downloads_dir.path()).await;

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "A");
        assert!(matches!(results[0].1, VerifyResult::Installed { .. }));
        assert_eq!(results[1].0, "B");
        assert!(matches!(results[1].1, VerifyResult::NotInstalled));
    }

    #[tokio::test]
    async fn test_is_patch_installed() {
        let data_dir = tempdir().unwrap();

        // Not installed
        assert!(!is_patch_installed("A", data_dir.path()).await);

        // Create installed file
        let installed = data_dir.path().join("Patch-A.mpq");
        create_test_file(&installed, b"test").await;

        // Now installed
        assert!(is_patch_installed("A", data_dir.path()).await);
    }

    #[tokio::test]
    async fn test_get_installed_patches() {
        let data_dir = tempdir().unwrap();

        // Create some installed files
        let installed_a = data_dir.path().join("Patch-A.mpq");
        create_test_file(&installed_a, b"test").await;

        let installed_c = data_dir.path().join("Patch-C.mpq");
        create_test_file(&installed_c, b"test").await;

        let installed = get_installed_patches(&["A", "B", "C"], data_dir.path()).await;

        assert_eq!(installed.len(), 2);
        assert!(installed.iter().any(|p| p.ends_with("Patch-A.mpq")));
        assert!(installed.iter().any(|p| p.ends_with("Patch-C.mpq")));
    }
}
