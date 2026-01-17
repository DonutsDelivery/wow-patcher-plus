//! Patch module parsing and metadata

use crate::models::{PatchId, PatchModule, ParserError, DownloadLink};
use crate::parser::dependencies::get_dependencies;

/// Get metadata for all known patch modules
pub fn get_all_modules() -> Vec<PatchModule> {
    vec![
        PatchModule {
            id: PatchId::A,
            name: "Player Characters & NPCs".into(),
            description: "HD models and textures for players and NPCs".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::A),
            file_size: None,
            last_updated: None,
            variants: None,
        },
        PatchModule {
            id: PatchId::B,
            name: "Buildings".into(),
            description: "HD building models".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::B),
            file_size: None,
            last_updated: None,
            variants: None,
        },
        PatchModule {
            id: PatchId::C,
            name: "Creatures".into(),
            description: "HD creature models".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::C),
            file_size: None,
            last_updated: None,
            variants: None,
        },
        PatchModule {
            id: PatchId::D,
            name: "Doodads".into(),
            description: "HD doodad models".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::D),
            file_size: None,
            last_updated: None,
            variants: None,
        },
        PatchModule {
            id: PatchId::E,
            name: "Environment".into(),
            description: "HD environment textures".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::E),
            file_size: None,
            last_updated: None,
            variants: None,
        },
        PatchModule {
            id: PatchId::G,
            name: "Gear & Weapons".into(),
            description: "HD equipment models".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::G),
            file_size: None,
            last_updated: None,
            variants: None,
        },
        PatchModule {
            id: PatchId::I,
            name: "Interface".into(),
            description: "HD UI elements".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::I),
            file_size: None,
            last_updated: None,
            variants: None,
        },
        PatchModule {
            id: PatchId::L,
            name: "A Little Extra for Females".into(),
            description: "Additional female model options".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::L),
            file_size: None,
            last_updated: None,
            variants: Some(vec![
                "Regular Version".into(),
                "Less Thicc Version".into(),
            ]),
        },
        PatchModule {
            id: PatchId::M,
            name: "Maps & Loading Screens".into(),
            description: "HD loading screens".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::M),
            file_size: None,
            last_updated: None,
            variants: None,
        },
        PatchModule {
            id: PatchId::N,
            name: "Darker Nights".into(),
            description: "Night ambiance enhancement".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::N),
            file_size: None,
            last_updated: None,
            variants: None,
        },
        PatchModule {
            id: PatchId::O,
            name: "Raid Visuals Mod".into(),
            description: "Enhanced raid effects".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::O),
            file_size: None,
            last_updated: None,
            variants: None,
        },
        PatchModule {
            id: PatchId::S,
            name: "Sounds & Music".into(),
            description: "HD audio".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::S),
            file_size: None,
            last_updated: None,
            variants: None,
        },
        PatchModule {
            id: PatchId::U,
            name: "Ultra HD".into(),
            description: "4K character textures".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::U),
            file_size: None,
            last_updated: None,
            variants: None,
        },
        PatchModule {
            id: PatchId::V,
            name: "Visual Effects".into(),
            description: "HD spell effects".into(),
            downloads: vec![],
            dependencies: get_dependencies(PatchId::V),
            file_size: None,
            last_updated: None,
            variants: None,
        },
    ]
}

/// Parse modules from forum post content
/// This enriches the known module list with download links found in the content
pub fn parse_modules(_content: &str, links: &[DownloadLink]) -> Result<Vec<PatchModule>, ParserError> {
    let mut modules = get_all_modules();

    // Associate links with modules based on filename patterns
    for link in links {
        if let Some(filename) = &link.file_name {
            let filename_lower = filename.to_lowercase();

            // Try to match filename to module
            let module_id = if filename_lower.contains("patch-a") || filename_lower.contains("patch_a") {
                Some(PatchId::A)
            } else if filename_lower.contains("patch-b") || filename_lower.contains("patch_b") {
                Some(PatchId::B)
            } else if filename_lower.contains("patch-c") || filename_lower.contains("patch_c") {
                Some(PatchId::C)
            } else if filename_lower.contains("patch-d") || filename_lower.contains("patch_d") {
                Some(PatchId::D)
            } else if filename_lower.contains("patch-e") || filename_lower.contains("patch_e") {
                Some(PatchId::E)
            } else if filename_lower.contains("patch-g") || filename_lower.contains("patch_g") {
                Some(PatchId::G)
            } else if filename_lower.contains("patch-i") || filename_lower.contains("patch_i") {
                Some(PatchId::I)
            } else if filename_lower.contains("patch-l") || filename_lower.contains("patch_l") {
                Some(PatchId::L)
            } else if filename_lower.contains("patch-m") || filename_lower.contains("patch_m") {
                Some(PatchId::M)
            } else if filename_lower.contains("patch-n") || filename_lower.contains("patch_n") {
                Some(PatchId::N)
            } else if filename_lower.contains("patch-o") || filename_lower.contains("patch_o") {
                Some(PatchId::O)
            } else if filename_lower.contains("patch-s") || filename_lower.contains("patch_s") {
                Some(PatchId::S)
            } else if filename_lower.contains("patch-u") || filename_lower.contains("patch_u") {
                Some(PatchId::U)
            } else if filename_lower.contains("patch-v") || filename_lower.contains("patch_v") {
                Some(PatchId::V)
            } else {
                None
            };

            if let Some(id) = module_id {
                if let Some(module) = modules.iter_mut().find(|m| m.id == id) {
                    module.downloads.push(link.clone());
                }
            }
        }
    }

    Ok(modules)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DownloadProvider;

    #[test]
    fn test_get_all_modules() {
        let modules = get_all_modules();
        assert_eq!(modules.len(), 14);

        // Check specific modules exist
        assert!(modules.iter().any(|m| m.id == PatchId::A));
        assert!(modules.iter().any(|m| m.id == PatchId::U));
    }

    #[test]
    fn test_parse_modules_with_links() {
        let links = vec![
            DownloadLink {
                provider: DownloadProvider::Mediafire,
                url: "https://mediafire.com/file/xyz/patch-a.7z".into(),
                file_name: Some("patch-a.7z".into()),
                variant: None,
            },
            DownloadLink {
                provider: DownloadProvider::GoogleDrive,
                url: "https://drive.google.com/file/d/abc/view".into(),
                file_name: Some("Patch-G-v2.rar".into()),
                variant: None,
            },
        ];

        let modules = parse_modules("", &links).unwrap();

        let module_a = modules.iter().find(|m| m.id == PatchId::A).unwrap();
        assert_eq!(module_a.downloads.len(), 1);

        let module_g = modules.iter().find(|m| m.id == PatchId::G).unwrap();
        assert_eq!(module_g.downloads.len(), 1);
    }
}
