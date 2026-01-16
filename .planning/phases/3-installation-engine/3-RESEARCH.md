# Phase 3: Installation Engine - Research

**Researched:** 2026-01-16
**Domain:** Archive extraction, file system operations, WoW folder detection, installation verification
**Confidence:** HIGH (Raw MPQ files confirmed, no archive extraction needed; Tauri plugins from official docs)

## Summary

This research investigates how to install HD Patch: Reforged files into the WoW DATA folder with verification and repair capabilities. The key discovery is that **HD Patch: Reforged distributes raw MPQ files directly** - not ZIP, 7z, or RAR archives. This dramatically simplifies the installation engine: downloaded files are already in their final format and just need to be copied to the DATA folder.

The installation process involves: (1) detecting/selecting the WoW installation folder, (2) copying downloaded MPQ files to the DATA folder, (3) verifying installation integrity via file presence and size checks, and (4) providing repair functionality through re-download and re-copy operations.

For WoW folder detection, the app should look for WoW.exe in the current directory (if run from WoW folder), allow manual folder selection via tauri-plugin-dialog, and validate by checking for WoW.exe and Data subfolder existence. User preferences (WoW path) persist via tauri-plugin-store.

**Primary recommendation:** Skip archive extraction entirely - just copy downloaded MPQ files to `{wow_folder}/Data/`. Use tauri-plugin-dialog for folder selection, tauri-plugin-store for path persistence, and file size + presence checks for verification.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tauri-plugin-dialog | 2.x | WoW folder selection | Official Tauri plugin for folder picker |
| tauri-plugin-store | 2.x | Persist WoW path | Official Tauri plugin for key-value storage |
| tauri-plugin-fs | 2.x | File operations | Official Tauri plugin for cross-platform file ops |
| tokio (fs) | 1.x | Async file copy | Already in project, async file I/O |
| sha2 | 0.10.x | File hash verification | RustCrypto standard, pure Rust |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| wow-mpq | 0.3.x | MPQ validation (optional) | Only if deep MPQ integrity checks needed |
| serde_json | 1.x | Settings serialization | Already in project, for store plugin |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| tauri-plugin-store | Manual JSON file | Store plugin handles edge cases, auto-save |
| sha2 for verification | md5 | SHA256 more secure, minimal perf difference |
| File copy | wow-mpq extraction | MPQ files don't need extraction, just copy |

**Installation:**
```bash
# Add Tauri plugins
npm run tauri add dialog
npm run tauri add store
npm run tauri add fs

# Or manually in Cargo.toml
cargo add tauri-plugin-dialog tauri-plugin-store tauri-plugin-fs sha2
```

```toml
# Cargo.toml additions
[dependencies]
tauri-plugin-dialog = "2"
tauri-plugin-store = "2"
tauri-plugin-fs = "2"
sha2 = "0.10"
```

## Architecture Patterns

### Recommended Module Structure
```
src-tauri/
├── src/
│   ├── install/
│   │   ├── mod.rs           # Module exports, InstallManager
│   │   ├── detector.rs      # WoW folder detection/validation
│   │   ├── copier.rs        # MPQ file copy operations
│   │   ├── verifier.rs      # Installation integrity checks
│   │   └── repair.rs        # Repair/re-apply logic
│   ├── settings/
│   │   ├── mod.rs           # Settings management
│   │   └── store.rs         # Tauri store wrapper
```

