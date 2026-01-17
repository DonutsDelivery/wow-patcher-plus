import { useState } from 'react';
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
import { verifyPatches, repairPatches, InstallEvent } from '@/lib/tauri';
import { Channel } from '@tauri-apps/api/core';
import { Loader2, RefreshCw } from 'lucide-react';
import './App.css';

type AppState = 'configure' | 'downloading' | 'installing' | 'complete';

function App() {
  const [appState, setAppState] = useState<AppState>('configure');
  const { patches, groups, selectedModules, loading, error, applyPreset, toggleModule } = usePatches();
  const { wowPath, loading: pathLoading, pickFolder } = useWowPath();
  const { downloads, downloadAll } = useDownload();
  const { installs, install, setInstalls } = useInstall();
  const [verifyResults, setVerifyResults] = useState<Map<string, string>>(new Map());
  const [repairing, setRepairing] = useState(false);
  const [verifying, setVerifying] = useState(false);
  const [updating, setUpdating] = useState(false);
  const [variantSelections, setVariantSelections] = useState<Map<string, number>>(new Map());

  const selectedPatches = patches.filter(p => selectedModules.has(p.id));
  const canStart = selectedModules.size > 0 && wowPath !== null;

  const handleVerify = async () => {
    setVerifying(true);
    setVerifyResults(new Map());
    try {
      const results = await verifyPatches(Array.from(selectedModules));
      const resultMap = new Map<string, string>();
      results.forEach(([id, result]) => {
        resultMap.set(id, result.status);
      });
      setVerifyResults(resultMap);
    } finally {
      setVerifying(false);
    }
  };

  const handleRepair = async () => {
    setRepairing(true);
    setVerifyResults(new Map());
    try {
      const onEvent = new Channel<InstallEvent>();
      onEvent.onmessage = (msg) => {
        setInstalls(prev => {
          const next = new Map(prev);
          const patchId = msg.data.patchId;
          const current = next.get(patchId) || {
            patchId,
            fileName: '',
            copiedBytes: 0,
            totalBytes: 0,
            percent: 0,
            status: 'pending' as const,
          };

          switch (msg.event) {
            case 'started':
              next.set(patchId, {
                ...current,
                fileName: msg.data.fileName,
                status: 'installing',
              });
              break;
            case 'progress':
              next.set(patchId, {
                ...current,
                copiedBytes: msg.data.bytesCopied,
                totalBytes: msg.data.totalBytes,
                percent: msg.data.totalBytes > 0
                  ? (msg.data.bytesCopied / msg.data.totalBytes) * 100
                  : 0,
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
      await repairPatches(Array.from(selectedModules), onEvent);
    } finally {
      setRepairing(false);
    }
  };

  const handleUpdateAll = async () => {
    if (!canStart) return;
    setUpdating(true);
    setAppState('downloading');

    try {
      // Re-download all selected patches
      await downloadAll(selectedPatches, variantSelections);

      // Re-install them
      setAppState('installing');
      await install(Array.from(selectedModules));

      setAppState('complete');
    } finally {
      setUpdating(false);
    }
  };

  const handleStart = async () => {
    if (!canStart) return;

    console.log('[App] Starting download phase');
    setAppState('downloading');
    await downloadAll(selectedPatches, variantSelections);
    console.log('[App] Download phase complete, starting install');

    setAppState('installing');
    await install(Array.from(selectedModules));
    console.log('[App] Install phase complete');

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
            <FolderPicker path={wowPath} loading={pathLoading} onPick={pickFolder} />
            <RequirementsPanel wowPath={wowPath} />
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
          <Card>
            <CardHeader>
              <CardTitle className="text-green-500">Installation Complete!</CardTitle>
              <CardDescription>You can now launch WoW.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex flex-wrap gap-3 justify-center">
                <Button variant="outline" onClick={handleVerify} disabled={verifying || repairing || updating}>
                  {verifying ? 'Verifying...' : 'Verify'}
                </Button>
                <Button variant="outline" onClick={handleRepair} disabled={repairing || verifying || updating}>
                  {repairing ? 'Repairing...' : 'Repair'}
                </Button>
                <Button variant="default" onClick={handleUpdateAll} disabled={updating || verifying || repairing}>
                  <RefreshCw className={`h-4 w-4 mr-2 ${updating ? 'animate-spin' : ''}`} />
                  {updating ? 'Updating...' : 'Update ALL'}
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
      </div>

      <p className="text-center text-xs text-muted-foreground mt-8">
        <a
          href="https://ko-fi.com/donutsdelivery"
          target="_blank"
          rel="noopener noreferrer"
          className="hover:text-foreground transition-colors"
        >
          ☕ Support on Ko-fi
        </a>
      </p>
    </div>
  );
}

export default App;
