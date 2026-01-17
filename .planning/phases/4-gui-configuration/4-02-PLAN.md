---
phase: 4-gui-configuration
plan: 02
type: execute
wave: 2
depends_on: ["4-01"]
files_modified:
  - src/lib/presets.ts
  - src/lib/tauri.ts
  - src/components/PresetSelector.tsx
  - src/components/ModuleList.tsx
  - src/components/FolderPicker.tsx
  - src/hooks/usePatches.ts
  - src/hooks/useWowPath.ts
autonomous: true

must_haves:
  truths:
    - "User can select a quality preset from dropdown"
    - "User can see list of all patch modules with descriptions"
    - "User can toggle individual modules on/off"
    - "User can select WoW folder via native dialog"
    - "App auto-detects and loads saved WoW path on startup"
  artifacts:
    - path: "src/lib/presets.ts"
      provides: "Quality preset definitions"
      exports: ["PRESETS", "OPTIONAL_MODULES"]
    - path: "src/lib/tauri.ts"
      provides: "Typed invoke wrappers for Tauri commands"
      exports: ["fetchPatches", "selectWowFolder", "autoDetectWow", "loadSavedWowPath"]
    - path: "src/components/PresetSelector.tsx"
      provides: "Quality preset dropdown component"
      exports: ["PresetSelector"]
    - path: "src/components/ModuleList.tsx"
      provides: "Module toggle list component"
      exports: ["ModuleList"]
    - path: "src/components/FolderPicker.tsx"
      provides: "WoW folder selection component"
      exports: ["FolderPicker"]
    - path: "src/hooks/usePatches.ts"
      provides: "Patch fetching and selection state"
      exports: ["usePatches"]
    - path: "src/hooks/useWowPath.ts"
      provides: "WoW folder detection and persistence"
      exports: ["useWowPath"]
  key_links:
    - from: "src/lib/tauri.ts"
      to: "Rust commands"
      via: "invoke calls"
      pattern: "invoke.*fetch_patches|select_wow_folder|auto_detect_wow"
    - from: "src/components/PresetSelector.tsx"
      to: "src/lib/presets.ts"
      via: "PRESETS import"
      pattern: "import.*PRESETS"
    - from: "src/hooks/useWowPath.ts"
      to: "src/lib/tauri.ts"
      via: "loadSavedWowPath call"
      pattern: "loadSavedWowPath|autoDetectWow"
---

<objective>
Build core UI components: PresetSelector, ModuleList, and FolderPicker with supporting hooks.

Purpose: Enable users to configure their patch selection and WoW installation path.
Output: Working components for preset selection, module toggling, and folder picking.
</objective>

<execution_context>
@./.claude/get-shit-done/workflows/execute-plan.md
@./.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/phases/4-gui-configuration/4-RESEARCH.md
@.planning/phases/4-gui-configuration/4-01-SUMMARY.md
@src-tauri/src/lib.rs
</context>

<tasks>

<task type="auto">
  <name>Task 1: Create presets and typed Tauri wrappers</name>
  <files>src/lib/presets.ts, src/lib/tauri.ts</files>
  <action>
Create src/lib/presets.ts with quality preset definitions from 4-RESEARCH.md:
```typescript
export const PRESETS = {
  low: {
    name: 'Low',
    description: 'Minimal HD patches for low-end systems',
    modules: ['I', 'M'],
  },
  medium: {
    name: 'Medium',
    description: 'Core visual improvements',
    modules: ['A', 'C', 'G', 'I', 'M', 'V'],
  },
  high: {
    name: 'High',
    description: 'Comprehensive HD overhaul',
    modules: ['A', 'B', 'C', 'D', 'E', 'G', 'I', 'M', 'S', 'V'],
  },
  ultra: {
    name: 'Ultra',
    description: 'Maximum quality with 4K textures',
    modules: ['A', 'B', 'C', 'D', 'E', 'G', 'I', 'M', 'S', 'U', 'V'],
  },
} as const;

export const OPTIONAL_MODULES = {
  L: 'A Little Extra for Females',
  N: 'Darker Nights',
  O: 'Raid Visuals Mod',
} as const;

export type PresetKey = keyof typeof PRESETS;
```

