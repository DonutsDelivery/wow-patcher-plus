---
phase: 4-gui-configuration
plan: 03
type: execute
wave: 3
depends_on: ["4-02"]
files_modified:
  - src/lib/tauri.ts
  - src/components/DownloadProgress.tsx
  - src/components/InstallProgress.tsx
  - src/hooks/useDownload.ts
  - src/hooks/useInstall.ts
  - src/App.tsx
autonomous: true

must_haves:
  truths:
    - "User sees download progress with percentage and speed"
    - "User sees install progress with current file"
    - "User can start download and installation workflow"
    - "App handles multiple concurrent downloads"
    - "Progress events stream from Rust to React via Channels"
  artifacts:
    - path: "src/components/DownloadProgress.tsx"
      provides: "Download progress display component"
      exports: ["DownloadProgress"]
    - path: "src/components/InstallProgress.tsx"
      provides: "Installation progress display component"
      exports: ["InstallProgress"]
    - path: "src/hooks/useDownload.ts"
      provides: "Download state and Channel management"
      exports: ["useDownload"]
    - path: "src/hooks/useInstall.ts"
      provides: "Installation state and Channel management"
      exports: ["useInstall"]
    - path: "src/App.tsx"
      provides: "Main application with full workflow"
      min_lines: 100
  key_links:
    - from: "src/hooks/useDownload.ts"
      to: "start_download command"
      via: "Tauri Channel"
      pattern: "Channel.*onmessage"
    - from: "src/hooks/useInstall.ts"
      to: "install_patches command"
      via: "Tauri Channel"
      pattern: "Channel.*onmessage"
    - from: "src/App.tsx"
      to: "All components"
      via: "Component composition"
      pattern: "PresetSelector|ModuleList|FolderPicker|DownloadProgress"
---

<objective>
Build download/install progress components and integrate all pieces into the main App.

Purpose: Create the complete user workflow from configuration to installation.
Output: Functional app with progress tracking for downloads and installations.
</objective>

<execution_context>
@./.claude/get-shit-done/workflows/execute-plan.md
@./.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/phases/4-gui-configuration/4-RESEARCH.md
@.planning/phases/4-gui-configuration/4-02-SUMMARY.md
@src-tauri/src/download/progress.rs
@src-tauri/src/install/copier.rs
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add download and install invoke wrappers with Channel types</name>
  <files>src/lib/tauri.ts</files>
  <action>
Extend src/lib/tauri.ts with download and install command wrappers:

Add TypeScript types matching Rust DownloadEvent and InstallEvent:
```typescript
import { invoke, Channel } from '@tauri-apps/api/core';

// Download events (matches DownloadEvent in progress.rs)
export type DownloadEvent =
  | { event: 'started'; data: { downloadId: string; fileName: string; totalBytes: number } }
  | { event: 'progress'; data: { downloadId: string; downloadedBytes: number; totalBytes: number; speedBps: number; percent: number } }
  | { event: 'completed'; data: { downloadId: string; filePath: string } }
  | { event: 'failed'; data: { downloadId: string; error: string } };

// Install events (matches InstallEvent in copier.rs)
export type InstallEvent =
  | { event: 'started'; data: { patchId: string; fileName: string; totalBytes: number } }
  | { event: 'progress'; data: { patchId: string; copiedBytes: number; totalBytes: number; percent: number } }
  | { event: 'completed'; data: { patchId: string; destPath: string } }
  | { event: 'failed'; data: { patchId: string; error: string } };

// Download commands
export async function startDownload(
  shareUrl: string,
  provider: string,
  destDir: string,
  onProgress: Channel<DownloadEvent>
): Promise<string> {
  return invoke('start_download', { shareUrl, provider, destDir, onProgress });
}

export async function getActiveDownloads(): Promise<number> {
  return invoke('get_active_downloads');
}

// Install commands
export async function installPatches(
  patchIds: string[],
  onEvent: Channel<InstallEvent>
): Promise<string[]> {
  return invoke('install_patches', { patchIds, onEvent });
}

export async function verifyPatches(patchIds: string[]): Promise<[string, { status: string }][]> {
  return invoke('verify_patches', { patchIds });
}

export async function repairPatches(
  patchIds: string[],
  onEvent: Channel<InstallEvent>
): Promise<{ patchId: string; success: boolean; error?: string }[]> {
  return invoke('repair_patches', { patchIds, onEvent });
}
```
  </action>
  <verify>
