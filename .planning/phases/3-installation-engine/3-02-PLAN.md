---
phase: 3-installation-engine
plan: 02
type: execute
wave: 2
depends_on: ["3-01"]
files_modified:
  - src-tauri/src/install/mod.rs
  - src-tauri/src/install/copier.rs
  - src-tauri/src/install/verifier.rs
autonomous: true
user_setup: []

must_haves:
  truths:
    - "App can copy MPQ files from downloads to WoW Data folder"
    - "App reports progress during file copy operations"
    - "App can verify installed patches exist with correct file size"
  artifacts:
    - path: "src-tauri/src/install/copier.rs"
      provides: "MPQ file copy with progress events"
      exports: ["install_mpq", "InstallEvent"]
    - path: "src-tauri/src/install/verifier.rs"
      provides: "Installation verification logic"
      exports: ["verify_patch", "verify_all", "VerifyResult"]
  key_links:
    - from: "src-tauri/src/install/copier.rs"
      to: "tokio::fs"
      via: "async file operations"
      pattern: "tokio::fs::(copy|metadata)"
    - from: "src-tauri/src/install/verifier.rs"
      to: "src-tauri/src/install/copier.rs"
      via: "uses same path conventions"
      pattern: "Patch-.*\\.mpq"
---

<objective>
Implement MPQ file copying with progress events and installation verification.

Purpose: Enable the app to copy downloaded MPQ files to the WoW Data folder and verify that installed patches have correct file sizes.

Output:
- copier.rs with install_mpq function and InstallEvent enum
- verifier.rs with verify_patch and verify_all functions
</objective>

<execution_context>
@./.claude/get-shit-done/workflows/execute-plan.md
@./.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/3-installation-engine/3-RESEARCH.md
@.planning/phases/3-installation-engine/3-01-SUMMARY.md

# Existing download progress pattern to follow
@src-tauri/src/download/progress.rs
@src-tauri/src/install/mod.rs
@src-tauri/src/install/detector.rs
</context>

<tasks>

<task type="auto">
  <name>Task 1: Implement MPQ file copier with progress events</name>
  <files>
    - src-tauri/src/install/copier.rs
    - src-tauri/src/install/mod.rs
  </files>
  <action>
    Create copier.rs with InstallEvent enum and install_mpq function:

    ```rust
    //! MPQ file copy operations with progress reporting
    //!
    //! HD Patch: Reforged distributes raw MPQ files (not archives),
    //! so installation is simply copying files to the WoW Data folder.

    use std::path::{Path, PathBuf};
    use tokio::fs;
    use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
    use tauri::ipc::Channel;
    use thiserror::Error;

    /// Buffer size for chunked copy (64KB)
    const COPY_BUFFER_SIZE: usize = 64 * 1024;

    /// Progress throttle interval in milliseconds
    const PROGRESS_THROTTLE_MS: u64 = 100;

    #[derive(Debug, Error)]
    pub enum InstallError {
        #[error("WoW path not configured")]
        WowPathNotSet,

        #[error("Invalid WoW installation folder")]
        InvalidWowFolder,

        #[error("Download not found: {0}")]
        DownloadNotFound(String),

        #[error("Invalid source path")]
        InvalidPath,

        #[error("I/O error: {0}")]
        IoError(#[from] std::io::Error),
    }

    /// Events emitted during installation
    #[derive(Clone, serde::Serialize)]
    #[serde(rename_all = "camelCase", tag = "event", content = "data")]
    pub enum InstallEvent {
        Started {
            patch_id: String,
            file_name: String
        },
        Progress {
            patch_id: String,
            bytes_copied: u64,
            total_bytes: u64
        },
        Completed {
            patch_id: String,
            dest_path: String
        },
        Failed {
            patch_id: String,
            error: String
        },
    }

    /// Install (copy) an MPQ file to the WoW Data folder
    ///
    /// Uses chunked copy with progress reporting for large files.
    /// Overwrites existing files if present.
    pub async fn install_mpq(
        source_path: &Path,
        data_folder: &Path,
        patch_id: String,
        on_event: Channel<InstallEvent>,
    ) -> Result<PathBuf, InstallError> {
        let file_name = source_path
            .file_name()
            .ok_or(InstallError::InvalidPath)?
            .to_string_lossy()
            .to_string();

        let dest_path = data_folder.join(&file_name);

        // Send started event
        let _ = on_event.send(InstallEvent::Started {
            patch_id: patch_id.clone(),
            file_name: file_name.clone(),
        });

        // Get file size for progress tracking
        let metadata = fs::metadata(source_path).await?;
        let total_bytes = metadata.len();

        // Perform chunked copy with progress
        let result = copy_with_progress(
            source_path,
            &dest_path,
            total_bytes,
            patch_id.clone(),
            on_event.clone(),
        ).await;

        match result {
            Ok(_) => {
                let _ = on_event.send(InstallEvent::Completed {
                    patch_id,
                    dest_path: dest_path.to_string_lossy().to_string(),
                });
                Ok(dest_path)
            }
            Err(e) => {
                let _ = on_event.send(InstallEvent::Failed {
                    patch_id,
                    error: e.to_string(),
                });
                Err(e)
            }
        }
    }

    /// Copy file with chunked reads and progress events
    async fn copy_with_progress(
        source: &Path,
        dest: &Path,
        total_bytes: u64,
        patch_id: String,
        on_event: Channel<InstallEvent>,
    ) -> Result<(), InstallError> {
        let source_file = fs::File::open(source).await?;
        let dest_file = fs::File::create(dest).await?;

        let mut reader = BufReader::new(source_file);
        let mut writer = BufWriter::new(dest_file);
        let mut buffer = vec![0u8; COPY_BUFFER_SIZE];
        let mut bytes_copied: u64 = 0;
        let mut last_progress = std::time::Instant::now();

        loop {
            let bytes_read = reader.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }

            writer.write_all(&buffer[..bytes_read]).await?;
            bytes_copied += bytes_read as u64;

            // Throttled progress events
            if last_progress.elapsed().as_millis() >= PROGRESS_THROTTLE_MS as u128 {
                let _ = on_event.send(InstallEvent::Progress {
                    patch_id: patch_id.clone(),
                    bytes_copied,
                    total_bytes,
                });
                last_progress = std::time::Instant::now();
            }
        }

        writer.flush().await?;

        // Final progress event
        let _ = on_event.send(InstallEvent::Progress {
            patch_id,
            bytes_copied,
            total_bytes,
        });

        Ok(())
    }

    /// Get expected MPQ filename for a patch ID
    pub fn get_mpq_filename(patch_id: &str) -> String {
        format!("Patch-{}.mpq", patch_id.to_uppercase())
    }
    ```

    Update install/mod.rs to add copier module and exports:
    ```rust
    pub mod copier;
    pub use copier::{install_mpq, InstallEvent, InstallError, get_mpq_filename};
    ```
  </action>
  <verify>
    `cargo check -p turtle-wow-hd-patcher` compiles without errors
  </verify>
  <done>
    install_mpq copies files with chunked reads and throttled progress events (100ms)
  </done>
