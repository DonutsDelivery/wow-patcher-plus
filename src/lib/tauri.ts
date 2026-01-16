import { invoke } from '@tauri-apps/api/core';

// Types matching Rust models
export interface PatchModule {
  id: string;
  name: string;
  description: string;
  links: { provider: string; url: string }[];
  dependencies: string[];
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
