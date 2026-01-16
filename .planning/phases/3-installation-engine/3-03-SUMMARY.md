---
phase: 3-installation-engine
plan: 03
subsystem: install
tags: [tauri, repair, manager, dialog, settings, commands]

# Dependency graph
requires:
  - phase: 3-installation-engine
    provides: WowDetector, Settings, install_mpq, verify_patch
provides:
  - repair_patch and repair_all for re-copying from downloads
  - InstallManager coordinating all install operations
  - Tauri commands for frontend integration
affects: [4-ui-integration]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - InstallManager with RwLock for thread-safe path storage
    - FilePath.into_path() for dialog file path conversion

key-files:
  created:
    - src-tauri/src/install/repair.rs
    - src-tauri/src/install/manager.rs
  modified:
    - src-tauri/src/install/mod.rs
    - src-tauri/src/lib.rs

key-decisions:
  - "FilePath.into_path() for converting dialog selections to PathBuf"
  - "InstallManager registered via setup() with app data downloads directory"

patterns-established:
  - "Tauri command pattern: State<'_, Manager> for managed state access"
  - "Channel<Event> for progress event streaming from backend"

# Metrics
duration: 4min
completed: 2026-01-16
---

# Phase 3 Plan 3: Batch Installation and Tauri Commands Summary

**Repair functionality with InstallManager coordinating install/verify/repair operations, exposed via 7 Tauri commands for frontend integration**

## Performance

- **Duration:** 4 min
- **Started:** 2026-01-16T21:50:30Z
- **Completed:** 2026-01-16T21:54:52Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments
- Repair functionality for re-copying patches from downloads folder
- InstallManager providing centralized coordination of all operations
- Full Tauri command API for frontend: select_wow_folder, get_wow_path, auto_detect_wow, install_patches, verify_patches, repair_patches, load_saved_wow_path
- Settings persistence integration with folder selection and auto-detect

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement repair functionality** - `e0a86d4` (feat)
2. **Task 2: Create InstallManager** - `46f301f` (feat)
3. **Task 3: Add Tauri commands** - `959ed53` (feat)

## Files Created/Modified
- `src-tauri/src/install/repair.rs` - Repair logic: repair_patch, repair_all, RepairResult, patches_needing_repair
- `src-tauri/src/install/manager.rs` - InstallManager coordinating install/verify/repair with path management
- `src-tauri/src/install/mod.rs` - Module exports for repair and manager
- `src-tauri/src/lib.rs` - 7 new Tauri commands and InstallManager registration

## Decisions Made
- Use FilePath.into_path() to convert tauri_plugin_dialog FilePath enum to PathBuf (Tauri v2 API change)
- Register InstallManager in setup() closure with app data downloads directory
- Downloads stored in app_data_dir()/downloads for cross-platform compatibility

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed FilePath API usage**
- **Found during:** Task 3 (Tauri commands implementation)
- **Issue:** Plan showed `f.path` but Tauri v2 FilePath is an enum, not a struct
- **Fix:** Changed to `file_path.into_path().map_err(...)` to properly convert FilePath to PathBuf
- **Files modified:** src-tauri/src/lib.rs
- **Verification:** cargo check passes, no compilation errors
- **Committed in:** 959ed53 (Task 3 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** API correction necessary for compilation. No scope creep.

## Issues Encountered
None - execution proceeded smoothly after API fix.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Installation engine complete with full Tauri command API
- Frontend can now:
  - Select WoW folder via native dialog
  - Load saved settings on startup
  - Auto-detect WoW installation
  - Install patches with progress events
  - Verify installed patches
  - Repair patches from downloads
- Ready for Phase 4: UI Integration

---
*Phase: 3-installation-engine*
*Completed: 2026-01-16*
