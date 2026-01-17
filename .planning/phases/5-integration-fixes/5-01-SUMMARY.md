---
phase: 5-integration-fixes
plan: 01
subsystem: integration
tags: [serde, tauri, typescript, download, install, ui]

# Dependency graph
requires:
  - phase: 4-gui-configuration
    provides: "UI components, download/install hooks, App state machine"
  - phase: 3-installation-engine
    provides: "InstallManager, verify/repair commands"
  - phase: 2-download-engine
    provides: "DownloadManager with provider resolution"
provides:
  - "Fixed PatchModule serialization (downloads -> links)"
  - "Custom filename support for downloads (Patch-{ID}.mpq)"
  - "Verify Installation button in complete state"
  - "Repair Installation button with progress tracking"
affects: [release]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "serde(rename) for field name mismatches between Rust and TypeScript"
    - "Optional parameter passthrough from TypeScript to Rust command"

key-files:
  created: []
  modified:
    - src-tauri/src/models/patch.rs
    - src-tauri/src/lib.rs
    - src-tauri/src/download/manager.rs
    - src/lib/tauri.ts
    - src/hooks/useDownload.ts
    - src/hooks/useInstall.ts
    - src/App.tsx

key-decisions:
  - "Use serde(rename) to fix field name mismatch - minimal change, no TypeScript modification needed"
  - "targetFilename as optional parameter - backwards compatible if called without it"
  - "Export setInstalls from useInstall hook for repair progress tracking"

patterns-established:
  - "serde(rename = \"x\") pattern for Rust->TypeScript field name mapping"
  - "Optional<String> for filename override in download pipeline"

# Metrics
duration: 4min
completed: 2026-01-17
---

# Phase 5 Plan 01: Integration Fixes Summary

**Serde rename for field serialization, custom download filename support, and Verify/Repair UI buttons**

## Performance

- **Duration:** 4 min
- **Started:** 2026-01-17T02:16:23Z
- **Completed:** 2026-01-17T02:19:51Z
- **Tasks:** 4
- **Files modified:** 7

## Accomplishments
- Fixed Rust->TypeScript field name mismatch (downloads->links) with serde rename
- Added target_filename parameter to download pipeline for Patch-{ID}.mpq naming
- Added Verify and Repair buttons to complete state UI
- Verification results display with status coloring

## Task Commits

Each task was committed atomically:

1. **Task 1: Fix type field mismatch with serde rename** - `fbf741e` (fix)
2. **Task 2a: Add target_filename parameter to Rust download pipeline** - `2c690bd` (feat)
3. **Task 2b: Update TypeScript to pass Patch-{ID}.mpq filename** - `77434d0` (feat)
4. **Task 3: Add Verify and Repair UI buttons** - `60aded4` (feat)

## Files Created/Modified
- `src-tauri/src/models/patch.rs` - Added serde(rename = "links") to downloads field
- `src-tauri/src/lib.rs` - Added target_filename parameter to start_download command
- `src-tauri/src/download/manager.rs` - Added target_filename parameter to download method
- `src/lib/tauri.ts` - Added targetFilename parameter to startDownload function
- `src/hooks/useDownload.ts` - Pass Patch-{ID}.mpq filename to startDownload
- `src/hooks/useInstall.ts` - Export setInstalls for repair progress tracking
- `src/App.tsx` - Added handleVerify, handleRepair functions and UI buttons

## Decisions Made
- **Serde rename approach:** Minimal fix on Rust side only, TypeScript already expects `links`
- **Optional filename parameter:** Backwards compatible - if not provided, uses provider filename
- **Verify/Repair as separate buttons:** Users can check status without re-installing

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- TypeScript type mismatch in handleRepair (used bytesCopied instead of copiedBytes) - fixed by matching InstallState interface

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- All v1 integration gaps closed
- Ready for release tagging
- Blocker: None

---
*Phase: 5-integration-fixes*
*Completed: 2026-01-17*