Create src/lib/tauri.ts with typed wrappers for backend commands:
```typescript
import { invoke } from '@tauri-apps/api/core';

// Types matching Rust models
export interface PatchModule {
  id: string;
  name: string;
  description: string;
  links: { provider: string; url: string }[];
  dependencies: string[];
}

// Parser commands
export async function fetchPatches(): Promise<PatchModule[]> {
  return invoke('fetch_patches');
}

export async function validateSelection(selected: string[]): Promise<void> {
  return invoke('validate_selection', { selected });
}

export async function autoSelectDeps(selected: string[]): Promise<string[]> {
  return invoke('auto_select_deps', { selected });
}

// Install commands
export async function selectWowFolder(): Promise<string | null> {
  return invoke('select_wow_folder');
}

export async function getWowPath(): Promise<string | null> {
  return invoke('get_wow_path');
}

export async function autoDetectWow(): Promise<string | null> {
  return invoke('auto_detect_wow');
}

export async function loadSavedWowPath(): Promise<string | null> {
  return invoke('load_saved_wow_path');
}
```
  </action>
  <verify>
Files exist and TypeScript compiles: `npm run build` succeeds.
  </verify>
  <done>Preset definitions and typed Tauri wrappers created.</done>
</task>

<task type="auto">
  <name>Task 2: Create usePatches and useWowPath hooks</name>
  <files>src/hooks/usePatches.ts, src/hooks/useWowPath.ts</files>
  <action>
Create src/hooks/usePatches.ts:
```typescript
import { useState, useEffect, useCallback } from 'react';
import { fetchPatches, autoSelectDeps, PatchModule } from '@/lib/tauri';
import { PRESETS, PresetKey } from '@/lib/presets';

export function usePatches() {
  const [patches, setPatches] = useState<PatchModule[]>([]);
  const [selectedModules, setSelectedModules] = useState<Set<string>>(new Set());
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchPatches()
      .then(setPatches)
      .catch(e => setError(e.toString()))
      .finally(() => setLoading(false));
  }, []);

  const applyPreset = useCallback(async (preset: PresetKey) => {
    const modules = PRESETS[preset].modules;
    const withDeps = await autoSelectDeps([...modules]);
    setSelectedModules(new Set(withDeps));
  }, []);

  const toggleModule = useCallback(async (moduleId: string) => {
    setSelectedModules(prev => {
      const next = new Set(prev);
      if (next.has(moduleId)) {
        next.delete(moduleId);
      } else {
        next.add(moduleId);
      }
      return next;
    });
  }, []);

  return { patches, selectedModules, loading, error, applyPreset, toggleModule, setSelectedModules };
}
```

Create src/hooks/useWowPath.ts:
```typescript
import { useState, useEffect, useCallback } from 'react';
import { loadSavedWowPath, autoDetectWow, selectWowFolder } from '@/lib/tauri';

export function useWowPath() {
  const [wowPath, setWowPath] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function init() {
      // Try loading saved path first
      let path = await loadSavedWowPath();

      // If no saved path, try auto-detect
      if (!path) {
        path = await autoDetectWow();
      }

      setWowPath(path);
      setLoading(false);
    }
    init();
  }, []);

  const pickFolder = useCallback(async () => {
    const path = await selectWowFolder();
    if (path) {
      setWowPath(path);
    }
    return path;
  }, []);

  return { wowPath, loading, pickFolder };
}
```
  </action>
  <verify>
TypeScript compiles: `npm run build` succeeds.
Hooks export correctly when imported.
  </verify>
  <done>usePatches and useWowPath hooks created with state management and Tauri integration.</done>
</task>

<task type="auto">
  <name>Task 3: Create PresetSelector, ModuleList, and FolderPicker components</name>
  <files>src/components/PresetSelector.tsx, src/components/ModuleList.tsx, src/components/FolderPicker.tsx</files>
  <action>