`npm run build` succeeds with new types.
  </verify>
  <done>Download and install command wrappers with typed Channel events added.</done>
</task>

<task type="auto">
  <name>Task 2: Create useDownload and useInstall hooks with Channel management</name>
  <files>src/hooks/useDownload.ts, src/hooks/useInstall.ts</files>
  <action>
Create src/hooks/useDownload.ts:
```typescript
import { useState, useCallback } from 'react';
import { Channel } from '@tauri-apps/api/core';
import { startDownload, DownloadEvent, PatchModule } from '@/lib/tauri';
import { appDataDir, join } from '@tauri-apps/api/path';

export interface DownloadState {
  downloadId: string;
  fileName: string;
  totalBytes: number;
  downloadedBytes: number;
  speedBps: number;
  percent: number;
  status: 'pending' | 'downloading' | 'completed' | 'failed';
  error?: string;
}

export function useDownload() {
  const [downloads, setDownloads] = useState<Map<string, DownloadState>>(new Map());

  const startModuleDownload = useCallback(async (module: PatchModule) => {
    // Find first available link
    const link = module.links[0];
    if (!link) throw new Error(`No download link for module ${module.id}`);

    const destDir = await join(await appDataDir(), 'downloads');

    const onProgress = new Channel<DownloadEvent>();

    onProgress.onmessage = (msg) => {
      setDownloads(prev => {
        const next = new Map(prev);
        const current = next.get(msg.data.downloadId) || {
          downloadId: msg.data.downloadId,
          fileName: '',
          totalBytes: 0,
          downloadedBytes: 0,
          speedBps: 0,
          percent: 0,
          status: 'pending' as const,
        };

        switch (msg.event) {
          case 'started':
            next.set(msg.data.downloadId, {
              ...current,
              fileName: msg.data.fileName,
              totalBytes: msg.data.totalBytes,
              status: 'downloading',
            });
            break;
          case 'progress':
            next.set(msg.data.downloadId, {
              ...current,
              downloadedBytes: msg.data.downloadedBytes,
              totalBytes: msg.data.totalBytes,
              speedBps: msg.data.speedBps,
              percent: msg.data.percent,
              status: 'downloading',
            });
            break;
          case 'completed':
            next.set(msg.data.downloadId, {
              ...current,
              percent: 100,
              status: 'completed',
            });
            break;
          case 'failed':
            next.set(msg.data.downloadId, {
              ...current,
              status: 'failed',
              error: msg.data.error,
            });
            break;
        }
        return next;
      });
    };

    const downloadId = await startDownload(link.url, link.provider, destDir, onProgress);
    return downloadId;
  }, []);

  const downloadAll = useCallback(async (modules: PatchModule[]) => {
    const results = await Promise.allSettled(
      modules.map(m => startModuleDownload(m))
    );
    return results;
  }, [startModuleDownload]);

  return { downloads, startModuleDownload, downloadAll };
}
```

Create src/hooks/useInstall.ts:
```typescript
import { useState, useCallback } from 'react';
import { Channel } from '@tauri-apps/api/core';
import { installPatches, InstallEvent } from '@/lib/tauri';

export interface InstallState {
  patchId: string;
  fileName: string;
  totalBytes: number;
  copiedBytes: number;
  percent: number;
  status: 'pending' | 'installing' | 'completed' | 'failed';
  error?: string;
}

export function useInstall() {
  const [installs, setInstalls] = useState<Map<string, InstallState>>(new Map());
  const [installing, setInstalling] = useState(false);

  const install = useCallback(async (patchIds: string[]) => {
    setInstalling(true);

    const onEvent = new Channel<InstallEvent>();

    onEvent.onmessage = (msg) => {
      setInstalls(prev => {
        const next = new Map(prev);
        const patchId = msg.data.patchId;
        const current = next.get(patchId) || {
          patchId,
          fileName: '',
          totalBytes: 0,
          copiedBytes: 0,
          percent: 0,
          status: 'pending' as const,
        };

        switch (msg.event) {
          case 'started':
            next.set(patchId, {
              ...current,
              fileName: msg.data.fileName,
              totalBytes: msg.data.totalBytes,
              status: 'installing',
            });
            break;
          case 'progress':
            next.set(patchId, {
              ...current,
              copiedBytes: msg.data.copiedBytes,
              totalBytes: msg.data.totalBytes,
              percent: msg.data.percent,
              status: 'installing',
            });
            break;
          case 'completed':
            next.set(patchId, {
              ...current,
              percent: 100,
              status: 'completed',
            });
            break;
          case 'failed':
            next.set(patchId, {
              ...current,
              status: 'failed',
              error: msg.data.error,
            });
            break;
        }
        return next;
      });
    };

    try {
      const installed = await installPatches(patchIds, onEvent);
      return installed;
    } finally {
      setInstalling(false);
    }
  }, []);

  return { installs, installing, install };
}
```
  </action>
  <verify>
