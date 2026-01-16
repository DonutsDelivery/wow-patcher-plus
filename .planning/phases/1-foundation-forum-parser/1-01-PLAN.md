---
phase: 1-foundation-forum-parser
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src-tauri/Cargo.toml
  - src-tauri/src/lib.rs
  - src-tauri/src/main.rs
  - src-tauri/src/parser/mod.rs
  - src-tauri/src/models/mod.rs
  - src-tauri/tauri.conf.json
  - src-tauri/capabilities/default.json
  - src/App.tsx
  - package.json
autonomous: true

must_haves:
  truths:
    - "Tauri development server starts without errors"
    - "Rust backend compiles with all dependencies"
    - "HTTP plugin is configured and available"
  artifacts:
    - path: "src-tauri/Cargo.toml"
      provides: "Rust dependencies configuration"
      contains: "tauri-plugin-http"
    - path: "src-tauri/src/lib.rs"
      provides: "Tauri plugin registration"
      contains: "tauri_plugin_http"
    - path: "src-tauri/src/parser/mod.rs"
      provides: "Parser module structure"
    - path: "src-tauri/src/models/mod.rs"
      provides: "Models module structure"
  key_links:
    - from: "src-tauri/src/lib.rs"
      to: "tauri_plugin_http"
      via: "plugin registration"
      pattern: "\\.plugin\\(tauri_plugin_http"
---

<objective>
Scaffold a new Tauri v2 project with React frontend and configure all dependencies needed for the forum parser.

Purpose: Establish the foundation for the HD Patcher application with proper project structure and dependencies.
Output: A compilable Tauri project with HTTP capabilities and module structure ready for parser implementation.
</objective>

<execution_context>
@./.claude/get-shit-done/workflows/execute-plan.md
@./.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/1-foundation-forum-parser/1-RESEARCH.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Create Tauri project with React</name>
  <files>
    package.json
    src-tauri/Cargo.toml
    src-tauri/src/main.rs
    src-tauri/src/lib.rs
    src-tauri/tauri.conf.json
    src/App.tsx
  </files>
  <action>
Create a new Tauri v2 project using npm create tauri-app:

```bash
cd "/home/user/Programs/wow patcher"
npm create tauri-app@latest . -- --template react-ts --yes
```

If the directory already contains files and the command fails, use:
```bash
npm create tauri-app@latest temp-tauri -- --template react-ts --yes
mv temp-tauri/* .
mv temp-tauri/.* . 2>/dev/null || true
rm -rf temp-tauri
```

After scaffolding:
1. Update `src-tauri/tauri.conf.json`:
   - Set productName to "Turtle WoW HD Patcher"
   - Set identifier to "com.turtlewow.hdpatcher"
   - Set title to "Turtle WoW HD Patcher"

2. Update `src/App.tsx` with minimal placeholder:
```tsx
function App() {
  return (
    <div className="container">
      <h1>Turtle WoW HD Patcher</h1>
      <p>Loading patch information...</p>
    </div>
  );
}

export default App;
```

3. Install npm dependencies:
```bash
npm install
```
  </action>
  <verify>
Run `npm run tauri dev` - app window should open showing "Turtle WoW HD Patcher" title.
  </verify>
  <done>
Tauri project created with React-TS template, app window opens successfully.
  </done>
</task>

<task type="auto">
  <name>Task 2: Add Rust dependencies and configure HTTP plugin</name>
  <files>
    src-tauri/Cargo.toml
    src-tauri/src/lib.rs
    src-tauri/capabilities/default.json
  </files>
  <action>
1. Update `src-tauri/Cargo.toml` to add required dependencies:

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"
tauri-plugin-http = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
scraper = "0.25"
regex = "1"
url = "2"
thiserror = "1"
tokio = { version = "1", features = ["full"] }
lazy_static = "1.4"
```

2. Update `src-tauri/src/lib.rs` to register the HTTP plugin:

```rust
mod parser;
mod models;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

3. Create or update `src-tauri/capabilities/default.json` to allow HTTP requests:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "opener:default",
    {
      "identifier": "http:default",
      "allow": [
        { "url": "https://forum.turtlecraft.gg/*" },
        { "url": "https://forum.turtle-wow.org/*" },
        { "url": "https://*.mediafire.com/*" },
        { "url": "https://*.google.com/*" }
      ]
    }
  ]
}
```

4. Run `cargo build` in src-tauri to verify dependencies resolve:
```bash
cd "/home/user/Programs/wow patcher/src-tauri"
cargo build
```
  </action>
  <verify>
Run `cargo build` in src-tauri directory - should complete without errors.
Run `cargo check` - should show no warnings about missing modules (parser/models will be empty stubs).
  </verify>
  <done>
All Rust dependencies added and compiling. HTTP plugin registered and configured with forum URL permissions.
  </done>
</task>

<task type="auto">
  <name>Task 3: Create module structure with stubs</name>
  <files>
    src-tauri/src/parser/mod.rs
    src-tauri/src/parser/forum.rs
    src-tauri/src/parser/modules.rs
    src-tauri/src/parser/links.rs
    src-tauri/src/parser/dependencies.rs
    src-tauri/src/models/mod.rs
    src-tauri/src/models/patch.rs
    src-tauri/src/models/download.rs
  </files>
  <action>
Create the module structure as defined in research:

1. Create `src-tauri/src/parser/mod.rs`:
```rust
pub mod forum;
pub mod modules;
pub mod links;
pub mod dependencies;