### Pattern 1: WoW Folder Detection and Validation
**What:** Detect if app is run from WoW folder, validate user-selected folders
**When to use:** At app startup and when user changes WoW path
**Example:**
```rust
// Source: Turtle WoW installation structure
use std::path::Path;

pub struct WowDetector;

impl WowDetector {
    /// Check if a path is a valid WoW installation
    pub fn is_valid_wow_folder(path: &Path) -> bool {
        // Must have WoW.exe (or WoWFoV.exe for Turtle WoW)
        let has_exe = path.join("WoW.exe").exists()
            || path.join("WoWFoV.exe").exists()
            || path.join("turtle-wow.exe").exists();

        // Must have Data folder
        let has_data = path.join("Data").exists() && path.join("Data").is_dir();

        has_exe && has_data
    }

    /// Try to auto-detect WoW folder from current directory
    pub fn auto_detect() -> Option<std::path::PathBuf> {
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

    /// Get the DATA folder path from WoW installation
    pub fn get_data_folder(wow_path: &Path) -> std::path::PathBuf {
        wow_path.join("Data")
    }
}
```

### Pattern 2: MPQ File Installation (Copy, Not Extract)
**What:** Copy downloaded MPQ files to WoW DATA folder
**When to use:** After download completes, during installation
**Example:**
```rust
// Source: Turtle WoW MPQ installation pattern
use std::path::{Path, PathBuf};
use tokio::fs;
use tauri::ipc::Channel;

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum InstallEvent {
    Started { patch_id: String, file_name: String },
    Progress { patch_id: String, bytes_copied: u64, total_bytes: u64 },
    Completed { patch_id: String, dest_path: String },
    Failed { patch_id: String, error: String },
}

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

    // Get file size for progress
    let metadata = fs::metadata(source_path).await?;
    let total_bytes = metadata.len();

    // Copy file (for large files, use chunked copy with progress)
    fs::copy(source_path, &dest_path).await?;

    // Send completed event
    let _ = on_event.send(InstallEvent::Completed {
        patch_id,
        dest_path: dest_path.to_string_lossy().to_string(),
    });

    Ok(dest_path)
}
```

### Pattern 3: Settings Persistence with Store Plugin
**What:** Save and load user preferences (WoW path, selected modules)
**When to use:** App startup, after user changes settings
**Example:**
```rust
// Source: https://v2.tauri.app/plugin/store/
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use serde_json::json;

const SETTINGS_FILE: &str = "settings.json";
const KEY_WOW_PATH: &str = "wow_path";
const KEY_SELECTED_MODULES: &str = "selected_modules";

pub struct Settings {
    store: tauri_plugin_store::Store<tauri::Wry>,
}

impl Settings {
    pub fn new(app: &AppHandle) -> Result<Self, SettingsError> {
        let store = app.store(SETTINGS_FILE)?;
        Ok(Self { store })
    }

    pub fn get_wow_path(&self) -> Option<String> {
        self.store.get(KEY_WOW_PATH)
            .and_then(|v| v.as_str().map(String::from))
    }

    pub fn set_wow_path(&self, path: &str) -> Result<(), SettingsError> {
        self.store.set(KEY_WOW_PATH, json!(path));
        self.store.save()?;
        Ok(())
    }

    pub fn get_selected_modules(&self) -> Vec<String> {
        self.store.get(KEY_SELECTED_MODULES)
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect())
            .unwrap_or_default()
    }

    pub fn set_selected_modules(&self, modules: &[String]) -> Result<(), SettingsError> {
        self.store.set(KEY_SELECTED_MODULES, json!(modules));
        self.store.save()?;
        Ok(())
    }
}
```

### Pattern 4: Folder Picker Dialog
**What:** Let user select WoW installation folder
**When to use:** When auto-detect fails or user wants to change path
**Example:**
```rust
// Source: https://v2.tauri.app/plugin/dialog/
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

pub async fn pick_wow_folder(app: &AppHandle) -> Option<std::path::PathBuf> {
    // Use blocking version in async context
    let folder = app.dialog()
        .file()
        .set_title("Select Turtle WoW Installation Folder")
        .blocking_pick_folder();

    folder.map(|f| f.path.clone())
}

// Frontend TypeScript
// Source: https://v2.tauri.app/reference/javascript/dialog/
/*
import { open } from '@tauri-apps/plugin-dialog';

async function selectWowFolder(): Promise<string | null> {
    const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Turtle WoW Installation Folder',
    });

    return selected as string | null;
}
*/
```

