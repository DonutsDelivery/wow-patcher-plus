//! Patch module data structures

use serde::{Deserialize, Serialize};
use crate::models::DownloadLink;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchModule {
    pub id: PatchId,
    pub name: String,
    pub description: String,
    pub downloads: Vec<DownloadLink>,
    pub dependencies: Vec<PatchId>,
    pub file_size: Option<String>,
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatchId {
    A, // Player Characters & NPCs
    B, // Buildings (requires D, E)
    C, // Creatures
    D, // Doodads (requires B, E)
    E, // Environment (requires B, D)
    G, // Gear & Weapons
    I, // Interface
    L, // A Little Extra for Females (requires A)
    M, // Maps & Loading Screens
    N, // Darker Nights
    O, // Raid Visuals Mod (requires S)
    S, // Sounds & Music
    U, // Ultra HD (requires A, G)
    V, // Visual Effects for Spells
}

impl PatchId {
    /// Returns the human-readable name for this patch
    pub fn name(&self) -> &'static str {
        match self {
            PatchId::A => "Player Characters & NPCs",
            PatchId::B => "Buildings",
            PatchId::C => "Creatures",
            PatchId::D => "Doodads",
            PatchId::E => "Environment",
            PatchId::G => "Gear & Weapons",
            PatchId::I => "Interface",
            PatchId::L => "A Little Extra for Females",
            PatchId::M => "Maps & Loading Screens",
            PatchId::N => "Darker Nights",
            PatchId::O => "Raid Visuals Mod",
            PatchId::S => "Sounds & Music",
            PatchId::U => "Ultra HD",
            PatchId::V => "Visual Effects",
        }
    }
}