pub use forum::ForumParser;
pub use modules::parse_modules;
pub use links::extract_download_links;
pub use dependencies::validate_module_selection;
```

2. Create `src-tauri/src/parser/forum.rs`:
```rust
//! Forum fetching and HTML parsing

use crate::models::{ParsedForumPost, ParserError};

pub struct ForumParser {
    // Will hold selectors
}

impl ForumParser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse(&self, _html: &str) -> Result<ParsedForumPost, ParserError> {
        // TODO: Implement in Plan 02
        todo!("Forum parsing not yet implemented")
    }
}

pub async fn fetch_forum_post(_url: &str) -> Result<String, ParserError> {
    // TODO: Implement in Plan 02
    todo!("Forum fetching not yet implemented")
}
```

3. Create `src-tauri/src/parser/modules.rs`:
```rust
//! Patch module parsing from post content

use crate::models::{PatchModule, ParserError};

pub fn parse_modules(_content: &str) -> Result<Vec<PatchModule>, ParserError> {
    // TODO: Implement in Plan 02
    todo!("Module parsing not yet implemented")
}
```

4. Create `src-tauri/src/parser/links.rs`:
```rust
//! Download link extraction using regex

use crate::models::DownloadLink;

pub fn extract_download_links(_content: &str) -> Vec<DownloadLink> {
    // TODO: Implement in Plan 02
    todo!("Link extraction not yet implemented")
}
```

5. Create `src-tauri/src/parser/dependencies.rs`:
```rust
//! Module dependency validation

use crate::models::PatchId;
use std::collections::HashSet;

pub fn validate_module_selection(_selected: &HashSet<PatchId>) -> Result<(), Vec<String>> {
    // TODO: Implement in Plan 02
    todo!("Dependency validation not yet implemented")
}
```

6. Create `src-tauri/src/models/mod.rs`:
```rust
pub mod patch;
pub mod download;

pub use patch::{PatchModule, PatchId};
pub use download::{DownloadLink, DownloadProvider};

/// Parsed forum post content
#[derive(Debug)]
pub struct ParsedForumPost {
    pub content: String,
    pub links: Vec<String>,
}

/// Parser error types
#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("Failed to parse CSS selector: {0}")]
    SelectorParse(String),

    #[error("Forum post not found")]
    PostNotFound,

    #[error("HTTP request failed: {0}")]
    HttpError(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}
```

7. Create `src-tauri/src/models/patch.rs`:
```rust
//! Patch module data structures

use serde::{Deserialize, Serialize};
use crate::models::DownloadLink;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchModule {
    pub id: PatchId,
    pub name: String,
    pub description: String,
    pub downloads: Vec<DownloadLink>,
    pub dependencies: Vec<PatchId>,
    pub file_size: Option<String>,
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatchId {
    A, // Player Characters & NPCs
    B, // Buildings (requires D, E)
    C, // Creatures
    D, // Doodads (requires B, E)
    E, // Environment (requires B, D)
    G, // Gear & Weapons
    I, // Interface
    L, // A Little Extra for Females (requires A)
    M, // Maps & Loading Screens
    N, // Darker Nights
    O, // Raid Visuals Mod (requires S)
    S, // Sounds & Music
    U, // Ultra HD (requires A, G)
    V, // Visual Effects for Spells
}

impl PatchId {
    /// Returns the human-readable name for this patch
    pub fn name(&self) -> &'static str {
        match self {
            PatchId::A => "Player Characters & NPCs",
            PatchId::B => "Buildings",
            PatchId::C => "Creatures",
            PatchId::D => "Doodads",
            PatchId::E => "Environment",
            PatchId::G => "Gear & Weapons",
            PatchId::I => "Interface",
            PatchId::L => "A Little Extra for Females",
            PatchId::M => "Maps & Loading Screens",
            PatchId::N => "Darker Nights",
            PatchId::O => "Raid Visuals Mod",
            PatchId::S => "Sounds & Music",
            PatchId::U => "Ultra HD",
            PatchId::V => "Visual Effects",
        }
    }
}
```

8. Create `src-tauri/src/models/download.rs`:
```rust
//! Download link data structures

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadLink {
    pub provider: DownloadProvider,
    pub url: String,
    pub file_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadProvider {
    Mediafire,
    GoogleDrive,
    Unknown,
}
```

9. Verify the module structure compiles:
```bash
cd "/home/user/Programs/wow patcher/src-tauri"
cargo check
```
  </action>
  <verify>
Run `cargo check` in src-tauri directory - should compile without errors.
All module files exist with proper exports.
  </verify>
  <done>
Module structure created with parser/ and models/ directories. All stubs compile and are ready for implementation in Plan 02.
  </done>
</task>

</tasks>

<verification>
After all tasks complete:

1. Run `npm run tauri dev` from project root - app should launch
2. Run `cargo check` in src-tauri - should compile clean
3. Verify file structure matches research recommendations:
   - src-tauri/src/parser/ exists with forum.rs, modules.rs, links.rs, dependencies.rs
   - src-tauri/src/models/ exists with patch.rs, download.rs
4. Verify HTTP plugin is configured in capabilities/default.json
</verification>

<success_criteria>
- Tauri v2 project scaffolded with React-TS frontend
- All Rust dependencies added and compiling (scraper, regex, url, thiserror, tauri-plugin-http)
- HTTP plugin registered and configured with forum URL permissions
- Parser and models module structure created with stubs
- `npm run tauri dev` launches the application window
- `cargo check` passes without errors
</success_criteria>

<output>
After completion, create `.planning/phases/1-foundation-forum-parser/1-01-SUMMARY.md`
</output>