### Anti-Patterns to Avoid
- **Extracting MPQ files:** MPQ files are the final format, don't try to unpack them
- **Hardcoding WoW paths:** Different users have different install locations
- **Writing to Program Files without elevation:** Recommend users install WoW elsewhere
- **Assuming folder exists:** Always create Data folder if missing (though it should exist)
- **Synchronous file operations on main thread:** Use async fs operations

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Folder picker | Custom file browser | tauri-plugin-dialog | Native OS dialog, accessibility, tested |
| Settings persistence | Manual JSON read/write | tauri-plugin-store | Handles race conditions, auto-save |
| Cross-platform paths | String manipulation | std::path::Path/PathBuf | Handles OS differences correctly |
| File copy | Manual byte streaming | tokio::fs::copy | Optimized, handles edge cases |
| Hash calculation | Manual byte reading | sha2 crate with Digest trait | Streaming API, pure Rust, tested |

**Key insight:** The HD Patch: Reforged uses raw MPQ files, not compressed archives. This means the "extraction" step is actually just a file copy operation. Focus effort on folder detection, verification, and repair logic rather than archive handling.

## Common Pitfalls

### Pitfall 1: Assuming Archive Extraction is Needed
**What goes wrong:** Building complex extraction logic for ZIP/7z
**Why it happens:** Many mod packs use compressed archives
**How to avoid:**
- Verify file format from source (HD Patch uses raw .mpq files)
- Download files maintain their original extension (.mpq)
- Just copy the downloaded file to Data folder
**Warning signs:** Downloaded files already have .mpq extension

### Pitfall 2: Permission Errors on Windows Program Files
**What goes wrong:** Copy fails with "access denied"
**Why it happens:** WoW installed in C:\Program Files requires admin elevation
**How to avoid:**
- Detect if WoW is in protected location
- Show user-friendly message recommending they move WoW folder
- OR request elevation (more complex, not recommended)
**Warning signs:** Works on some machines, fails on others

### Pitfall 3: Not Validating WoW Folder Before Installation
**What goes wrong:** Files copied to wrong location, user confusion
**Why it happens:** User selects wrong folder or folder was moved
**How to avoid:**
- Always validate WoW folder before any operation
- Check for WoW.exe AND Data folder
- Re-validate on each app launch (folder may have moved)
**Warning signs:** Data folder doesn't exist, no .exe found

### Pitfall 4: Forgetting to Handle Existing Files
**What goes wrong:** Old patch version not overwritten, or user loses customizations
**Why it happens:** fs::copy doesn't prompt for overwrite confirmation
**How to avoid:**
- Check if target file exists
- Compare file sizes/hashes
- Overwrite if different, skip if identical (optimization)
- Provide user option to force reinstall
**Warning signs:** Patch reports "installed" but game uses old version

### Pitfall 5: Blocking UI During Large File Operations
**What goes wrong:** App appears frozen during copy
**Why it happens:** File copy done synchronously on main thread
**How to avoid:**
- Use async file operations (tokio::fs)
- Send progress events during copy (for large files)
- Show indeterminate progress for small files
**Warning signs:** UI unresponsive during install

### Pitfall 6: Not Handling Concurrent Modifications
**What goes wrong:** Corrupted files if user runs installer twice, or game running
**Why it happens:** No file locking, no state tracking
**How to avoid:**
- Track installation state in memory
- Prevent concurrent install operations
- Optionally check if WoW.exe is running
**Warning signs:** Partial files, CRC mismatches

## Code Examples

