# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-01-16)

**Core value:** One-click patch installation and repair — users never manually download, unpack, or place MPQ files.
**Current focus:** Phase 4 — GUI Configuration (In Progress)

## Current Position

Phase: 4 of 4 (GUI Configuration)
Plan: 3/4 complete
Status: In progress
Last activity: 2026-01-16 — Completed 4-03-PLAN.md (Download/Install Progress and App Integration)

Progress: ██████████ 97%

## Performance Metrics

**Velocity:**
- Total plans completed: 12
- Average duration: 4 min
- Total execution time: 45 min

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 1-foundation | 2/2 | 10 min | 5 min |
| 2-download-engine | 4/4 | 16 min | 4 min |
| 3-installation-engine | 3/3 | 10 min | 3 min |
| 4-gui-configuration | 3/4 | 9 min | 3 min |

**Recent Trend:**
- Last 5 plans: 3-02 (4 min), 3-03 (4 min), 4-01 (4 min), 4-02 (2 min), 4-03 (3 min)
- Trend: Consistent velocity, fast execution

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- Use Tauri v2 with React-TS frontend
- Use scraper crate for HTML parsing (Servo's html5ever)
- Use tauri-plugin-http (reqwest) for HTTP requests
- Forum URL: https://forum.turtlecraft.gg/viewtopic.php?t=21355
- 14 patch modules: A, B, C, D, E, G, I, L, M, N, O, S, U, V
- Dependencies: B+D+E together, L->A, U->A+G, O->S
- Package name: turtle-wow-hd-patcher (from 1-01)
- HTTP permissions scoped to forum and download host domains only (from 1-01)
- Use div.postbody div.content selector for phpBB post extraction (from 1-02)
- Implement URL fallback from turtlecraft.gg to turtle-wow.org (from 1-02)
- 100ms throttle for progress events to prevent UI flooding (from 2-01)
- Use bytes_stream() + StreamExt for memory-efficient downloads (from 2-01)
- DownloadProvider trait for URL resolution abstraction (from 2-01)
- Use regex for Google Drive file ID extraction (from 2-02)
- Use regex for Mediafire download URL extraction with numbered subdomains (from 2-03)
- 1.5s delay before dkey URL fetch to avoid rate limiting (from 2-03)
- MAX_CONCURRENT_DOWNLOADS = 3 with Semaphore limiting (from 2-04)
- Spawn async task for non-blocking download initiation (from 2-04)
- HTTP Range header pattern: bytes={start}- for resume (from 2-04)
- WoW validation: check for WoW.exe, WoWFoV.exe, or turtle-wow.exe PLUS Data directory (from 3-01)
- Settings stored in settings.json via tauri-plugin-store (from 3-01)
- Store plugin uses Builder::default().build() pattern (from 3-01)
- 64KB buffer for chunked MPQ copy (COPY_BUFFER_SIZE) (from 3-02)
- 100ms throttle for install progress events (matches download pattern) (from 3-02)
- Size-based verification for installed patches (from 3-02)
- FilePath.into_path() for dialog file path conversion (from 3-03)
- InstallManager registered via setup() with app data downloads directory (from 3-03)
- Use Tailwind CSS v4 with @tailwindcss/vite plugin (from 4-01)
- Use tw-animate-css (replaced tailwindcss-animate in March 2025) (from 4-01)
- Use OKLCH color space for CSS variables (from 4-01)
- Force dark mode on app load via classList.add('dark') (from 4-01)
- New York style for shadcn/ui with neutral base color (from 4-01)
- Presets: low (I,M), medium (A,C,G,I,M,V), high (+B,D,E,S), ultra (+U) (from 4-02)
- usePatches returns Set<string> for O(1) selection lookups (from 4-02)
- useWowPath tries saved path first, then auto-detect on mount (from 4-02)
- Use Channel.onmessage callback for streaming updates from Rust (from 4-03)
- Map<string, State> for concurrent download/install tracking (from 4-03)
- App state machine: configure -> downloading -> installing -> complete (from 4-03)

### Pending Todos

None.

### Blockers/Concerns

- Node.js version (20.18.1) lower than Vite requirement (20.19+) - warning only, not blocking

## Session Continuity

Last session: 2026-01-16T22:33:04Z
Stopped at: Completed 4-03-PLAN.md
Resume file: None

## Next Steps

Phase 4 (GUI Configuration) in progress. Next:
1. Execute 4-04-PLAN.md: Final polish and testing

Complete functional UI assembled:
- Download progress with Channel streaming
- Install progress with Channel streaming
- Full workflow state machine in App.tsx
- All components integrated and building
