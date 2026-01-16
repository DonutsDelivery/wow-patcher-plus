# Phase 4: GUI & Configuration - Research

**Researched:** 2026-01-16
**Domain:** Tauri v2 + React + Desktop UI
**Confidence:** HIGH

## Summary

This phase involves building a complete desktop GUI for the Turtle WoW HD Patcher using the existing Tauri v2 + React-TS stack. The research covers five key areas: React component library selection, Tauri event/channel patterns, quality preset definitions, cross-platform build configuration, and state management approach.

The standard approach is to use **shadcn/ui with Tailwind CSS** for the component library (provides native dark theme support, accessibility, and lightweight footprint), **Tauri Channels** for streaming download/install progress events, **React useState + LazyStore** for state management, and **GitHub Actions matrix strategy** for cross-platform builds.

**Primary recommendation:** Use shadcn/ui components with Tailwind CSS dark theme, communicate via Tauri Channels for progress streaming, and leverage the existing tauri-plugin-store for persistence.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| shadcn/ui | latest | UI components | Copy-paste components, full dark theme, accessibility built-in |
| Tailwind CSS | 4.x | Styling | First-class shadcn/ui integration, dark mode via CSS variables |
| @tauri-apps/api | 2.x | Tauri IPC | Already in use, provides Channel and invoke APIs |
| @tauri-apps/plugin-store | 2.x | Settings persistence | Already configured in backend, provides LazyStore for React |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tw-animate-css | latest | Animations | Replaced tailwindcss-animate in March 2025 |
| lucide-react | latest | Icons | Recommended by shadcn/ui, tree-shakeable |
| @types/node | latest | TypeScript | Required for Vite path resolution |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| shadcn/ui | Radix primitives directly | More control, more work |
| shadcn/ui | Material UI | Heavier, harder to customize |
| useState | Zustand | Overkill for this app's complexity |
| Manual types | tauri-typegen | Nice for larger apps, adds build complexity |

**Installation:**
```bash
# Tailwind CSS for Vite
npm install tailwindcss @tailwindcss/vite

# Initialize shadcn/ui
npx shadcn@latest init

# Add required components
npx shadcn@latest add button card checkbox switch progress select tabs scroll-area

# Icons
npm install lucide-react

# Tauri store plugin (frontend)
npm install @tauri-apps/plugin-store
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── components/
│   ├── ui/                 # shadcn/ui components (auto-generated)
│   ├── PatchCard.tsx       # Individual patch module card
│   ├── PresetSelector.tsx  # Quality preset dropdown
│   ├── DownloadProgress.tsx # Progress bar with status
│   ├── FolderPicker.tsx    # WoW folder selection
│   └── ModuleList.tsx      # Scrollable list of all patches
├── hooks/
│   ├── useTauriChannel.ts  # Generic channel listener hook
│   ├── usePatches.ts       # Patch fetching and state
│   ├── useSettings.ts      # Settings persistence hook
│   └── useWowPath.ts       # WoW folder detection/selection
├── lib/
│   ├── utils.ts            # shadcn/ui utility (cn function)
│   ├── presets.ts          # Quality preset definitions
│   └── tauri.ts            # Typed invoke wrappers
├── App.tsx                 # Main application component
├── App.css                 # Global styles (Tailwind imports)
└── main.tsx                # React entry point
```

### Pattern 1: Tauri Channel in React

**What:** Use Tauri's Channel API for streaming progress events from Rust to React.
**When to use:** Any long-running operation (downloads, installs) that needs progress updates.
**Example:**
```typescript
// Source: https://v2.tauri.app/develop/calling-frontend/
import { invoke, Channel } from '@tauri-apps/api/core';

type DownloadEvent =
  | { event: 'started'; data: { downloadId: string; fileName: string; totalBytes: number } }
  | { event: 'progress'; data: { downloadId: string; downloadedBytes: number; totalBytes: number; speedBps: number; percent: number } }
  | { event: 'completed'; data: { downloadId: string; filePath: string } }
  | { event: 'failed'; data: { downloadId: string; error: string } };

async function startDownload(shareUrl: string, provider: string, destDir: string): Promise<string> {
  const onProgress = new Channel<DownloadEvent>();

  onProgress.onmessage = (message) => {
    switch (message.event) {
      case 'progress':
        // Update UI state with message.data
        break;
      case 'completed':
        // Handle completion
        break;
      case 'failed':
        // Handle error
        break;
    }
  };

  return await invoke('start_download', {
    shareUrl,
    provider,
    destDir,
    onProgress,
  });
}
```

