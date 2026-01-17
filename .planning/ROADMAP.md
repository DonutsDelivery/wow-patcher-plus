# Roadmap: Turtle WoW HD Patcher

## Overview

Build a cross-platform desktop application that automates the HD Patch: Reforged installation for Turtle WoW. The journey goes from parsing the forum to discover patches, through downloading from external hosts, to installing MPQ files, and finally wrapping it all in a user-friendly GUI with preset configurations.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

- [x] **Phase 1: Foundation & Forum Parser** - Parse forum post to discover patches and download links
- [x] **Phase 2: Download Engine** - Download files from external hosts with progress tracking
- [x] **Phase 3: Installation Engine** - Copy MPQ files to WoW DATA folder with verification
- [x] **Phase 4: GUI & Configuration** - Desktop interface with presets and module toggles
- [ ] **Phase 5: Integration Fixes** - Fix cross-phase wiring issues found in audit

## Phase Details

### Phase 1: Foundation & Forum Parser
**Goal**: Parse Turtle WoW forum post to discover available patches, download links, and module dependencies
**Depends on**: Nothing (first phase)
**Requirements**: PARSE-01, PARSE-02, PARSE-03
**Success Criteria** (what must be TRUE):
  1. App can fetch and parse the HD Patch forum post
  2. App knows all available patch modules (A-U)
  3. App can list download links for each module
  4. App understands dependency relationships between modules

Plans:
- [x] 1-01: Project scaffolding and Tauri setup (Wave 1)
- [x] 1-02: Parser implementation with models (Wave 2, depends on 1-01)

### Phase 2: Download Engine
**Goal**: Download patch files from external hosts (Google Drive, Mediafire) with progress tracking and parallel downloads
**Depends on**: Phase 1
**Requirements**: DL-01, DL-02, DL-03, DL-04, DL-05
**Success Criteria** (what must be TRUE):
  1. User can download files from Google Drive links
  2. User can download files from Mediafire links
  3. User sees download progress (speed, percentage)
  4. Interrupted downloads can resume (where supported by host)
  5. Multiple downloads can run in parallel

Plans:
- [x] 2-01: Core download infrastructure (Wave 1) - Provider trait, streaming engine, progress types
- [x] 2-02: Google Drive provider (Wave 2, depends on 2-01) - URL resolution, virus scan handling
- [x] 2-03: Mediafire provider (Wave 2, depends on 2-01) - URL resolution, dynamic subdomain handling
- [x] 2-04: Resume, parallel manager, Tauri commands (Wave 3, depends on 2-02, 2-03)

### Phase 3: Installation Engine
**Goal**: Copy downloaded MPQ files to WoW DATA folder with verification and repair
**Depends on**: Phase 2
**Requirements**: INST-01, INST-02, INST-03, INST-04
**Note**: Research found HD Patch distributes raw MPQ files, not archives. No extraction needed.
**Success Criteria** (what must be TRUE):
  1. Downloaded MPQ files are copied to WoW DATA folder
  2. User can select WoW folder via native dialog
  3. User can repair/re-apply patches on demand
  4. Installation integrity is verified after install

Plans:
- [x] 3-01: Tauri plugins + WoW detection + Settings (Wave 1)
- [x] 3-02: MPQ copier + Verification logic (Wave 2, depends on 3-01)
- [x] 3-03: Repair + Tauri commands (Wave 3, depends on 3-01, 3-02)

### Phase 4: GUI & Configuration
**Goal**: Desktop application with quality presets, individual module toggles, and cross-platform builds
**Depends on**: Phase 3
**Requirements**: CFG-01, CFG-02, CFG-03, CFG-04, UI-01, UI-02, UI-03, UI-04, UI-05
**Success Criteria** (what must be TRUE):
  1. User can select quality presets (Low/Medium/High/Ultra)
  2. User can toggle individual modules on/off
  3. App auto-detects WoW folder if run from inside it
  4. App remembers WoW installation path between sessions
  5. Desktop GUI shows download/install progress
  6. Native builds work on Windows, Linux, and macOS
  7. App uses dark theme

Plans:
- [x] 4-01: Tailwind/shadcn setup + dark theme (Wave 1)
- [x] 4-02: PresetSelector, ModuleList, FolderPicker components (Wave 2, depends on 4-01)
- [x] 4-03: Progress components + main App workflow (Wave 3, depends on 4-02)
- [x] 4-04: Cross-platform CI/CD + bundle config (Wave 4, depends on 4-03)

### Phase 5: Integration Fixes
**Goal**: Fix cross-phase wiring issues that will cause runtime failures
**Depends on**: Phase 4
**Gap Closure**: Closes gaps from v1-MILESTONE-AUDIT.md
**Success Criteria** (what must be TRUE):
  1. Frontend can access download URLs from parsed modules (type mismatch fixed)
  2. Downloaded files match the naming convention expected by installer
  3. User can trigger verify and repair from the UI

Plans:
- [ ] 5-01: Type mismatch + filename convention + repair UI (Wave 1)

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation & Forum Parser | 2/2 | Complete | 2026-01-16 |
| 2. Download Engine | 4/4 | Complete | 2026-01-16 |
| 3. Installation Engine | 3/3 | Complete | 2026-01-16 |
| 4. GUI & Configuration | 4/4 | Complete | 2026-01-17 |
| 5. Integration Fixes | 0/1 | Pending | — |

---
*Created: 2026-01-16*
*Milestone: v1.0 MVP*
