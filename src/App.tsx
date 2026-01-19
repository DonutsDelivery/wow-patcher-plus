import { useState, useEffect, useCallback } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { PresetSelector } from '@/components/PresetSelector';
import { ModuleList } from '@/components/ModuleList';
import { FolderPicker } from '@/components/FolderPicker';
import { RequirementsPanel } from '@/components/RequirementsPanel';
import { DownloadProgress } from '@/components/DownloadProgress';
import { InstallProgress } from '@/components/InstallProgress';
import { UpdateBanner } from '@/components/UpdateBanner';
import { usePatches } from '@/hooks/usePatches';
import { useWowPath } from '@/hooks/useWowPath';
import { useDownload } from '@/hooks/useDownload';
import { useInstall } from '@/hooks/useInstall';
import { detectInstalledPatches, uninstallPatches } from '@/lib/tauri';
import { Loader2, Plus, Minus, RefreshCw } from 'lucide-react';
import './App.css';

type AppState = 'configure' | 'downloading' | 'installing' | 'uninstalling' | 'complete';

function App() {
  const [appState, setAppState] = useState<AppState>('configure');
  const { patches, groups, selectedModules, loading, error, applyPreset, toggleModule, setSelectedModules } = usePatches();
  const { wowPath, loading: pathLoading, pickFolder } = useWowPath();
  const { downloads, downloadAll } = useDownload();
  const { installs, install, setInstalls } = useInstall();
  const [variantSelections, setVariantSelections] = useState<Map<string, number>>(new Map());
  const [installedPatches, setInstalledPatches] = useState<Set<string>>(new Set());
  const [detecting, setDetecting] = useState(false);
  const [applying, setApplying] = useState(false);

  // Calculate what needs to change
  const toInstall = Array.from(selectedModules).filter(id => !installedPatches.has(id));
  const toUninstall = Array.from(installedPatches).filter(id => !selectedModules.has(id));
  const alreadyInstalled = Array.from(selectedModules).filter(id => installedPatches.has(id));

  const hasChanges = toInstall.length > 0 || toUninstall.length > 0;
  const canApply = wowPath !== null && (hasChanges || selectedModules.size > 0);

  // Detect installed patches when wowPath or patches change
  const detectInstalled = useCallback(async () => {
    if (!wowPath || patches.length === 0) {
      setInstalledPatches(new Set());
      return;
    }

    setDetecting(true);
    try {
      const patchIds = patches.map(p => p.id);
      const installed = await detectInstalledPatches(patchIds);
      const installedSet = new Set(installed);
      setInstalledPatches(installedSet);

      // Auto-select installed mods
      if (installed.length > 0 && selectedModules.size === 0) {
        setSelectedModules(installedSet);
      }
    } catch (err) {
      console.error('Failed to detect installed patches:', err);
      setInstalledPatches(new Set());
    } finally {
      setDetecting(false);
    }
  }, [wowPath, patches, selectedModules.size, setSelectedModules]);

  // Run detection when wowPath or patches change
  useEffect(() => {
    detectInstalled();
  }, [wowPath, patches.length]); // Only re-run when path or patches list changes, not on every detectInstalled change

  // Custom folder picker that triggers detection after selection
  const handlePickFolder = async () => {
    const path = await pickFolder();
    return path;
  };

  // Unified Apply function - handles install, uninstall, and update
  const handleApply = async () => {
    if (!wowPath) return;

    setApplying(true);
    setInstalls(new Map());

    try {
      // Step 1: Uninstall mods that were deselected
      if (toUninstall.length > 0) {
        console.log('[App] Uninstalling:', toUninstall);
        setAppState('uninstalling');
        await uninstallPatches(toUninstall);
      }

      // Step 2: Download new mods
      if (toInstall.length > 0) {
        console.log('[App] Downloading:', toInstall);
        setAppState('downloading');
        const patchesToDownload = patches.filter(p => toInstall.includes(p.id));
        await downloadAll(patchesToDownload, variantSelections);

        // Step 3: Install new mods
        console.log('[App] Installing:', toInstall);
        setAppState('installing');
        await install(toInstall);
      }

      setAppState('complete');

      // Refresh detection
      await detectInstalled();
    } catch (err) {
      console.error('[App] Apply failed:', err);
    } finally {
      setApplying(false);
    }
  };

  // Get button text based on what will happen
  const getApplyButtonText = () => {
    if (applying) {
      if (appState === 'uninstalling') return 'Removing...';
      if (appState === 'downloading') return 'Downloading...';
      if (appState === 'installing') return 'Installing...';
      return 'Applying...';
    }

    if (!hasChanges) {
      if (installedPatches.size > 0 && selectedModules.size === installedPatches.size) {
        return 'Up to date';
      }
      return 'Select mods to install';
    }

    const parts: string[] = [];
    if (toInstall.length > 0) parts.push(`Install ${toInstall.length}`);
    if (toUninstall.length > 0) parts.push(`Remove ${toUninstall.length}`);
    return parts.join(', ');
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
        <UpdateBanner />

        <div className="text-center">
          <h1 className="text-3xl font-bold">WoW HD Patcher</h1>
          <p className="text-muted-foreground mt-2">
            Automated HD Patch installation
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
            <FolderPicker path={wowPath} loading={pathLoading} onPick={handlePickFolder} />
            <RequirementsPanel wowPath={wowPath} />

            {/* Status summary */}
            {wowPath && !detecting && (
              <div className="rounded-lg border bg-muted/50 p-3 text-sm space-y-1">
                {installedPatches.size > 0 && (
                  <div className="text-muted-foreground">
                    {installedPatches.size} mod{installedPatches.size !== 1 ? 's' : ''} currently installed
                  </div>
                )}
                {hasChanges && (
                  <div className="flex flex-wrap gap-3 pt-1">
                    {toInstall.length > 0 && (
                      <span className="flex items-center gap-1 text-green-600">
                        <Plus className="h-3 w-3" />
                        {toInstall.length} to install
                      </span>
                    )}
                    {toUninstall.length > 0 && (
                      <span className="flex items-center gap-1 text-red-500">
                        <Minus className="h-3 w-3" />
                        {toUninstall.length} to remove
                      </span>
                    )}
                    {alreadyInstalled.length > 0 && (
                      <span className="flex items-center gap-1 text-muted-foreground">
                        <RefreshCw className="h-3 w-3" />
                        {alreadyInstalled.length} unchanged
                      </span>
                    )}
                  </div>
                )}
                {!hasChanges && installedPatches.size > 0 && selectedModules.size > 0 && (
                  <div className="text-green-600">All selected mods are installed</div>
                )}
              </div>
            )}

            <PresetSelector onSelect={applyPreset} />
            <ModuleList
              modules={patches}
              groups={groups}
              selected={selectedModules}
              onToggle={toggleModule}
              variantSelections={variantSelections}
              onVariantChange={(patchId, index) => {
                setVariantSelections(prev => new Map(prev).set(patchId, index));
              }}
              installedPatches={installedPatches}
            />
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
            onClick={appState === 'complete' ? () => setAppState('configure') : handleApply}
            disabled={!canApply || (!hasChanges && appState !== 'complete') || applying}
          >
            {appState === 'complete' ? 'Done' : getApplyButtonText()}
          </Button>
        </div>

      </div>

      <p className="text-center text-xs text-muted-foreground mt-8">
        <a
          href="https://ko-fi.com/donutsdelivery"
          target="_blank"
          rel="noopener noreferrer"
          className="hover:text-foreground transition-colors"
        >
          Support on Ko-fi
        </a>
      </p>
    </div>
  );
}

export default App;