`npm run build` succeeds.
Channel API correctly typed.
  </verify>
  <done>useDownload and useInstall hooks created with Tauri Channel integration.</done>
</task>

<task type="auto">
  <name>Task 3: Create progress components and assemble main App</name>
  <files>src/components/DownloadProgress.tsx, src/components/InstallProgress.tsx, src/App.tsx</files>
  <action>
Create src/components/DownloadProgress.tsx:
```typescript
import { Progress } from '@/components/ui/progress';
import { DownloadState } from '@/hooks/useDownload';
import { Download, CheckCircle, XCircle } from 'lucide-react';

interface Props {
  downloads: Map<string, DownloadState>;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

function formatSpeed(bps: number): string {
  return formatBytes(bps) + '/s';
}

export function DownloadProgress({ downloads }: Props) {
  if (downloads.size === 0) return null;

  return (
    <div className="space-y-3">
      <h3 className="text-sm font-medium">Downloads</h3>
      {Array.from(downloads.values()).map((dl) => (
        <div key={dl.downloadId} className="space-y-1">
          <div className="flex items-center justify-between text-sm">
            <div className="flex items-center gap-2">
              {dl.status === 'completed' && <CheckCircle className="h-4 w-4 text-green-500" />}
              {dl.status === 'failed' && <XCircle className="h-4 w-4 text-red-500" />}
              {dl.status === 'downloading' && <Download className="h-4 w-4 animate-pulse" />}
              <span className="truncate max-w-[150px]">{dl.fileName || 'Starting...'}</span>
            </div>
            <span className="text-muted-foreground">
              {dl.status === 'downloading' && formatSpeed(dl.speedBps)}
              {dl.status === 'completed' && 'Done'}
              {dl.status === 'failed' && 'Failed'}
            </span>
          </div>
          <Progress value={dl.percent} className="h-2" />
          {dl.error && <p className="text-xs text-red-500">{dl.error}</p>}
        </div>
      ))}
    </div>
  );
}
```

Create src/components/InstallProgress.tsx:
```typescript
import { Progress } from '@/components/ui/progress';
import { InstallState } from '@/hooks/useInstall';
import { HardDrive, CheckCircle, XCircle } from 'lucide-react';

interface Props {
  installs: Map<string, InstallState>;
}

export function InstallProgress({ installs }: Props) {
  if (installs.size === 0) return null;

  return (
    <div className="space-y-3">
      <h3 className="text-sm font-medium">Installing</h3>
      {Array.from(installs.values()).map((inst) => (
        <div key={inst.patchId} className="space-y-1">
          <div className="flex items-center justify-between text-sm">
            <div className="flex items-center gap-2">
              {inst.status === 'completed' && <CheckCircle className="h-4 w-4 text-green-500" />}
              {inst.status === 'failed' && <XCircle className="h-4 w-4 text-red-500" />}
              {inst.status === 'installing' && <HardDrive className="h-4 w-4 animate-pulse" />}
              <span>Module {inst.patchId}</span>
            </div>
            <span className="text-muted-foreground">
              {inst.percent.toFixed(0)}%
            </span>
          </div>
          <Progress value={inst.percent} className="h-2" />
          {inst.error && <p className="text-xs text-red-500">{inst.error}</p>}
        </div>
      ))}
    </div>
  );
}
```