### Pattern 2: React Hook for Channel Events

**What:** Encapsulate Channel lifecycle in a reusable hook.
**When to use:** When multiple components need to listen to the same event patterns.
**Example:**
```typescript
// Source: https://github.com/tauri-apps/tauri/discussions/5194
import { useEffect, useCallback, useRef, useState } from 'react';
import { Channel } from '@tauri-apps/api/core';

function useDownloadProgress() {
  const [downloads, setDownloads] = useState<Map<string, DownloadState>>(new Map());
  const channelRef = useRef<Channel<DownloadEvent> | null>(null);

  const createChannel = useCallback(() => {
    const channel = new Channel<DownloadEvent>();

    channel.onmessage = (message) => {
      setDownloads(prev => {
        const next = new Map(prev);
        // Update state based on message.event
        return next;
      });
    };

    channelRef.current = channel;
    return channel;
  }, []);

  return { downloads, createChannel };
}
```

### Pattern 3: Settings Persistence with LazyStore

**What:** Use tauri-plugin-store's LazyStore for deferred loading of settings.
**When to use:** App-wide settings that persist between sessions.
**Example:**
```typescript
// Source: https://v2.tauri.app/plugin/store/
import { LazyStore } from '@tauri-apps/plugin-store';

const settingsStore = new LazyStore('settings.json');

// In a React hook
async function useSettings() {
  const [wowPath, setWowPath] = useState<string | null>(null);
  const [selectedPreset, setSelectedPreset] = useState<string>('medium');

  useEffect(() => {
    async function loadSettings() {
      const path = await settingsStore.get<string>('wowPath');
      const preset = await settingsStore.get<string>('preset');
      if (path) setWowPath(path);
      if (preset) setSelectedPreset(preset);
    }
    loadSettings();
  }, []);

  const saveWowPath = async (path: string) => {
    await settingsStore.set('wowPath', path);
    setWowPath(path);
  };

  return { wowPath, saveWowPath, selectedPreset, setSelectedPreset };
}
```

### Anti-Patterns to Avoid

- **Don't use Tauri Events for high-frequency data:** Events are slower and unordered compared to Channels. Use Channels for progress streaming.
- **Don't forget to handle Channel cleanup in React:** While Channels auto-cleanup on page reload, SPA navigation requires manual cleanup.
- **Don't mix component state with global state:** Keep download progress in component state, only persist user preferences to store.
- **Don't block the UI during large operations:** Always use async patterns and show loading states.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Progress bar component | Custom div-based progress | shadcn/ui Progress | Accessibility, animations, styling |
| Toggle/checkbox | Custom input styling | shadcn/ui Switch/Checkbox | Accessibility, consistent design |
| Dark theme toggle | Manual CSS class switching | Tailwind dark: variant + CSS vars | shadcn/ui has built-in support |
| Modal dialogs | Custom overlay div | shadcn/ui Dialog | Focus trapping, escape handling |
| Settings persistence | localStorage wrapper | tauri-plugin-store | Cross-platform, file-based, already set up |
| Type-safe IPC | Manual type definitions | Define once, use TypeScript | Types already exist in models.rs |
| Cross-platform builds | Manual CI scripts | tauri-action GitHub Action | Handles all platforms, signing, artifacts |

**Key insight:** shadcn/ui components are copy-pasted into your codebase, so you own them completely. This means you get accessible, well-tested components with full customization ability.

## Common Pitfalls

### Pitfall 1: React Strict Mode Double Mounting
**What goes wrong:** Event listeners or Channels get registered twice in development.
**Why it happens:** React 18+ Strict Mode intentionally mounts components twice to detect side effects.
**How to avoid:** Always clean up in useEffect return function; use refs to track initialization.
**Warning signs:** Duplicate events, state updating twice, console logs appearing twice.

### Pitfall 2: Channel Not Ready Before Invoke
**What goes wrong:** Progress events are missed because the channel wasn't set up before calling invoke.
**Why it happens:** Channel creation is synchronous, but if you don't set onmessage before invoke returns, early events are lost.
**How to avoid:** Always set up `channel.onmessage` handler before passing channel to invoke.
**Warning signs:** Missing "started" events, progress jumps from 0% to a higher value.

### Pitfall 3: Tailwind CSS Purging Component Classes
**What goes wrong:** Dynamically constructed class names don't appear in production builds.
**Why it happens:** Tailwind purges classes it doesn't see as complete strings in source code.
**How to avoid:** Use complete class names or safelist patterns in config.
**Warning signs:** Styles work in dev but break in production.