### Complete Installation Manager
```rust
// Source: Combined from Tauri plugin docs and WoW installation patterns
use std::path::{Path, PathBuf};
use tokio::fs;
use sha2::{Sha256, Digest};
use tokio::io::AsyncReadExt;

pub struct InstallManager {
    wow_path: Option<PathBuf>,
    downloads_path: PathBuf,
}

impl InstallManager {
    pub fn new(downloads_path: PathBuf) -> Self {
        Self {
            wow_path: None,
            downloads_path,
        }
    }

    pub fn set_wow_path(&mut self, path: PathBuf) -> Result<(), InstallError> {
        if !WowDetector::is_valid_wow_folder(&path) {
            return Err(InstallError::InvalidWowFolder);
        }
        self.wow_path = Some(path);
        Ok(())
    }

    pub fn get_data_folder(&self) -> Result<PathBuf, InstallError> {
        let wow_path = self.wow_path.as_ref()
            .ok_or(InstallError::WowPathNotSet)?;
        Ok(wow_path.join("Data"))
    }

    /// Install a downloaded MPQ file to the WoW Data folder
    pub async fn install_patch(&self, patch_id: &str) -> Result<PathBuf, InstallError> {
        let data_folder = self.get_data_folder()?;

        // Find the downloaded file
        let mpq_name = format!("Patch-{}.mpq", patch_id);
        let source = self.downloads_path.join(&mpq_name);

        if !source.exists() {
            return Err(InstallError::DownloadNotFound(mpq_name));
        }

        let dest = data_folder.join(&mpq_name);

        // Copy file (overwrites existing)
        fs::copy(&source, &dest).await?;

        Ok(dest)
    }

    /// Verify an installed patch exists and has correct size
    pub async fn verify_patch(&self, patch_id: &str) -> Result<VerifyResult, InstallError> {
        let data_folder = self.get_data_folder()?;
        let mpq_name = format!("Patch-{}.mpq", patch_id);

        let installed_path = data_folder.join(&mpq_name);
        let download_path = self.downloads_path.join(&mpq_name);

        // Check if installed
        if !installed_path.exists() {
            return Ok(VerifyResult::NotInstalled);
        }

        // Check if download exists for comparison
        if !download_path.exists() {
            return Ok(VerifyResult::Installed { verified: false });
        }

        // Compare file sizes
        let installed_size = fs::metadata(&installed_path).await?.len();
        let download_size = fs::metadata(&download_path).await?.len();

        if installed_size != download_size {
            return Ok(VerifyResult::SizeMismatch {
                installed: installed_size,
                expected: download_size,
            });
        }

        Ok(VerifyResult::Installed { verified: true })
    }

    /// Repair a patch by re-copying from downloads
    pub async fn repair_patch(&self, patch_id: &str) -> Result<PathBuf, InstallError> {
        // Simply re-install (copy again)
        self.install_patch(patch_id).await
    }

    /// Check all installed patches
    pub async fn verify_all(&self, patch_ids: &[&str]) -> Vec<(String, VerifyResult)> {
        let mut results = Vec::new();

        for id in patch_ids {
            let result = self.verify_patch(id).await
                .unwrap_or(VerifyResult::Error);
            results.push((id.to_string(), result));
        }

        results
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum VerifyResult {
    NotInstalled,
    Installed { verified: bool },
    SizeMismatch { installed: u64, expected: u64 },
    Error,
}
```

### File Hash Verification (Optional, for stricter checks)
```rust
// Source: https://docs.rs/sha2/latest/sha2/
use sha2::{Sha256, Digest};
use tokio::io::AsyncReadExt;
use tokio::fs::File;

pub async fn compute_file_hash(path: &Path) -> Result<String, std::io::Error> {
    let mut file = File::open(path).await?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

pub async fn verify_file_hash(
    path: &Path,
    expected_hash: &str
) -> Result<bool, std::io::Error> {
    let actual_hash = compute_file_hash(path).await?;
    Ok(actual_hash.eq_ignore_ascii_case(expected_hash))
}
```

