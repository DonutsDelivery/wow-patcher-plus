# Turtle WoW HD Patcher

## What This Is

A cross-platform desktop application that automates downloading, installing, and maintaining the HD Patch: Reforged for Turtle WoW. Instead of manually downloading dozens of files from various hosts and placing them correctly, users select their desired modules and the app handles everything.

## Core Value

One-click patch installation and repair — users never manually download, unpack, or place MPQ files.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Parse the Turtle WoW forum post (https://forum.turtlecraft.gg/viewtopic.php?t=21355) to discover available patches and download links
- [ ] Download patch files from external hosts (Google Drive, Mediafire, etc.) as linked in the forum
- [ ] Extract and place MPQ files in the correct WoW DATA folder
- [ ] Provide preset configurations (Low/Medium/High/Ultra) that bundle modules sensibly
- [ ] Provide individual module toggles for users who want granular control
- [ ] Handle module dependencies (e.g., Patch-L requires Patch-A)
- [ ] Auto-detect WoW folder if executable is run from inside it, otherwise user browses to folder
- [ ] Remember user's WoW installation path for future runs
- [ ] Repair/re-apply patches on demand when Turtle WoW updates
- [ ] Desktop GUI with progress indicators for downloads and installation
- [ ] Cross-platform: Windows native, Linux native, macOS native

### Out of Scope

- Auto-detecting Turtle WoW updates — user triggers repair manually
- Auto-detecting WoW installation path — user browses to folder
- CLI interface — GUI only for now
- Hosting patch files ourselves — scrape existing forum links

## Context

The HD Patch: Reforged is a modular graphics enhancement for Turtle WoW (vanilla WoW private server). It consists of:
- Multiple MPQ files (Patch-A through Patch-U) covering different visual categories
- Dependencies between some patches (Patch-L needs Patch-A; Patch-U needs Patch-A and Patch-G)
- VanillaHelpers mod as a mandatory dependency
- DXVK recommended for performance

Current installation is tedious: users must manually download many files from various hosts, unpack archives, and place MPQ files in the DATA folder. After Turtle WoW updates, patches may need reapplication.

Forum post with all patch information: https://forum.turtlecraft.gg/viewtopic.php?t=21355

User's test installation: `/mnt/storage/Downloads/WoW/`

## Constraints

- **Download sources**: Must scrape forum post for links — no control over host availability
- **File hosts**: External services (Google Drive, Mediafire) may have rate limits or require special handling
- **Cross-platform**: Need to build for Windows, Linux, and macOS with native executables
- **MPQ format**: Must correctly place files in WoW's DATA folder structure

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Scrape forum post for updates | User doesn't want to host files; forum is source of truth | — Pending |
| Desktop GUI (no CLI) | Target audience wants simple graphical interface | — Pending |
| Smart folder detection | Auto-detect if run from WoW folder, else manual browse | — Pending |
| Repair on demand | Simpler than monitoring for Turtle WoW updates | — Pending |

---
*Last updated: 2026-01-16 after initialization*