### Pitfall 4: Dialog/Modal Focus Issues in Tauri
**What goes wrong:** Dialogs lose focus or keyboard navigation breaks.
**Why it happens:** Tauri's webview can have focus quirks compared to browsers.
**How to avoid:** Use shadcn/ui Dialog (Radix-based) which handles focus trapping properly.
**Warning signs:** Tab key doesn't cycle through dialog buttons, escape doesn't close.

### Pitfall 5: Cross-Platform Build Failures
**What goes wrong:** Build succeeds on one platform but fails on others.
**Why it happens:** Platform-specific dependencies, case sensitivity, path separators.
**How to avoid:** Always test in CI matrix across all platforms before release.
**Warning signs:** Works on dev machine, fails in GitHub Actions on another OS.

## Code Examples

Verified patterns from official sources:

### Dark Theme Setup (CSS)
```css
/* Source: https://ui.shadcn.com/docs/theming */
@import "tailwindcss";
@import "tw-animate-css";

@custom-variant dark (&:is(.dark *));

:root {
  --background: oklch(1 0 0);
  --foreground: oklch(0.145 0 0);
  --card: oklch(1 0 0);
  --card-foreground: oklch(0.145 0 0);
  --primary: oklch(0.205 0 0);
  --primary-foreground: oklch(0.985 0 0);
  --secondary: oklch(0.97 0 0);
  --secondary-foreground: oklch(0.205 0 0);
  --muted: oklch(0.97 0 0);
  --muted-foreground: oklch(0.556 0 0);
  --accent: oklch(0.97 0 0);
  --accent-foreground: oklch(0.205 0 0);
  --destructive: oklch(0.577 0.245 27.325);
  --border: oklch(0.922 0 0);
  --input: oklch(0.922 0 0);
  --ring: oklch(0.708 0 0);
}

.dark {
  --background: oklch(0.145 0 0);
  --foreground: oklch(0.985 0 0);
  --card: oklch(0.205 0 0);
  --card-foreground: oklch(0.985 0 0);
  --primary: oklch(0.922 0 0);
  --primary-foreground: oklch(0.205 0 0);
  --secondary: oklch(0.269 0 0);
  --secondary-foreground: oklch(0.985 0 0);
  --muted: oklch(0.269 0 0);
  --muted-foreground: oklch(0.708 0 0);
  --accent: oklch(0.269 0 0);
  --accent-foreground: oklch(0.985 0 0);
  --destructive: oklch(0.704 0.191 22.216);
  --border: oklch(0.269 0 0);
  --input: oklch(0.269 0 0);
  --ring: oklch(0.556 0 0);
}
```

### Force Dark Mode
```typescript
// In main.tsx or App.tsx
// Source: https://ui.shadcn.com/docs/theming
useEffect(() => {
  document.documentElement.classList.add('dark');
}, []);
```

### shadcn/ui Progress Component Usage
```typescript
// Source: https://ui.shadcn.com/docs/components/progress
import { Progress } from "@/components/ui/progress";

function DownloadProgress({ percent }: { percent: number }) {
  return (
    <div className="space-y-2">
      <Progress value={percent} className="w-full" />
      <p className="text-sm text-muted-foreground">{percent.toFixed(1)}%</p>
    </div>
  );
}
```

### LazyStore for Settings
```typescript
// Source: https://v2.tauri.app/plugin/store/
import { LazyStore } from '@tauri-apps/plugin-store';

const store = new LazyStore('settings.json');

// Save
await store.set('selectedModules', ['A', 'C', 'G', 'M']);
await store.set('preset', 'high');

// Load with type safety
const modules = await store.get<string[]>('selectedModules');
const preset = await store.get<string>('preset');
```

## Quality Presets Definition

Based on the 14 available modules and their purposes, here are recommended presets:

### Preset Definitions
```typescript
// Source: Analysis of patch modules from parser/modules.rs
export const PRESETS = {
  low: {
    name: 'Low',
    description: 'Minimal HD patches for low-end systems',
    modules: ['I', 'M'],  // Interface + Maps/Loading Screens
  },
  medium: {
    name: 'Medium',
    description: 'Core visual improvements',
    modules: ['A', 'C', 'G', 'I', 'M', 'V'],  // Characters, Creatures, Gear, Interface, Maps, VFX
  },
  high: {
    name: 'High',
    description: 'Comprehensive HD overhaul',
    modules: ['A', 'B', 'C', 'D', 'E', 'G', 'I', 'M', 'S', 'V'],  // All core + Buildings/Doodads/Environment + Sound
  },
  ultra: {
    name: 'Ultra',
    description: 'Maximum quality with 4K textures',
    modules: ['A', 'B', 'C', 'D', 'E', 'G', 'I', 'M', 'S', 'U', 'V'],  // High + Ultra HD characters
  },
} as const;

// Optional modules (user can toggle independently)
export const OPTIONAL_MODULES = {
  L: 'A Little Extra for Females',  // Requires A
  N: 'Darker Nights',
  O: 'Raid Visuals Mod',  // Requires S
} as const;
```

