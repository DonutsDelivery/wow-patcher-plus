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

  const startModuleDownload = useCallback(async (module: PatchModule, variantIndex?: number): Promise<void> => {
    // Find appropriate link - use variant index if specified for patches with variants
    const hasVariants = module.variants && module.variants.length > 1;
    const linkIndex = hasVariants && variantIndex !== undefined ? variantIndex : 0;
    const link = module.links[linkIndex] || module.links[0];
    if (!link) throw new Error(`No download link for module ${module.id}`);

    const destDir = await join(await appDataDir(), 'downloads');

    // Create a promise that resolves when download completes or fails
    return new Promise<void>((resolve, reject) => {
      const onProgress = new Channel<DownloadEvent>();

      onProgress.onmessage = (msg) => {
        console.log('[Download Event]', msg.event, msg.data);
        // Update state first
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
              // Ensure percent only ever increases (prevents visual "bouncing")
              const newPercent = Math.max(current.percent, msg.data.percent);
              const newDownloaded = Math.max(current.downloadedBytes, msg.data.downloadedBytes);
              next.set(msg.data.downloadId, {
                ...current,
                downloadedBytes: newDownloaded,
                totalBytes: msg.data.totalBytes,
                speedBps: msg.data.speedBps,
                percent: newPercent,
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

        // Resolve/reject AFTER state update, outside the updater function
        if (msg.event === 'completed') {
          console.log('[Download] Resolving promise for', module.id);
          resolve();
        } else if (msg.event === 'failed') {
          console.log('[Download] Rejecting promise for', module.id, msg.data.error);
          reject(new Error(msg.data.error || 'Download failed'));
        }
      };

      const targetFilename = `Patch-${module.id.toUpperCase()}.mpq`;
      console.log('[Download] Starting', module.id, link.url);
      startDownload(link.url, link.provider, destDir, onProgress, targetFilename)
        .then(id => console.log('[Download] Got ID', id, 'for', module.id))
        .catch(err => {
          console.log('[Download] Start failed for', module.id, err);
          reject(err);
        });
    });
  }, []);

  const downloadAll = useCallback(async (modules: PatchModule[], variantSelections?: Map<string, number>) => {
    console.log('[DownloadAll] Starting downloads for', modules.map(m => m.id));
    const results = await Promise.allSettled(
      modules.map(m => startModuleDownload(m, variantSelections?.get(m.id)))
    );
    console.log('[DownloadAll] All downloads settled', results);
    return results;
  }, [startModuleDownload]);

  return { downloads, startModuleDownload, downloadAll };
}
