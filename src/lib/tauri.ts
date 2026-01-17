import { invoke, Channel } from '@tauri-apps/api/core';

// Types matching Rust models
export interface PatchModule {
  id: string;
  name: string;
  description: string;
  links: { provider: string; url: string }[];
  dependencies: string[];
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
export async function fetchPatches(): Promise<PatchModule[]> {
  return invoke('fetch_patches');
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
