# Requirements: Turtle WoW HD Patcher

**Defined:** 2026-01-16
**Core Value:** One-click patch installation and repair — users never manually download, unpack, or place MPQ files.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Forum Parsing

- [x] **PARSE-01**: App can parse Turtle WoW forum post to discover available patches
- [x] **PARSE-02**: App can extract download links from forum post
- [x] **PARSE-03**: App understands module dependencies (e.g., Patch-L requires Patch-A)

### Downloading

- [x] **DL-01**: User can download patch files from Google Drive
- [x] **DL-02**: User can download patch files from Mediafire
- [x] **DL-03**: User sees progress indicators during download
- [x] **DL-04**: Downloads can resume if interrupted (where host supports)
- [x] **DL-05**: Multiple patches can download in parallel

### Installation

- [ ] **INST-01**: App extracts downloaded archives
- [ ] **INST-02**: App places MPQ files in correct WoW DATA folder
- [ ] **INST-03**: User can repair/re-apply patches on demand
- [ ] **INST-04**: App verifies installation integrity after install

### Configuration

- [ ] **CFG-01**: User can select from preset configurations (Low/Medium/High/Ultra)
- [ ] **CFG-02**: User can toggle individual modules on/off
- [ ] **CFG-03**: App auto-detects WoW folder if run from inside it
- [ ] **CFG-04**: App remembers user's WoW installation path

### User Interface

- [ ] **UI-01**: Desktop GUI displays download and installation progress
- [ ] **UI-02**: App builds as native executable for Windows
- [ ] **UI-03**: App builds as native executable for Linux
- [ ] **UI-04**: App builds as native executable for macOS
- [ ] **UI-05**: App uses dark mode theme

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### User Interface

- **UI-V2-01**: System tray / background operation
- **UI-V2-02**: Light theme option

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Auto-detect Turtle WoW updates | User triggers repair manually |
| CLI interface | GUI only for now |
| Hosting patch files | Scrape existing forum links instead |

## Traceability

Which phases cover which requirements. Updated by create-roadmap.

| Requirement | Phase | Status |
|-------------|-------|--------|
| PARSE-01 | Phase 1 | Complete |
| PARSE-02 | Phase 1 | Complete |
| PARSE-03 | Phase 1 | Complete |
| DL-01 | Phase 2 | Complete |
| DL-02 | Phase 2 | Complete |
| DL-03 | Phase 2 | Complete |
| DL-04 | Phase 2 | Complete |
| DL-05 | Phase 2 | Complete |
| INST-01 | Phase 3 | Pending |
| INST-02 | Phase 3 | Pending |
| INST-03 | Phase 3 | Pending |
| INST-04 | Phase 3 | Pending |
| CFG-01 | Phase 4 | Pending |
| CFG-02 | Phase 4 | Pending |
| CFG-03 | Phase 4 | Pending |
| CFG-04 | Phase 4 | Pending |
| UI-01 | Phase 4 | Pending |
| UI-02 | Phase 4 | Pending |
| UI-03 | Phase 4 | Pending |
| UI-04 | Phase 4 | Pending |
| UI-05 | Phase 4 | Pending |

**Coverage:**
- v1 requirements: 21 total
- Mapped to phases: 21 ✓
- Unmapped: 0

---
*Requirements defined: 2026-01-16*
*Last updated: 2026-01-16 after roadmap creation*
