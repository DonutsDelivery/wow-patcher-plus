---
phase: 5-integration-fixes
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src-tauri/src/models/patch.rs
  - src-tauri/src/lib.rs
  - src-tauri/src/download/manager.rs
  - src/hooks/useDownload.ts
  - src/lib/tauri.ts
  - src/App.tsx
autonomous: true
gap_closure: true

must_haves:
  truths:
    - "Frontend can access download URLs via module.links"
    - "Downloaded files are saved as Patch-{ID}.mpq"
    - "Installer finds downloaded files by expected naming convention"
    - "User can trigger verify from complete state"
    - "User can trigger repair from complete state"
  artifacts:
    - path: "src-tauri/src/models/patch.rs"
      provides: "PatchModule with serde rename on downloads field"
      contains: "serde(rename"
    - path: "src/hooks/useDownload.ts"
      provides: "Download hook that saves files as Patch-{ID}.mpq"
      contains: "Patch-"
    - path: "src/App.tsx"
      provides: "Verify and Repair buttons in complete state"
      contains: "verifyPatches"
  key_links:
    - from: "src-tauri/src/models/patch.rs"
      to: "src/lib/tauri.ts"
      via: "serde serialization"
      pattern: "links.*provider.*url"
    - from: "src/hooks/useDownload.ts"
      to: "src-tauri/src/install/manager.rs"
      via: "filename convention"
      pattern: "Patch-.*\\.mpq"
---

<objective>
Fix three cross-phase integration gaps identified in the v1 milestone audit that will cause runtime failures.

Purpose: Close critical wiring issues before v1.0 release - without these fixes, the app will crash or fail silently when users try to install patches.

Output:
- Type field mismatch fixed (Rust downloads -> JSON links)
- Downloads saved with expected Patch-{ID}.mpq naming
- Verify/Repair UI buttons added to complete state
</objective>

<execution_context>
@./.claude/get-shit-done/workflows/execute-plan.md
@./.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/v1-MILESTONE-AUDIT.md

Key files:
@src-tauri/src/models/patch.rs
@src/lib/tauri.ts
@src/hooks/useDownload.ts
@src/App.tsx
@src-tauri/src/install/copier.rs (get_mpq_filename function)
</context>

<tasks>

<task type="auto">
  <name>Task 1: Fix type field mismatch with serde rename</name>
  <files>src-tauri/src/models/patch.rs</files>
  <action>
Add `#[serde(rename = "links")]` attribute to the `downloads` field in `PatchModule` struct.

