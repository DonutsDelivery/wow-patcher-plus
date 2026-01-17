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

    const targetFilename = `Patch-${module.id.toUpperCase()}.mpq`;
    const downloadId = await startDownload(link.url, link.provider, destDir, onProgress, targetFilename);
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
