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
