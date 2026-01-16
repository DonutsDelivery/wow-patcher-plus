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
