# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-01-16)

**Core value:** One-click patch installation and repair — users never manually download, unpack, or place MPQ files.
**Current focus:** Phase 4 — GUI Configuration (In Progress)

## Current Position

Phase: 4 of 4 (GUI Configuration)
Plan: 1/4 complete
Status: In progress
Last activity: 2026-01-16 — Completed 4-01-PLAN.md (Tailwind + shadcn/ui Foundation)

Progress: █████████▓ 92%

## Performance Metrics

**Velocity:**
- Total plans completed: 10
- Average duration: 4 min
- Total execution time: 40 min

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 1-foundation | 2/2 | 10 min | 5 min |
| 2-download-engine | 4/4 | 16 min | 4 min |
| 3-installation-engine | 3/3 | 10 min | 3 min |
| 4-gui-configuration | 1/4 | 4 min | 4 min |

**Recent Trend:**
- Last 5 plans: 2-04 (4 min), 3-01 (2 min), 3-02 (4 min), 3-03 (4 min), 4-01 (4 min)
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

### Pending Todos

None.

### Blockers/Concerns

- Node.js version (20.18.1) lower than Vite requirement (20.19+) - warning only, not blocking

## Session Continuity

Last session: 2026-01-16T22:24:38Z
Stopped at: Completed 4-01-PLAN.md
Resume file: None

## Next Steps

Phase 4 (GUI Configuration) in progress. Next:
1. Execute 4-02-PLAN.md: Application shell layout

UI foundation complete:
- Tailwind CSS v4 with Vite plugin
- shadcn/ui components: Button, Card, Checkbox, Progress, ScrollArea, Select, Switch
- Dark theme with OKLCH CSS variables
- Path alias @/* configured
- cn() utility for class merging
