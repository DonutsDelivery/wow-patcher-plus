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

    try {
      const installed = await installPatches(patchIds, onEvent);
      return installed;
    } finally {
      setInstalling(false);
    }
  }, []);

  return { installs, setInstalls, installing, install };
}
