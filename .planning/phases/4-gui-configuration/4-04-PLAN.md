---
phase: 4-gui-configuration
plan: 04
type: execute
wave: 4
depends_on: ["4-03"]
files_modified:
  - .github/workflows/release.yml
  - src-tauri/tauri.conf.json
  - src-tauri/icons/icon.png
autonomous: true
user_setup:
  - service: github
    why: "Release workflow requires repository access"
    env_vars: []
    dashboard_config:
      - task: "Enable GitHub Actions if not already enabled"
        location: "Repository Settings -> Actions -> General"

must_haves:
  truths:
    - "GitHub Actions workflow builds for Windows"
    - "GitHub Actions workflow builds for Linux"
    - "GitHub Actions workflow builds for macOS (x86_64 and aarch64)"
    - "Release artifacts are uploaded to GitHub releases"
    - "App has proper bundle identifier and metadata"
  artifacts:
    - path: ".github/workflows/release.yml"
      provides: "Cross-platform build workflow"
      contains: "tauri-apps/tauri-action"
    - path: "src-tauri/tauri.conf.json"
      provides: "Bundle configuration with icons and identifier"
      contains: "bundle"
    - path: "src-tauri/icons/icon.png"
      provides: "Source icon for all platforms"
  key_links:
    - from: ".github/workflows/release.yml"
      to: "tauri-apps/tauri-action@v0"
      via: "uses directive"
      pattern: "tauri-apps/tauri-action"
    - from: "src-tauri/tauri.conf.json"
      to: "src-tauri/icons/*"
      via: "icon paths"
      pattern: "icons/"
---

<objective>
Configure cross-platform builds via GitHub Actions and finalize app bundle settings.

Purpose: Enable native builds for Windows, Linux, and macOS to fulfill UI-02, UI-03, UI-04 requirements.
Output: Working CI/CD pipeline that produces release artifacts for all platforms.
</objective>

<execution_context>
@./.claude/get-shit-done/workflows/execute-plan.md
@./.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/phases/4-gui-configuration/4-RESEARCH.md
@.planning/phases/4-gui-configuration/4-03-SUMMARY.md
@src-tauri/tauri.conf.json
</context>

<tasks>

<task type="auto">
  <name>Task 1: Create GitHub Actions release workflow</name>
  <files>.github/workflows/release.yml</files>
  <action>
Create .github/workflows/release.yml with cross-platform build matrix from 4-RESEARCH.md:

```yaml
name: 'Build and Release'

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

jobs:
  build:
    permissions:
      contents: write
    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: 'macos-latest'
            args: '--target aarch64-apple-darwin'
          - platform: 'macos-latest'
            args: '--target x86_64-apple-darwin'
          - platform: 'ubuntu-22.04'
            args: ''
          - platform: 'windows-latest'
            args: ''

    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4

      - name: Install dependencies (Ubuntu only)
        if: matrix.platform == 'ubuntu-22.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: lts/*
          cache: 'npm'

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.platform == 'macos-latest' && 'aarch64-apple-darwin,x86_64-apple-darwin' || '' }}

      - name: Rust cache
        uses: swatinem/rust-cache@v2
        with:
          workspaces: './src-tauri -> target'

      - name: Install frontend dependencies
        run: npm install

      - uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tagName: v__VERSION__
          releaseName: 'Turtle WoW HD Patcher v__VERSION__'
          releaseBody: 'See assets below to download for your platform.'
          releaseDraft: true
          prerelease: false
          args: ${{ matrix.args }}
```

Key points:
- Triggers on version tags (v*) or manual dispatch
- Matrix builds for: macOS ARM64, macOS x86_64, Ubuntu (deb), Windows (NSIS)
- Ubuntu needs webkit/appindicator/librsvg/patchelf
- macOS needs both ARM64 and x86_64 targets
- Uses tauri-action@v0 for building and releasing
- Creates draft release (user can review before publishing)
  </action>
  <verify>
File exists at .github/workflows/release.yml.
YAML syntax is valid.
  </verify>
  <done>GitHub Actions workflow created for cross-platform builds.</done>
</task>

<task type="auto">
  <name>Task 2: Update tauri.conf.json with bundle configuration</name>
  <files>src-tauri/tauri.conf.json</files>
  <action>
Update src-tauri/tauri.conf.json bundle section with proper metadata:

Ensure these fields are set in the bundle section:
```json
{
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "identifier": "com.turtlewow.hdpatcher",
    "shortDescription": "Automated HD Patch installer for Turtle WoW",
    "longDescription": "Download and install HD texture packs for Turtle WoW with automatic dependency resolution, quality presets, and progress tracking.",
    "category": "Game",
    "copyright": "MIT License"
  }
}
```

Also ensure app section has correct title:
```json
{
  "app": {
    "windows": [
      {
        "title": "Turtle WoW HD Patcher",
        "width": 800,
        "height": 700,
        "resizable": true,
        "fullscreen": false
      }
    ]
  }
}
```
  </action>
  <verify>
`npm run tauri build` starts without config errors (may fail on icons, that's expected).
JSON is valid.
  </verify>
  <done>tauri.conf.json configured with bundle identifier, description, and window settings.</done>
</task>

<task type="auto">
  <name>Task 3: Generate application icons</name>
  <files>src-tauri/icons/icon.png, src-tauri/icons/*</files>
  <action>
Tauri needs icons in multiple formats. The simplest approach:

1. Check if icons already exist:
```bash
ls src-tauri/icons/
```

2. If icons exist (from Tauri scaffold), they are placeholder icons - sufficient for builds.

3. If custom icons needed later, create a 1024x1024 PNG and run:
```bash
npm run tauri icon path/to/icon.png
```

For now, verify placeholder icons exist in src-tauri/icons/:
- 32x32.png
- 128x128.png
- 128x128@2x.png
- icon.icns (macOS)
- icon.ico (Windows)

If missing, create directory and add placeholder:
```bash
mkdir -p src-tauri/icons
```

Then use Tauri's icon generator with any PNG (can use a simple placeholder).

Note: Custom branded icons can be added later. The build system requires these files to exist but doesn't care about their content for testing.
  </action>
  <verify>
`npm run tauri build` completes successfully (at least on current platform).
Icons directory contains required files.
  </verify>
  <done>Application icons configured. Build produces native executable.</done>
</task>

</tasks>

<verification>
1. `.github/workflows/release.yml` exists with valid YAML
2. `src-tauri/tauri.conf.json` has bundle section with identifier
3. `src-tauri/icons/` contains required icon files
4. `npm run tauri build` produces executable for current platform
5. Workflow can be triggered via manual dispatch (workflow_dispatch)
</verification>

<success_criteria>
- CI workflow builds on Windows, Linux, macOS (all 4 matrix configurations)
- Release artifacts include: .exe (Windows), .deb (Linux), .dmg/.app (macOS)
- App identifier is "com.turtlewow.hdpatcher"
- Window title shows "Turtle WoW HD Patcher"
- Local `npm run tauri build` produces working executable
</success_criteria>

<output>
After completion, create `.planning/phases/4-gui-configuration/4-04-SUMMARY.md`
</output>
