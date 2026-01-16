---
phase: 3-installation-engine
plan: 01
subsystem: install
tags: [tauri-plugin-dialog, tauri-plugin-store, wow-detection, settings-persistence]

# Dependency graph
requires:
  - phase: 2-download-engine
    provides: Download infrastructure with provider resolution
provides:
  - WoW folder detection and validation (WowDetector)
  - Settings persistence via Tauri store plugin (Settings)
  - Tauri dialog plugin for folder selection UI
affects: [3-02-PLAN, 3-03-PLAN, 4-ui-shell]

# Tech tracking
tech-stack:
  added: [tauri-plugin-dialog, tauri-plugin-store]
  patterns: [WowDetector static methods for validation, Settings struct with AppHandle reference]

key-files:
  created:
    - src-tauri/src/install/mod.rs
    - src-tauri/src/install/detector.rs
    - src-tauri/src/install/settings.rs
  modified:
    - src-tauri/Cargo.toml
    - src-tauri/src/lib.rs
    - src-tauri/capabilities/default.json

key-decisions:
  - "WoW validation: check for WoW.exe, WoWFoV.exe, or turtle-wow.exe PLUS Data directory"
  - "Auto-detect checks cwd first, then exe directory"
  - "Settings file: settings.json via tauri-plugin-store"
  - "Store plugin uses Builder::default().build() pattern"

patterns-established:
  - "WowDetector: Static methods for validation, no instance state"
  - "Settings: Borrows AppHandle for store access"

# Metrics
duration: 2min
completed: 2026-01-16
---

# Phase 3 Plan 01: Installation Foundation Summary

**WoW folder detection via exe+Data validation with Tauri dialog/store plugins for folder selection and path persistence**

## Performance

- **Duration:** 2 min
- **Started:** 2026-01-16T21:41:28Z
- **Completed:** 2026-01-16T21:43:43Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments
- Added tauri-plugin-dialog and tauri-plugin-store dependencies
- Created install/ module with WowDetector for folder validation
- Implemented Settings struct for persisting WoW path and selected modules
- Added dialog:default and store:default to capabilities

## Task Commits

Each task was committed atomically:

1. **Task 1-3: Add plugins, detector, and settings** - `b584300` (feat)
   - All tasks completed together since files are interdependent for compilation

**Plan metadata:** (included in same commit)

## Files Created/Modified
- `src-tauri/src/install/mod.rs` - Module exports for detector and settings
- `src-tauri/src/install/detector.rs` - WowDetector with is_valid_wow_folder, auto_detect, get_data_folder
- `src-tauri/src/install/settings.rs` - Settings struct with get/set for wow_path and selected_modules
- `src-tauri/Cargo.toml` - Added tauri-plugin-dialog and tauri-plugin-store
- `src-tauri/src/lib.rs` - Registered dialog and store plugins
- `src-tauri/capabilities/default.json` - Added dialog:default and store:default permissions

## Decisions Made
- WoW folder validation checks for WoW.exe OR WoWFoV.exe OR turtle-wow.exe PLUS Data directory exists
- Auto-detect checks current working directory first, then executable directory
- Settings stored in settings.json via tauri-plugin-store
- Store plugin initialized with Builder::default().build() for proper configuration

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed borrow issue in get_selected_modules**
- **Found during:** Task 1 (initial compilation)
- **Issue:** Chained `.and_then(|v| v.as_array())` caused borrowing error
- **Fix:** Restructured to explicit match statements with early returns
- **Files modified:** src-tauri/src/install/settings.rs
- **Verification:** cargo check passes
- **Committed in:** b584300 (combined commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Fix was necessary for compilation. No scope creep.

## Issues Encountered
None - plan executed smoothly after borrow fix.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- WowDetector ready for use in Tauri commands (next plan)
- Settings ready for persisting user selections
- Dialog plugin registered for folder selection UI
- Ready for 3-02-PLAN.md (Tauri commands for folder selection)

---
*Phase: 3-installation-engine*
*Plan: 01*
*Completed: 2026-01-16*
