import { useState, useEffect, useCallback } from 'react';
import { fetchPatches, autoSelectDeps, PatchModule, PatchGroup } from '@/lib/tauri';
import { PRESETS, PresetKey } from '@/lib/presets';

export function usePatches() {
  const [patches, setPatches] = useState<PatchModule[]>([]);
  const [groups, setGroups] = useState<PatchGroup[]>([]);
  const [selectedModules, setSelectedModules] = useState<Set<string>>(new Set());
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchPatches()
      .then(response => {
        setPatches(response.patches);
        setGroups(response.groups);
      })
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

  return { patches, groups, selectedModules, loading, error, applyPreset, toggleModule, setSelectedModules };
}