Update src/App.tsx with complete workflow:
```typescript
import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { PresetSelector } from '@/components/PresetSelector';
import { ModuleList } from '@/components/ModuleList';
import { FolderPicker } from '@/components/FolderPicker';
import { DownloadProgress } from '@/components/DownloadProgress';
import { InstallProgress } from '@/components/InstallProgress';
import { usePatches } from '@/hooks/usePatches';
import { useWowPath } from '@/hooks/useWowPath';
import { useDownload } from '@/hooks/useDownload';
import { useInstall } from '@/hooks/useInstall';
import { Loader2 } from 'lucide-react';
import './App.css';

type AppState = 'configure' | 'downloading' | 'installing' | 'complete';

function App() {
  const [appState, setAppState] = useState<AppState>('configure');
  const { patches, selectedModules, loading, error, applyPreset, toggleModule } = usePatches();
  const { wowPath, loading: pathLoading, pickFolder } = useWowPath();
  const { downloads, downloadAll } = useDownload();
  const { installs, installing, install } = useInstall();

  const selectedPatches = patches.filter(p => selectedModules.has(p.id));
  const canStart = selectedModules.size > 0 && wowPath !== null;

  const handleStart = async () => {
    if (!canStart) return;

    setAppState('downloading');
    await downloadAll(selectedPatches);

    setAppState('installing');
    await install(Array.from(selectedModules));

    setAppState('complete');
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center p-8">
        <Card className="max-w-md">
          <CardHeader>
            <CardTitle className="text-red-500">Error</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-muted-foreground">{error}</p>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background text-foreground p-6">
      <div className="max-w-2xl mx-auto space-y-6">
        <div className="text-center">
          <h1 className="text-3xl font-bold">Turtle WoW HD Patcher</h1>
          <p className="text-muted-foreground mt-2">
            Automated HD Patch installation for Turtle WoW
          </p>
        </div>

        <Card>
          <CardHeader>
            <CardTitle>Configuration</CardTitle>
            <CardDescription>
              Select your quality preset or customize individual modules
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            <FolderPicker path={wowPath} loading={pathLoading} onPick={pickFolder} />
            <PresetSelector onSelect={applyPreset} />
            <ModuleList modules={patches} selected={selectedModules} onToggle={toggleModule} />
          </CardContent>
        </Card>

        {downloads.size > 0 && (
          <Card>
            <CardContent className="pt-6">
              <DownloadProgress downloads={downloads} />
            </CardContent>
          </Card>
        )}

        {installs.size > 0 && (
          <Card>
            <CardContent className="pt-6">
              <InstallProgress installs={installs} />
            </CardContent>
          </Card>
        )}

        <div className="flex justify-center">
          <Button
            size="lg"
            onClick={handleStart}
            disabled={!canStart || appState !== 'configure'}
          >
            {appState === 'configure' && `Install ${selectedModules.size} Module${selectedModules.size !== 1 ? 's' : ''}`}
            {appState === 'downloading' && 'Downloading...'}
            {appState === 'installing' && 'Installing...'}
            {appState === 'complete' && 'Complete!'}
          </Button>
        </div>

        {appState === 'complete' && (
          <p className="text-center text-green-500">
            Installation complete! You can now launch Turtle WoW.
          </p>
        )}
      </div>
    </div>
  );
}

export default App;
```
  </action>
  <verify>
`npm run tauri dev` shows complete UI:
- Loading spinner while fetching patches
- Folder picker with auto-detect
- Preset selector dropdown
- Module list with checkboxes
- Install button shows selected count
- Progress cards appear during download/install
  </verify>
  <done>Progress components created. Main App assembled with complete workflow.</done>
</task>

</tasks>

<verification>
1. `npm run tauri dev` launches complete app
2. Selecting preset populates module checkboxes
3. Folder picker works (dialog opens, path shows when selected)
4. Install button enables only when modules selected AND folder chosen
5. Progress appears during download/install phases
</verification>

<success_criteria>
- User can complete full workflow: select preset -> pick folder -> install
- Download progress shows file name, speed, percentage
- Install progress shows module and percentage
- State transitions: configure -> downloading -> installing -> complete
- All UI renders with dark theme styling
</success_criteria>

<output>
After completion, create `.planning/phases/4-gui-configuration/4-03-SUMMARY.md`
</output>
