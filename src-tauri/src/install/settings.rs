//! Settings persistence using Tauri store plugin

use serde_json::json;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use thiserror::Error;

const SETTINGS_FILE: &str = "settings.json";
const KEY_WOW_PATH: &str = "wow_path";
const KEY_SELECTED_MODULES: &str = "selected_modules";

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("Store error: {0}")]
    StoreError(String),

    #[error("Failed to save settings: {0}")]
    SaveError(String),
}

/// Settings manager using Tauri store plugin
pub struct Settings<'a> {
    app: &'a AppHandle,
}

impl<'a> Settings<'a> {
    pub fn new(app: &'a AppHandle) -> Self {
        Self { app }
    }

    /// Get the saved WoW installation path
    pub fn get_wow_path(&self) -> Option<String> {
        let store = self.app.store(SETTINGS_FILE).ok()?;
        store
            .get(KEY_WOW_PATH)
            .and_then(|v| v.as_str().map(String::from))
    }

    /// Save the WoW installation path
    pub fn set_wow_path(&self, path: &str) -> Result<(), SettingsError> {
        let store = self
            .app
            .store(SETTINGS_FILE)
            .map_err(|e| SettingsError::StoreError(e.to_string()))?;
        store.set(KEY_WOW_PATH, json!(path));
        store
            .save()
            .map_err(|e| SettingsError::SaveError(e.to_string()))?;
        Ok(())
    }

    /// Get the list of selected module IDs
    pub fn get_selected_modules(&self) -> Vec<String> {
        let store = match self.app.store(SETTINGS_FILE) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let value = match store.get(KEY_SELECTED_MODULES) {
            Some(v) => v,
            None => return Vec::new(),
        };

        match value.as_array() {
            Some(arr) => arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            None => Vec::new(),
        }
    }

    /// Save the list of selected module IDs
    pub fn set_selected_modules(&self, modules: &[String]) -> Result<(), SettingsError> {
        let store = self
            .app
            .store(SETTINGS_FILE)
            .map_err(|e| SettingsError::StoreError(e.to_string()))?;
        store.set(KEY_SELECTED_MODULES, json!(modules));
        store
            .save()
            .map_err(|e| SettingsError::SaveError(e.to_string()))?;
        Ok(())
    }
}
