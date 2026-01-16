# Roadmap: Turtle WoW HD Patcher

## Overview

Build a cross-platform desktop application that automates the HD Patch: Reforged installation for Turtle WoW. The journey goes from parsing the forum to discover patches, through downloading from external hosts, to installing MPQ files, and finally wrapping it all in a user-friendly GUI with preset configurations.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

- [ ] **Phase 1: Foundation & Forum Parser** - Parse forum post to discover patches and download links
- [ ] **Phase 2: Download Engine** - Download files from external hosts with progress tracking
- [ ] **Phase 3: Installation Engine** - Extract archives and install MPQ files to WoW
- [ ] **Phase 4: GUI & Configuration** - Desktop interface with presets and module toggles

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
- [ ] 1-01: Project scaffolding and Tauri setup (Wave 1)
- [ ] 1-02: Parser implementation with models (Wave 2, depends on 1-01)

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
**Plans**: TBD

Plans:
- [ ] 02-01: Download engine implementation

### Phase 3: Installation Engine
**Goal**: Extract downloaded archives and install MPQ files to WoW DATA folder with verification
**Depends on**: Phase 2
**Requirements**: INST-01, INST-02, INST-03, INST-04
**Success Criteria** (what must be TRUE):
  1. Downloaded archives are extracted automatically
  2. MPQ files are placed in correct WoW DATA folder
  3. User can repair/re-apply patches on demand
  4. Installation integrity is verified after install
**Plans**: TBD

Plans:
- [ ] 03-01: Installation engine implementation

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
**Plans**: TBD

Plans:
- [ ] 04-01: GUI implementation with Tauri

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation & Forum Parser | 0/2 | Planned | - |
| 2. Download Engine | 0/TBD | Not started | - |
| 3. Installation Engine | 0/TBD | Not started | - |
| 4. GUI & Configuration | 0/TBD | Not started | - |

---
*Created: 2026-01-16*
*Milestone: v1.0 MVP*