</task>

<task type="auto">
  <name>Task 2: Implement installation verification</name>
  <files>
    - src-tauri/src/install/verifier.rs
    - src-tauri/src/install/mod.rs
  </files>
  <action>
    Create verifier.rs with verification logic:

    ```rust
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
    ```

    Update install/mod.rs to add verifier module and exports:
    ```rust
    pub mod verifier;
    pub use verifier::{verify_patch, verify_all, VerifyResult, is_patch_installed, get_installed_patches};
    ```

    Add unit tests for verifier in verifier.rs:
    ```rust
    #[cfg(test)]
    mod tests {
        use super::*;
        use tempfile::tempdir;
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;

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
            let mut f = File::create(&installed).await.unwrap();
            f.write_all(b"test content").await.unwrap();

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
            let mut f = File::create(&installed).await.unwrap();
            f.write_all(content).await.unwrap();

            // Create download with same content
            let download = downloads_dir.path().join("Patch-B.mpq");
            let mut f = File::create(&download).await.unwrap();
            f.write_all(content).await.unwrap();

            let result = verify_patch("B", data_dir.path(), downloads_dir.path()).await;
            assert!(matches!(result, VerifyResult::Installed { verified: true }));
        }

        #[tokio::test]
        async fn test_verify_size_mismatch() {
            let data_dir = tempdir().unwrap();
            let downloads_dir = tempdir().unwrap();

            // Create installed file
            let installed = data_dir.path().join("Patch-C.mpq");
            let mut f = File::create(&installed).await.unwrap();
            f.write_all(b"short").await.unwrap();

            // Create download with different size
            let download = downloads_dir.path().join("Patch-C.mpq");
            let mut f = File::create(&download).await.unwrap();
            f.write_all(b"much longer content here").await.unwrap();

            let result = verify_patch("C", data_dir.path(), downloads_dir.path()).await;
            assert!(matches!(result, VerifyResult::SizeMismatch { .. }));
        }
    }
    ```
  </action>
  <verify>
    `cargo test -p turtle-wow-hd-patcher verifier` passes all tests
  </verify>
  <done>
    verify_patch checks file existence and size match; verify_all handles multiple patches
  </done>
</task>

</tasks>

<verification>
After all tasks complete:
1. `cargo check -p turtle-wow-hd-patcher` compiles successfully
2. `cargo test -p turtle-wow-hd-patcher` all tests pass (including new verifier tests)
3. New files exist: src-tauri/src/install/{copier.rs, verifier.rs}
4. install_mpq uses chunked copy with 100ms throttled progress events
5. verify_patch returns NotInstalled, Installed, SizeMismatch, or Error
</verification>

<success_criteria>
- install_mpq copies MPQ files with progress events to frontend
- copy_with_progress uses 64KB chunks and 100ms throttle
- verify_patch checks file existence and size match
- verify_all handles multiple patches in sequence
- All verifier tests pass with tempdir fixtures
</success_criteria>

<output>
After completion, create `.planning/phases/3-installation-engine/3-02-SUMMARY.md`
</output>