### Tauri Commands for Frontend
```rust
// Source: Tauri v2 command patterns
use tauri::{AppHandle, State, ipc::Channel};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn select_wow_folder(app: AppHandle) -> Result<Option<String>, String> {
    let folder = app.dialog()
        .file()
        .set_title("Select Turtle WoW Installation Folder")
        .blocking_pick_folder();

    match folder {
        Some(f) => {
            let path = f.path;
            // Validate the selected folder
            if WowDetector::is_valid_wow_folder(&path) {
                Ok(Some(path.to_string_lossy().to_string()))
            } else {
                Err("Selected folder is not a valid Turtle WoW installation".to_string())
            }
        }
        None => Ok(None), // User cancelled
    }
}

#[tauri::command]
pub async fn install_patches(
    manager: State<'_, InstallManager>,
    patch_ids: Vec<String>,
    on_event: Channel<InstallEvent>,
) -> Result<Vec<String>, String> {
    let mut installed = Vec::new();

    for id in patch_ids {
        match manager.install_patch(&id).await {
            Ok(path) => {
                let _ = on_event.send(InstallEvent::Completed {
                    patch_id: id.clone(),
                    dest_path: path.to_string_lossy().to_string(),
                });
                installed.push(id);
            }
            Err(e) => {
                let _ = on_event.send(InstallEvent::Failed {
                    patch_id: id.clone(),
                    error: e.to_string(),
                });
            }
        }
    }

    Ok(installed)
}

#[tauri::command]
pub async fn verify_installation(
    manager: State<'_, InstallManager>,
    patch_ids: Vec<String>,
) -> Result<Vec<(String, VerifyResult)>, String> {
    let ids: Vec<&str> = patch_ids.iter().map(|s| s.as_str()).collect();
    Ok(manager.verify_all(&ids).await)
}

#[tauri::command]
pub async fn repair_patches(
    manager: State<'_, InstallManager>,
    patch_ids: Vec<String>,
) -> Result<Vec<String>, String> {
    let mut repaired = Vec::new();

    for id in patch_ids {
        if manager.repair_patch(&id).await.is_ok() {
            repaired.push(id);
        }
    }

    Ok(repaired)
}
```

### Frontend TypeScript Types and Functions
```typescript
// Source: Tauri v2 IPC patterns
import { invoke, Channel } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

type InstallEvent =
  | { event: 'started'; data: { patchId: string; fileName: string } }
  | { event: 'progress'; data: { patchId: string; bytesCopied: number; totalBytes: number } }
  | { event: 'completed'; data: { patchId: string; destPath: string } }
  | { event: 'failed'; data: { patchId: string; error: string } };

type VerifyResult =
  | { type: 'notInstalled' }
  | { type: 'installed'; verified: boolean }
  | { type: 'sizeMismatch'; installed: number; expected: number }
  | { type: 'error' };

export async function selectWowFolder(): Promise<string | null> {
    return await invoke<string | null>('select_wow_folder');
}

export async function installPatches(
    patchIds: string[],
    onProgress: (event: InstallEvent) => void
): Promise<string[]> {
    const channel = new Channel<InstallEvent>();
    channel.onmessage = onProgress;

    return await invoke<string[]>('install_patches', {
        patchIds,
        onEvent: channel,
    });
}

export async function verifyInstallation(
    patchIds: string[]
): Promise<Array<[string, VerifyResult]>> {
    return await invoke('verify_installation', { patchIds });
}

export async function repairPatches(patchIds: string[]): Promise<string[]> {
    return await invoke('repair_patches', { patchIds });
}
```

## WoW Installation Detection Logic

### Folder Validation Criteria
```
Valid WoW folder must have:
1. One of these executables:
   - WoW.exe (standard)
   - WoWFoV.exe (widescreen patch for Turtle WoW)
   - turtle-wow.exe (Turtle WoW launcher)

2. Data subfolder (directory)
   - May contain existing patch-*.mpq files
   - May be empty for fresh install (create if needed)

3. Optionally validate:
   - Interface/AddOns subfolder
   - WTF folder (character settings)
```

