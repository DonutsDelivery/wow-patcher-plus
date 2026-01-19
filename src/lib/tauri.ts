import { invoke, Channel } from '@tauri-apps/api/core';

// Types matching Rust models
export interface DownloadLink {
  provider: string;
  url: string;
  file_name?: string;
  variant?: string;
}

export interface PatchModule {
  id: string;
  name: string;
  description: string;
  links: DownloadLink[];
  dependencies: string[];
  conflicts: string[];
  variants?: string[];
  preview?: string;
  author?: string;
  forumUrl?: string;
}

export interface PatchGroup {
  name: string;
  description: string;
  ids: string[];
  linked?: boolean;
}

export interface PatchesResponse {
  patches: PatchModule[];
  groups: PatchGroup[];
}

// Download events (matches DownloadEvent in progress.rs with serde camelCase)
export type DownloadEvent =
  | { event: 'started'; data: { downloadId: string; fileName: string; totalBytes: number } }
  | { event: 'progress'; data: { downloadId: string; downloadedBytes: number; totalBytes: number; speedBps: number; percent: number } }
  | { event: 'completed'; data: { downloadId: string; filePath: string } }
  | { event: 'failed'; data: { downloadId: string; error: string } };

// Install events (matches InstallEvent in copier.rs with serde camelCase)
export type InstallEvent =
  | { event: 'started'; data: { patchId: string; fileName: string } }
  | { event: 'progress'; data: { patchId: string; bytesCopied: number; totalBytes: number } }
  | { event: 'completed'; data: { patchId: string; destPath: string } }
  | { event: 'failed'; data: { patchId: string; error: string } };

// Verification result from Rust
export interface VerifyResult {
  status: 'ok' | 'sizeMismatch' | 'missing' | 'noReference';
}

// Repair result from Rust
export interface RepairResult {
  patchId: string;
  success: boolean;
  error?: string;
}

// Parser commands
export async function fetchPatches(): Promise<PatchesResponse> {
  return invoke('fetch_patches');
}

export async function getConflicts(selected: string[]): Promise<string[]> {
  return invoke('get_conflicts', { selected });
}

export async function validateSelection(selected: string[]): Promise<void> {
  return invoke('validate_selection', { selected });
}

export async function autoSelectDeps(selected: string[]): Promise<string[]> {
  return invoke('auto_select_deps', { selected });
}

// Install commands
export async function selectWowFolder(): Promise<string | null> {
  return invoke('select_wow_folder');
}

export async function getWowPath(): Promise<string | null> {
  return invoke('get_wow_path');
}

export async function autoDetectWow(): Promise<string | null> {
  return invoke('auto_detect_wow');
}

export async function loadSavedWowPath(): Promise<string | null> {
  return invoke('load_saved_wow_path');
}

// Download commands
export async function startDownload(
  shareUrl: string,
  provider: string,
  destDir: string,
  onProgress: Channel<DownloadEvent>,
  targetFilename?: string
): Promise<string> {
  return invoke('start_download', { shareUrl, provider, destDir, onProgress, targetFilename });
}

export async function getActiveDownloads(): Promise<number> {
  return invoke('get_active_downloads');
}

// Install commands
export async function installPatches(
  patchIds: string[],
  onEvent: Channel<InstallEvent>
): Promise<string[]> {
  return invoke('install_patches', { patchIds, onEvent });
}

export async function verifyPatches(patchIds: string[]): Promise<[string, VerifyResult][]> {
  return invoke('verify_patches', { patchIds });
}

export async function repairPatches(
  patchIds: string[],
  onEvent: Channel<InstallEvent>
): Promise<RepairResult[]> {
  return invoke('repair_patches', { patchIds, onEvent });
}

export async function detectInstalledPatches(patchIds: string[]): Promise<string[]> {
  return invoke('detect_installed_patches', { patchIds });
}

export async function uninstallPatches(patchIds: string[]): Promise<string[]> {
  return invoke('uninstall_patches', { patchIds });
}

// Requirements check
export interface RequirementsStatus {
  vanilla_helpers: boolean;
  dxvk: boolean;
}

export async function checkRequirements(): Promise<RequirementsStatus | null> {
  return invoke('check_requirements');
}

export async function installVanillaHelpers(): Promise<void> {
  return invoke('install_vanilla_helpers');
}

export async function installDxvk(version?: string): Promise<void> {
  return invoke('install_dxvk', { version });
}

// Update check types
export interface UpdateInfo {
  currentVersion: string;
  latestVersion: string;
  updateAvailable: boolean;
  downloadUrl?: string;
  releaseNotes?: string;
}

export interface PatchFreshness {
  patchId: string;
  remoteModified?: string;
  localModified?: string;
  needsUpdate: boolean;
}

// Update commands
export async function checkForUpdates(): Promise<UpdateInfo> {
  return invoke('check_for_updates');
}

export async function checkPatchFreshness(patchId: string, downloadUrl: string): Promise<PatchFreshness> {
  return invoke('check_patch_freshness', { patchId, downloadUrl });
}