This makes Rust serialize the field as "links" in JSON, matching what TypeScript expects:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchModule {
    pub id: PatchId,
    pub name: String,
    pub description: String,
    #[serde(rename = "links")]  // <-- Add this line
    pub downloads: Vec<DownloadLink>,
    pub dependencies: Vec<PatchId>,
    pub file_size: Option<String>,
    pub last_updated: Option<String>,
}
```

This is the minimal fix - no TypeScript changes required because frontend already expects `links`.
  </action>
  <verify>
Run `cargo check` in src-tauri to verify the code compiles.
Run `cargo test` in src-tauri to verify no test regressions.
  </verify>
  <done>
PatchModule serializes `downloads` field as `links` in JSON output.
Frontend code `module.links[0]` will now access the correct data.
  </done>
</task>

<task type="auto">
  <name>Task 2a: Add target_filename parameter to Rust download pipeline</name>
  <files>src-tauri/src/lib.rs, src-tauri/src/download/manager.rs</files>
  <action>
The installer expects files named `Patch-{ID}.mpq` (from copier.rs:165), but downloads save with provider's original filename. This task adds the Rust backend support for custom filenames.

1. Update src-tauri/src/lib.rs - add `target_filename` parameter to start_download command:
```rust
async fn start_download(
    manager: State<'_, DownloadManager>,
    share_url: String,
    provider: String,
    dest_dir: String,
    on_progress: Channel<DownloadEvent>,
    target_filename: Option<String>,  // Add this parameter
) -> Result<String, String>
```

Pass target_filename through to manager.download().

2. Update src-tauri/src/download/manager.rs - add `target_filename` parameter to download method:
```rust
pub async fn download(
    &self,
    share_url: String,
    provider_type: ProviderType,
    dest_dir: PathBuf,
    download_id: String,
    on_event: Channel<DownloadEvent>,
    target_filename: Option<String>,  // Add this parameter
) -> Result<String, DownloadError>
```

In the download method, use target_filename if provided instead of provider-supplied filename:
```rust
let file_name = target_filename.unwrap_or_else(|| {
    // existing filename logic from provider info
});
```
  </action>
  <verify>
Run `cargo check` in src-tauri to verify the code compiles.
Run `cargo test` in src-tauri to verify no test regressions.
  </verify>
  <done>
Rust download pipeline accepts optional target_filename parameter.
When provided, downloaded files are saved with the specified filename.
  </done>
</task>

<task type="auto">
  <name>Task 2b: Update TypeScript to pass Patch-{ID}.mpq filename</name>
  <files>src/lib/tauri.ts, src/hooks/useDownload.ts</files>
  <action>
Wire the frontend to pass the expected filename through to the Rust backend.

1. Update src/lib/tauri.ts - add `targetFilename` parameter to startDownload:
```typescript
export async function startDownload(
  shareUrl: string,
  provider: string,
  destDir: string,
  onProgress: Channel<DownloadEvent>,
  targetFilename?: string
): Promise<string> {
  return invoke('start_download', { shareUrl, provider, destDir, onProgress, targetFilename });
}
```

2. Update src/hooks/useDownload.ts - pass the expected filename in startModuleDownload:
```typescript
const targetFilename = `Patch-${module.id.toUpperCase()}.mpq`;
const downloadId = await startDownload(link.url, link.provider, destDir, onProgress, targetFilename);
```
  </action>
  <verify>
Run `npm run build` to verify TypeScript compiles without errors.
  </verify>
  <done>
Downloads are saved as `Patch-{ID}.mpq` format.
Installer can find downloaded files in downloads folder using expected naming convention.
  </done>
</task>

<task type="auto">
  <name>Task 3: Add Verify and Repair UI buttons</name>
  <files>src/App.tsx</files>
  <action>
Add Verify and Repair buttons to the complete state section of App.tsx.

1. Import verifyPatches and repairPatches from '@/lib/tauri'
2. Add state for verification results: `const [verifyResults, setVerifyResults] = useState<Map<string, string>>(new Map());`
3. Add state for repair in progress: `const [repairing, setRepairing] = useState(false);`
4. Create handleVerify function:
   ```typescript
   const handleVerify = async () => {
     const results = await verifyPatches(Array.from(selectedModules));
     const resultMap = new Map<string, string>();
     results.forEach(([id, result]) => {
       resultMap.set(id, result.status);
     });
     setVerifyResults(resultMap);
   };
   ```
5. Create handleRepair function:
   ```typescript
   const handleRepair = async () => {
     setRepairing(true);
     const onEvent = new Channel<InstallEvent>();
     onEvent.onmessage = (msg) => {
       // Reuse install progress tracking
       setInstalls(prev => {
         const next = new Map(prev);
         // ... same logic as useInstall
         return next;
       });
     };
     await repairPatches(Array.from(selectedModules), onEvent);
     setRepairing(false);
   };
   ```

   Actually, simpler - just use the existing useInstall hook's repair capability. The hook already has install tracking.

6. Update the complete state section to show Verify/Repair buttons:
   ```tsx
   {appState === 'complete' && (
     <Card>
       <CardHeader>
         <CardTitle className="text-green-500">Installation Complete!</CardTitle>
         <CardDescription>You can now launch Turtle WoW.</CardDescription>
       </CardHeader>
       <CardContent className="space-y-4">
         <div className="flex gap-4 justify-center">
           <Button variant="outline" onClick={handleVerify}>
             Verify Installation
           </Button>
           <Button variant="outline" onClick={handleRepair} disabled={repairing}>
             {repairing ? 'Repairing...' : 'Repair Installation'}
           </Button>
         </div>
         {verifyResults.size > 0 && (
           <div className="text-sm space-y-1">
             {Array.from(verifyResults.entries()).map(([id, status]) => (
               <div key={id} className={status === 'ok' ? 'text-green-500' : 'text-red-500'}>
                 Patch {id}: {status}
               </div>
             ))}
           </div>
         )}
       </CardContent>
     </Card>
   )}
   ```

7. Import Channel from '@tauri-apps/api/core' and InstallEvent from '@/lib/tauri'
  </action>
  <verify>
Run `npm run build` to verify TypeScript compiles
Run `npm run dev` and visually check that Verify/Repair buttons appear after installation
  </verify>
  <done>
User can click "Verify Installation" button to check installed patches.
User can click "Repair Installation" button to re-copy patches.
Verification results displayed to user.
  </done>
</task>

</tasks>

<verification>
After all tasks complete:

1. Build verification:
   - `cd src-tauri && cargo check` passes
   - `cd src-tauri && cargo test` passes
   - `npm run build` passes

2. Type verification (manual inspection):
   - Rust PatchModule has `#[serde(rename = "links")]` on downloads field
   - TypeScript PatchModule interface has `links` field (unchanged)
   - Frontend `module.links[0]` will now receive data

3. Filename convention verification:
   - useDownload.ts passes `Patch-{ID}.mpq` as targetFilename
   - DownloadManager uses targetFilename when saving
   - InstallManager looks for `Patch-{ID}.mpq` (from get_mpq_filename)

4. UI verification:
   - App.tsx imports verifyPatches, repairPatches
   - Complete state shows Verify and Repair buttons
   - Buttons are wired to call the respective functions
</verification>

<success_criteria>
1. Type mismatch fixed - `module.links[0]` returns first download link
2. Files download as `Patch-A.mpq`, `Patch-B.mpq`, etc.
3. Verify button calls verifyPatches and displays results
4. Repair button calls repairPatches with progress tracking
5. All builds pass (cargo check, cargo test, npm run build)
</success_criteria>

<output>
After completion, create `.planning/phases/5-integration-fixes/5-01-SUMMARY.md`
</output>