### Preset Selection Logic
```typescript
function applyPreset(presetKey: keyof typeof PRESETS) {
  const preset = PRESETS[presetKey];
  const withDeps = autoSelectDependencies(preset.modules);
  return withDeps;
}
```

## Cross-Platform Build Configuration

### GitHub Actions Workflow
```yaml
# Source: https://v2.tauri.app/distribute/pipelines/github/
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

### tauri.conf.json Bundle Configuration
```json
{
  "bundle": {
    "active": true,
    "targets": ["nsis", "deb", "rpm", "app", "dmg"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "identifier": "com.turtlewow.hdpatcher",
    "shortDescription": "Automated HD Patch installer for Turtle WoW",
    "longDescription": "Download and install HD texture packs for Turtle WoW with automatic dependency resolution."
  }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| tailwindcss-animate | tw-animate-css | March 2025 | New shadcn/ui projects use tw-animate-css by default |
| HSL colors | OKLCH colors | 2025 | Better perceptual uniformity, shadcn/ui migrated |
| forwardRef in components | Direct props | React 19 | shadcn/ui removed forwardRefs for React 19 |
| Tauri Events for streaming | Tauri Channels | Tauri v2 | Channels are faster and maintain order |
| next-themes | Tailwind dark: + class | 2025 | For non-Next.js apps, just add `dark` class to html |

**Deprecated/outdated:**
- `tailwindcss-animate`: Replaced by `tw-animate-css` for shadcn/ui projects
- Tauri v1 event patterns: v2 uses different API (listen from `@tauri-apps/api/event`)
- HSL color format: OKLCH is now the default in shadcn/ui

## Open Questions

Things that couldn't be fully resolved:

1. **Exact dependency sizes for presets**
   - What we know: Module sizes vary significantly (U is likely largest due to 4K textures)
   - What's unclear: Exact download sizes per module aren't in the codebase
   - Recommendation: Display estimated sizes from forum parsing if available, otherwise omit

2. **Icon assets**
   - What we know: tauri.conf.json references icon paths that need to exist
   - What's unclear: Whether icon assets currently exist in the project
   - Recommendation: Create or source appropriate icons before release builds

3. **Error handling UX**
   - What we know: Rust backend emits Failed events with error strings
   - What's unclear: Best UX for displaying errors (toast, dialog, inline)
   - Recommendation: Use shadcn/ui Alert or Toast component for errors

## Sources

### Primary (HIGH confidence)
- [Tauri v2 Calling Frontend docs](https://v2.tauri.app/develop/calling-frontend/) - Channel API patterns
- [Tauri v2 Event API](https://v2.tauri.app/reference/javascript/api/namespaceevent/) - Event types and functions
- [Tauri v2 GitHub Actions](https://v2.tauri.app/distribute/pipelines/github/) - Cross-platform build workflow
- [Tauri v2 Store Plugin](https://v2.tauri.app/plugin/store/) - Settings persistence API
- [shadcn/ui Vite installation](https://ui.shadcn.com/docs/installation/vite) - Installation steps
- [shadcn/ui Theming](https://ui.shadcn.com/docs/theming) - Dark mode CSS variables
- [shadcn/ui Progress](https://ui.shadcn.com/docs/components/progress) - Progress component

### Secondary (MEDIUM confidence)
- [Tauri events discussion](https://github.com/tauri-apps/tauri/discussions/5194) - React cleanup patterns
- [shadcn/ui Tailwind v4](https://ui.shadcn.com/docs/tailwind-v4) - Migration notes

### Tertiary (LOW confidence)
- [Game mod installer UI patterns](https://www.artstation.com/artwork/WKP3lv) - Visual reference only
- Quality preset definitions derived from module analysis in codebase

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Official documentation for all tools
- Architecture: HIGH - Patterns from official Tauri and React documentation
- Pitfalls: MEDIUM - Community discussions and documented issues
- Cross-platform builds: HIGH - Official Tauri GitHub Actions workflow
- Presets: MEDIUM - Derived from module analysis, may need adjustment

**Research date:** 2026-01-16
**Valid until:** 2026-02-16 (30 days - stable ecosystem)