### Common WoW Locations by Platform

| Platform | Typical Locations |
|----------|-------------------|
| Windows | `C:\TurtleWoW`, `C:\Games\TurtleWoW`, `D:\Games\TurtleWoW` |
| macOS | `~/Games/TurtleWoW`, `/Applications/TurtleWoW` |
| Linux | `~/TurtleWoW`, `~/.wine/drive_c/TurtleWoW`, `~/Games/TurtleWoW` |

**Note:** Turtle WoW recommends NOT installing to `C:\Program Files` due to permission issues.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Extract archives | Direct MPQ copy | HD Patch uses raw MPQ | Simpler installation |
| Manual path input | Folder picker dialog | Tauri v2 dialog plugin | Better UX |
| Config files | Tauri store plugin | Tauri v2 | Built-in persistence |
| Sync file ops | Async tokio::fs | tokio 1.0 | Non-blocking UI |

**Deprecated/outdated:**
- tauri v1 `fs` module - Use tauri-plugin-fs instead
- Manual JSON settings - Use tauri-plugin-store
- Custom file dialogs - Use tauri-plugin-dialog

## Open Questions

1. **Hash verification source**
   - What we know: HD Patch forum post has file sizes but no checksums
   - What's unclear: Are checksums published anywhere?
   - Recommendation: Use file size verification primarily; add hash support if checksums become available

2. **Concurrent game access**
   - What we know: WoW locks some MPQ files while running
   - What's unclear: Does WoW lock all Data folder files or just specific ones?
   - Recommendation: Check if WoW.exe is running before installation; warn user if so

3. **Optimal repair strategy**
   - What we know: Can re-copy from downloads, or re-download entirely
   - What's unclear: User preference for speed vs. freshness
   - Recommendation: Default to re-copy if download exists; offer "full repair" that re-downloads

## Sources

### Primary (HIGH confidence)
- [Tauri v2 Dialog Plugin](https://v2.tauri.app/plugin/dialog/) - Folder picker API
- [Tauri v2 Store Plugin](https://v2.tauri.app/plugin/store/) - Settings persistence
- [Tauri v2 FS Plugin](https://v2.tauri.app/plugin/file-system/) - File operations
- [sha2 crate docs](https://docs.rs/sha2/latest/sha2/) - Hash calculation
- [Turtle WoW Installation Guide](https://turtle-wow.fandom.com/wiki/Installation_Guide) - Folder structure

### Secondary (MEDIUM confidence)
- [HD Patch: Reforged Forum Post](https://forum.turtlecraft.gg/viewtopic.php?t=21355) - MPQ file format confirmation
- [TurtleHD GitHub](https://github.com/redmagejoe/TurtleHD) - MPQ distribution pattern
- [WoW Modding Guide](https://www.wowmodding.net/topic/1460-41-creating-your-first-mpq-patch/) - MPQ installation patterns

### Tertiary (LOW confidence)
- [wow-mpq crate](https://crates.io/crates/wow-mpq) - Only if deep MPQ validation needed (not required for installation)

## Metadata

**Confidence breakdown:**
- File format (raw MPQ): HIGH - Confirmed from forum post and downloads
- Tauri plugins: HIGH - Official documentation
- WoW folder structure: HIGH - Standard across WoW versions
- Verification approach: MEDIUM - File size is reliable; hash depends on data availability
- Repair logic: MEDIUM - Simple re-copy works; advanced scenarios need testing

**Research date:** 2026-01-16
**Valid until:** 2026-02-16 (30 days - stable domain, unlikely to change)

**Key insight:** The installation engine is much simpler than initially assumed. HD Patch: Reforged distributes raw MPQ files, so there's no archive extraction step. The core work is folder detection, file copying, and verification - all well-supported by Tauri plugins and standard Rust libraries.
