//! Module dependency validation - now fully dynamic from JSON

use std::collections::{HashSet, HashMap};
use crate::models::{PatchId, PatchModule};

/// Validate that a selection of modules satisfies all dependency and conflict rules
/// Rules are now read dynamically from the patch modules
pub fn validate_module_selection(
    selected: &HashSet<PatchId>,
    modules: &[PatchModule],
    linked_groups: &[Vec<PatchId>],
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Build lookup map
    let module_map: HashMap<&PatchId, &PatchModule> = modules
        .iter()
        .map(|m| (&m.id, m))
        .collect();

    // Check linked groups - all or none must be selected
    for group in linked_groups {
        let group_selected: Vec<_> = group.iter().filter(|p| selected.contains(*p)).collect();
        if !group_selected.is_empty() && group_selected.len() != group.len() {
            let missing: Vec<_> = group
                .iter()
                .filter(|p| !selected.contains(*p))
                .cloned()
                .collect();
            errors.push(format!(
                "Patches {} must be installed together. Missing: {}",
                group.join(", "),
                missing.join(", ")
            ));
        }
    }

    // Check dependencies
    for patch_id in selected {
        if let Some(module) = module_map.get(patch_id) {
            for dep in &module.dependencies {
                if !selected.contains(dep) {
                    errors.push(format!(
                        "Patch {} ({}) requires Patch {} to be selected",
                        patch_id, module.name, dep
                    ));
                }
            }
        }
    }

    // Check conflicts
    for patch_id in selected {
        if let Some(module) = module_map.get(patch_id) {
            for conflict in &module.conflicts {
                if selected.contains(conflict) {
                    if let Some(conflict_module) = module_map.get(conflict) {
                        errors.push(format!(
                            "Patch {} ({}) conflicts with Patch {} ({})",
                            patch_id, module.name, conflict, conflict_module.name
                        ));
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Auto-select dependencies for a given selection
/// Returns a new set with all required dependencies added
pub fn auto_select_dependencies(
    selected: &HashSet<PatchId>,
    modules: &[PatchModule],
    linked_groups: &[Vec<PatchId>],
) -> HashSet<PatchId> {
    let mut result = selected.clone();

    // Build lookup map
    let module_map: HashMap<&PatchId, &PatchModule> = modules
        .iter()
        .map(|m| (&m.id, m))
        .collect();

    // Add dependencies
    for patch_id in selected.iter() {
        if let Some(module) = module_map.get(patch_id) {
            for dep in &module.dependencies {
                result.insert(dep.clone());
            }
        }
    }

    // Add linked group members
    for group in linked_groups {
        if group.iter().any(|p| selected.contains(p)) {
            for member in group {
                result.insert(member.clone());
            }
        }
    }

    // Recurse until no more changes (handles transitive deps)
    if result.len() > selected.len() {
        auto_select_dependencies(&result, modules, linked_groups)
    } else {
        result
    }
}

/// Get conflicting patches for a selection
pub fn get_conflicts(
    selected: &HashSet<PatchId>,
    modules: &[PatchModule],
) -> HashSet<PatchId> {
    let mut conflicts = HashSet::new();

    let module_map: HashMap<&PatchId, &PatchModule> = modules
        .iter()
        .map(|m| (&m.id, m))
        .collect();

    for patch_id in selected {
        if let Some(module) = module_map.get(patch_id) {
            for conflict in &module.conflicts {
                conflicts.insert(conflict.clone());
            }
        }
    }

    conflicts
}