Create src/components/PresetSelector.tsx:
```typescript
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { PRESETS, PresetKey } from '@/lib/presets';

interface Props {
  onSelect: (preset: PresetKey) => void;
}

export function PresetSelector({ onSelect }: Props) {
  return (
    <div className="space-y-2">
      <label className="text-sm font-medium">Quality Preset</label>
      <Select onValueChange={(v) => onSelect(v as PresetKey)}>
        <SelectTrigger className="w-full">
          <SelectValue placeholder="Select a preset..." />
        </SelectTrigger>
        <SelectContent>
          {(Object.entries(PRESETS) as [PresetKey, typeof PRESETS[PresetKey]][]).map(([key, preset]) => (
            <SelectItem key={key} value={key}>
              <div className="flex flex-col">
                <span>{preset.name}</span>
                <span className="text-xs text-muted-foreground">{preset.description}</span>
              </div>
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
```

Create src/components/ModuleList.tsx:
```typescript
import { ScrollArea } from '@/components/ui/scroll-area';
import { Checkbox } from '@/components/ui/checkbox';
import { PatchModule } from '@/lib/tauri';

interface Props {
  modules: PatchModule[];
  selected: Set<string>;
  onToggle: (moduleId: string) => void;
}

export function ModuleList({ modules, selected, onToggle }: Props) {
  return (
    <ScrollArea className="h-[300px] rounded-md border p-4">
      <div className="space-y-4">
        {modules.map((mod) => (
          <div key={mod.id} className="flex items-start space-x-3">
            <Checkbox
              id={mod.id}
              checked={selected.has(mod.id)}
              onCheckedChange={() => onToggle(mod.id)}
            />
            <div className="grid gap-1.5 leading-none">
              <label htmlFor={mod.id} className="text-sm font-medium cursor-pointer">
                {mod.id}: {mod.name}
              </label>
              <p className="text-xs text-muted-foreground">{mod.description}</p>
              {mod.dependencies.length > 0 && (
                <p className="text-xs text-muted-foreground">
                  Requires: {mod.dependencies.join(', ')}
                </p>
              )}
            </div>
          </div>
        ))}
      </div>
    </ScrollArea>
  );
}
```

Create src/components/FolderPicker.tsx:
```typescript
import { Button } from '@/components/ui/button';
import { Folder, Check, AlertCircle } from 'lucide-react';

interface Props {
  path: string | null;
  loading: boolean;
  onPick: () => void;
}

export function FolderPicker({ path, loading, onPick }: Props) {
  return (
    <div className="space-y-2">
      <label className="text-sm font-medium">WoW Installation</label>
      <div className="flex items-center gap-2">
        <Button variant="outline" onClick={onPick} disabled={loading}>
          <Folder className="mr-2 h-4 w-4" />
          {loading ? 'Detecting...' : 'Select Folder'}
        </Button>
        {path && (
          <div className="flex items-center text-sm text-muted-foreground">
            <Check className="mr-1 h-4 w-4 text-green-500" />
            <span className="truncate max-w-[200px]">{path}</span>
          </div>
        )}
        {!path && !loading && (
          <div className="flex items-center text-sm text-muted-foreground">
            <AlertCircle className="mr-1 h-4 w-4 text-yellow-500" />
            <span>No folder selected</span>
          </div>
        )}
      </div>
    </div>
  );
}
```
  </action>
  <verify>
`npm run build` succeeds.
Components render without errors when imported into App.tsx.
  </verify>
  <done>PresetSelector, ModuleList, and FolderPicker components created with proper styling and Tauri integration.</done>
</task>

</tasks>

<verification>
1. `npm run build` completes without errors
2. All components export correctly from their files
3. TypeScript types are properly defined (no any types)
4. Components use shadcn/ui primitives correctly
</verification>

<success_criteria>
- PresetSelector shows dropdown with Low/Medium/High/Ultra presets
- ModuleList displays all modules with checkboxes
- FolderPicker calls native dialog and shows selected path
- useWowPath loads saved path or auto-detects on mount
- usePatches fetches modules and manages selection state
</success_criteria>

<output>
After completion, create `.planning/phases/4-gui-configuration/4-02-SUMMARY.md`
</output>
