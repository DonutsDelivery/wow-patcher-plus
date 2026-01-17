# Turtle WoW HD Patcher

## What This Is

A cross-platform desktop application that automates downloading, installing, and maintaining the HD Patch: Reforged for Turtle WoW. Users select quality presets or individual modules, and the app handles forum parsing, multi-host downloads, and MPQ file installation.

## Core Value

One-click patch installation and repair — users never manually download, unpack, or place MPQ files.

## Requirements

### Validated

- ✓ Parse the Turtle WoW forum post to discover available patches and download links — v1.0
- ✓ Download patch files from external hosts (Google Drive, Mediafire) — v1.0
- ✓ Copy MPQ files to the correct WoW DATA folder — v1.0
- ✓ Provide preset configurations (Low/Medium/High/Ultra) — v1.0
- ✓ Provide individual module toggles for granular control — v1.0
- ✓ Handle module dependencies (e.g., Patch-L requires Patch-A) — v1.0
- ✓ Auto-detect WoW folder if executable is run from inside it — v1.0
- ✓ Remember user's WoW installation path for future runs — v1.0
- ✓ Repair/re-apply patches on demand — v1.0
- ✓ Verify installation integrity — v1.0
- ✓ Desktop GUI with progress indicators — v1.0
- ✓ Cross-platform: Windows, Linux, macOS native builds — v1.0
- ✓ Dark theme UI — v1.0

### Active

(None — define requirements for next milestone)

### Out of Scope

- Auto-detecting Turtle WoW updates — user triggers repair manually
- CLI interface — GUI only for now
- Hosting patch files ourselves — scrape existing forum links
- System tray / background operation — deferred to v2
- Light theme option — deferred to v2

## Context

Shipped v1.0 MVP with ~1,930 LOC (Rust + TypeScript).
Tech stack: Tauri v2, React-TypeScript, scraper crate, reqwest.
CI/CD: GitHub Actions with 4-way cross-platform build matrix.

The HD Patch: Reforged consists of 14 MPQ modules (A-V) with dependencies between some patches. The app parses the forum post to discover current download links, handles Google Drive and Mediafire hosts, and installs to the WoW DATA folder.

User's test installation: `/mnt/storage/Downloads/WoW/`

## Constraints

- **Download sources**: Must scrape forum post for links — no control over host availability
- **File hosts**: External services (Google Drive, Mediafire) may have rate limits or require special handling
- **Cross-platform**: Build for Windows, Linux, and macOS with native executables
- **MPQ format**: Must correctly place files in WoW's DATA folder structure

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Scrape forum post for updates | User doesn't want to host files; forum is source of truth | ✓ Good — works reliably |
| Desktop GUI (no CLI) | Target audience wants simple graphical interface | ✓ Good — Tauri v2 delivers |
| Smart folder detection | Auto-detect if run from WoW folder, else manual browse | ✓ Good — both paths work |
| Repair on demand | Simpler than monitoring for Turtle WoW updates | ✓ Good — UI buttons added |
| Tauri v2 with React-TS | Modern cross-platform framework with Rust backend | ✓ Good — fast dev cycle |
| scraper crate for HTML parsing | Servo's html5ever, reliable CSS selectors | ✓ Good — forum parsing works |
| serde rename for field mapping | Minimal fix for Rust→TypeScript field name mismatch | ✓ Good — no TS changes needed |
| MAX_CONCURRENT_DOWNLOADS = 3 | Avoid overwhelming download servers | ✓ Good — stable downloads |

---
*Last updated: 2026-01-17 after v1.0 milestone*
