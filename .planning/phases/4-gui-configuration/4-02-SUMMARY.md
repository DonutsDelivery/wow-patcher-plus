---
phase: 4-gui-configuration
plan: 02
subsystem: ui
tags: [react, hooks, shadcn-ui, tailwind, tauri-ipc, presets]

# Dependency graph
requires:
  - phase: 4-gui-configuration
    plan: 01
    provides: shadcn/ui components (Button, Card, Checkbox, Progress, ScrollArea, Select, Switch)
  - phase: 3-installation-engine
    provides: WoW folder detection, settings persistence, install commands
provides:
  - Quality preset definitions (low, medium, high, ultra)
  - Typed Tauri wrappers for IPC commands
  - usePatches hook for module fetching and selection
  - useWowPath hook for folder detection and selection
  - PresetSelector dropdown component
  - ModuleList scrollable checkbox list
  - FolderPicker with status indicators
affects: [4-03-PLAN, 4-04-PLAN]

# Tech tracking
tech-stack:
  added: []
  patterns: [typed-tauri-invoke, custom-react-hooks, controlled-selection-state]

key-files:
  created:
    - src/lib/presets.ts
    - src/lib/tauri.ts
    - src/hooks/usePatches.ts
    - src/hooks/useWowPath.ts
    - src/components/PresetSelector.tsx
    - src/components/ModuleList.tsx
    - src/components/FolderPicker.tsx
  modified: []

key-decisions:
  - "Presets define module arrays: low (I,M), medium (A,C,G,I,M,V), high (+B,D,E,S), ultra (+U)"
  - "Optional modules (L, N, O) excluded from presets - user toggles manually"
  - "usePatches returns Set<string> for O(1) selection lookups"
  - "useWowPath tries saved path first, then auto-detect on mount"
  - "FolderPicker uses Lucide icons (Folder, Check, AlertCircle) for status"

patterns-established:
  - "Typed Tauri wrappers: src/lib/tauri.ts exports async functions matching Rust commands"
  - "Hook pattern: fetch on mount with loading/error state, expose callback actions"
  - "Component props: minimal interface with callbacks, no internal state"
  - "Selection state: Set<string> for module IDs with toggle pattern"

# Metrics
duration: 2min
completed: 2026-01-16
---

# Phase 4 Plan 02: Core UI Components Summary

**Quality presets, typed Tauri wrappers, usePatches/useWowPath hooks, and PresetSelector/ModuleList/FolderPicker components**

## Performance

- **Duration:** 2 min
- **Started:** 2026-01-16T22:26:44Z
- **Completed:** 2026-01-16T22:28:52Z
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments
- Created quality preset definitions with 4 tiers (Low/Medium/High/Ultra)
- Built typed Tauri wrappers for all backend commands (parser, install)
- Created usePatches hook with selection state and preset application
- Created useWowPath hook with auto-detection and folder picker integration
- Built PresetSelector dropdown with shadcn/ui Select component
- Built ModuleList with ScrollArea and Checkbox components
- Built FolderPicker with status indicators using Lucide icons

## Task Commits

Each task was committed atomically:

1. **Task 1: Create presets and typed Tauri wrappers** - `61d2d85` (feat)
2. **Task 2: Create usePatches and useWowPath hooks** - `9f3d647` (feat)
3. **Task 3: Create PresetSelector, ModuleList, and FolderPicker components** - `efb289a` (feat)

## Files Created/Modified

Created:
- `src/lib/presets.ts` - Quality preset definitions (PRESETS, OPTIONAL_MODULES, PresetKey type)
- `src/lib/tauri.ts` - Typed invoke wrappers for all Tauri commands
- `src/hooks/usePatches.ts` - Patch fetching, selection state, preset application
- `src/hooks/useWowPath.ts` - WoW path loading, auto-detection, folder picking
- `src/components/PresetSelector.tsx` - Dropdown with Low/Medium/High/Ultra presets
- `src/components/ModuleList.tsx` - Scrollable list with checkboxes for all modules
- `src/components/FolderPicker.tsx` - Button with path display and status indicators

## Decisions Made
- Presets use explicit module arrays that are processed through auto_select_deps for dependency resolution
- Optional modules (L: Female extras, N: Darker Nights, O: Raid Visuals) kept separate from presets
- Selection state stored as Set<string> for fast has/add/delete operations
- FolderPicker uses three states: loading (auto-detect), path set (green check), no path (yellow alert)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - all components compiled and verified on first attempt.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- All core configuration components ready
- PresetSelector/ModuleList/FolderPicker ready to integrate into App layout
- Hooks provide complete state management for patch selection workflow
- Ready for 4-03-PLAN.md: Application shell and layout integration

---
*Phase: 4-gui-configuration*
*Completed: 2026-01-16*
