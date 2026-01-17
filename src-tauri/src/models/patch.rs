//! Patch module data structures

use serde::{Deserialize, Serialize};
use crate::models::DownloadLink;

/// Patch ID is now a simple string to allow dynamic patches from JSON
pub type PatchId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchModule {
    pub id: PatchId,
    pub name: String,
    pub description: String,
    #[serde(rename = "links")]
    pub downloads: Vec<DownloadLink>,
    #[serde(default)]
    pub dependencies: Vec<PatchId>,
    #[serde(default)]
    pub conflicts: Vec<PatchId>,
    pub file_size: Option<String>,
    pub last_updated: Option<String>,
    /// Named variants for patches with multiple download options
    pub variants: Option<Vec<String>>,
    /// Preview image URL
    pub preview: Option<String>,
}

/// Group definition for organizing patches in the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchGroup {
    pub name: String,
    pub description: String,
    pub ids: Vec<PatchId>,
    /// If true, all patches in this group toggle together
    #[serde(default)]
    pub linked: bool,
}

/// Full patches configuration from remote JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchesConfig {
    pub version: u32,
    pub patches: std::collections::HashMap<PatchId, PatchData>,
    #[serde(default)]
    pub groups: Vec<PatchGroup>,
}

/// Raw patch data from JSON (before conversion to PatchModule)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchData {
    pub name: String,
    pub description: String,
    pub links: Vec<DownloadLink>,
    #[serde(default)]
    pub dependencies: Vec<PatchId>,
    #[serde(default)]
    pub conflicts: Vec<PatchId>,
    pub variants: Option<Vec<String>>,
    pub preview: Option<String>,
}
