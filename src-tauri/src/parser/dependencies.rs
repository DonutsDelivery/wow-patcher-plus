//! Module dependency validation

use std::collections::HashSet;
use crate::models::PatchId;

/// Validate that a selection of modules satisfies all dependency rules
///
/// Rules:
/// - B, D, E must be installed together (all or none)
/// - L requires A
/// - U requires A and G
/// - O requires S
pub fn validate_module_selection(selected: &HashSet<PatchId>) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // B, D, E must be together or none
    let bde_group = [PatchId::B, PatchId::D, PatchId::E];
    let bde_selected: Vec<_> = bde_group.iter().filter(|p| selected.contains(p)).collect();
    if !bde_selected.is_empty() && bde_selected.len() != 3 {
        let missing: Vec<_> = bde_group
            .iter()
            .filter(|p| !selected.contains(p))
            .map(|p| format!("{:?}", p))
            .collect();
        errors.push(format!(
            "Patches B, D, and E must be installed together. Missing: {}",
            missing.join(", ")
        ));
    }

    // L requires A
    if selected.contains(&PatchId::L) && !selected.contains(&PatchId::A) {
        errors.push("Patch L (A Little Extra for Females) requires Patch A (Player Characters)".into());
    }

    // U requires A and G
    if selected.contains(&PatchId::U) {
        if !selected.contains(&PatchId::A) {
            errors.push("Patch U (Ultra HD) requires Patch A (Player Characters)".into());
        }
        if !selected.contains(&PatchId::G) {
            errors.push("Patch U (Ultra HD) requires Patch G (Gear & Weapons)".into());
        }
    }

    // O requires S
    if selected.contains(&PatchId::O) && !selected.contains(&PatchId::S) {
        errors.push("Patch O (Raid Visuals Mod) requires Patch S (Sounds & Music)".into());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Get the dependencies for a given patch
/// Note: B, D, E are a GROUP (must install together) but don't have dependencies on each other
/// They are handled as a group toggle in the UI, not as blocking dependencies
pub fn get_dependencies(patch: PatchId) -> Vec<PatchId> {
    match patch {
        // L requires A (character base)
        PatchId::L => vec![PatchId::A],
        // U requires A (characters) and G (gear)
        PatchId::U => vec![PatchId::A, PatchId::G],
        // All other patches are independent (B, D, E are a group but handled separately)
        _ => vec![],
    }
}

/// Auto-select dependencies for a given selection
/// Returns a new set with all required dependencies added
pub fn auto_select_dependencies(selected: &HashSet<PatchId>) -> HashSet<PatchId> {
    let mut result = selected.clone();

    for patch in selected.iter() {
        for dep in get_dependencies(*patch) {
            result.insert(dep);
        }
    }

    // Recurse until no more changes (handles transitive deps)
    if result.len() > selected.len() {
        auto_select_dependencies(&result)
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_selection_empty() {
        let selected = HashSet::new();
        assert!(validate_module_selection(&selected).is_ok());
    }

    #[test]
    fn test_valid_selection_single() {
        let mut selected = HashSet::new();
        selected.insert(PatchId::A);
        selected.insert(PatchId::C);
        selected.insert(PatchId::G);
        assert!(validate_module_selection(&selected).is_ok());
    }

    #[test]
    fn test_valid_bde_together() {
        let mut selected = HashSet::new();
        selected.insert(PatchId::B);
        selected.insert(PatchId::D);
        selected.insert(PatchId::E);
        assert!(validate_module_selection(&selected).is_ok());
    }

    #[test]
    fn test_invalid_bde_partial() {
        let mut selected = HashSet::new();
        selected.insert(PatchId::B);
        selected.insert(PatchId::D);
        // Missing E
        let result = validate_module_selection(&selected);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("B, D, and E must be installed together"));
    }

    #[test]
    fn test_invalid_l_without_a() {
        let mut selected = HashSet::new();
        selected.insert(PatchId::L);
        let result = validate_module_selection(&selected);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("Patch L"));
        assert!(errors[0].contains("requires Patch A"));
    }

    #[test]
    fn test_invalid_u_without_a_and_g() {
        let mut selected = HashSet::new();
        selected.insert(PatchId::U);
        let result = validate_module_selection(&selected);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2); // Missing both A and G
    }

    #[test]
    fn test_invalid_o_without_s() {
        let mut selected = HashSet::new();
        selected.insert(PatchId::O);
        let result = validate_module_selection(&selected);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("Patch O"));
    }

    #[test]
    fn test_auto_select_dependencies() {
        let mut selected = HashSet::new();
        selected.insert(PatchId::L);

        let result = auto_select_dependencies(&selected);
        assert!(result.contains(&PatchId::L));
        assert!(result.contains(&PatchId::A)); // Auto-added
    }

    #[test]
    fn test_auto_select_bde() {
        let mut selected = HashSet::new();
        selected.insert(PatchId::B);

        let result = auto_select_dependencies(&selected);
        assert!(result.contains(&PatchId::B));
        assert!(result.contains(&PatchId::D));
        assert!(result.contains(&PatchId::E));
    }
}
