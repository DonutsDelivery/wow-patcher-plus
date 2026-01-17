import { useEffect, useState } from 'react';
import { CheckCircle, XCircle, Download, Loader2, AlertTriangle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { checkRequirements, installVanillaHelpers, installDxvk, RequirementsStatus } from '@/lib/tauri';

interface Props {
  wowPath: string | null;
}

export function RequirementsPanel({ wowPath }: Props) {
  const [status, setStatus] = useState<RequirementsStatus | null>(null);
  const [installingVH, setInstallingVH] = useState(false);
  const [installingDxvk, setInstallingDxvk] = useState(false);
  const [dxvkVersion, setDxvkVersion] = useState<'2.7.1' | '2.5.3'>('2.7.1');
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const refreshStatus = async () => {
    if (wowPath) {
      const s = await checkRequirements();
      setStatus(s);
    }
  };

  useEffect(() => {
    refreshStatus();
  }, [wowPath]);

  const handleInstallVH = async () => {
    setInstallingVH(true);
    setError(null);
    setSuccess(null);
    try {
      await installVanillaHelpers();
      await refreshStatus();
      setSuccess('VanillaHelpers installed successfully!');
      setTimeout(() => setSuccess(null), 3000);
    } catch (e) {
      setError(`VanillaHelpers: ${e}`);
    } finally {
      setInstallingVH(false);
    }
  };

  const handleInstallDxvk = async () => {
    setInstallingDxvk(true);
    setError(null);
    setSuccess(null);
    try {
      await installDxvk(dxvkVersion);
      await refreshStatus();
      setSuccess(`DXVK ${dxvkVersion} installed successfully!`);
      setTimeout(() => setSuccess(null), 3000);
    } catch (e) {
      setError(`DXVK: ${e}`);
    } finally {
      setInstallingDxvk(false);
    }
  };

  if (!wowPath || !status) {
    return null;
  }

  const allGood = status.vanilla_helpers && status.dxvk;

  return (
    <div className={`rounded-lg border p-3 ${allGood ? 'border-green-500/30 bg-green-500/5' : 'border-yellow-500/30 bg-yellow-500/5'}`}>
      <div className="flex items-center gap-2 mb-3">
        {allGood ? (
          <CheckCircle className="h-4 w-4 text-green-500" />
        ) : (
          <AlertTriangle className="h-4 w-4 text-yellow-500" />
        )}
        <span className="text-sm font-semibold">
          {allGood ? 'Requirements Installed' : 'Required Mods'}
        </span>
      </div>

      <div className="space-y-2">
        {/* VanillaHelpers */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            {status.vanilla_helpers ? (
              <CheckCircle className="h-4 w-4 text-green-500" />
            ) : (
              <XCircle className="h-4 w-4 text-red-500" />
            )}
            <div>
              <span className="text-sm">VanillaHelpers</span>
              <span className="text-xs text-red-400 ml-1">(required)</span>
            </div>
          </div>
          <Button
            size="sm"
            variant="outline"
            onClick={handleInstallVH}
            disabled={installingVH}
            className="h-7 text-xs"
          >
            {installingVH ? (
              <Loader2 className="h-3 w-3 animate-spin" />
            ) : (
              <>
                <Download className="h-3 w-3 mr-1" />
                {status.vanilla_helpers ? 'Update' : 'Install'}
              </>
            )}
          </Button>
        </div>

        {/* DXVK */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            {status.dxvk ? (
              <CheckCircle className="h-4 w-4 text-green-500" />
            ) : (
              <XCircle className="h-4 w-4 text-yellow-500" />
            )}
            <div>
              <span className="text-sm">DXVK</span>
              <span className="text-xs text-yellow-400 ml-1">(recommended)</span>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <select
              value={dxvkVersion}
              onChange={(e) => setDxvkVersion(e.target.value as '2.7.1' | '2.5.3')}
              disabled={installingDxvk}
              className="h-7 text-xs bg-background border rounded px-1"
            >
              <option value="2.7.1">v2.7.1 (NVIDIA)</option>
              <option value="2.5.3">v2.5.3 (AMD)</option>
            </select>
            <Button
              size="sm"
              variant="outline"
              onClick={handleInstallDxvk}
              disabled={installingDxvk}
              className="h-7 text-xs"
            >
              {installingDxvk ? (
                <Loader2 className="h-3 w-3 animate-spin" />
              ) : (
                <>
                  <Download className="h-3 w-3 mr-1" />
                  {status.dxvk ? 'Update' : 'Install'}
                </>
              )}
            </Button>
          </div>
        </div>
      </div>

      {error && (
        <div className="mt-2 p-2 rounded bg-red-500/20 border border-red-500/50">
          <p className="text-xs text-red-400 font-medium">{error}</p>
        </div>
      )}

      {success && (
        <div className="mt-2 p-2 rounded bg-green-500/20 border border-green-500/50">
          <p className="text-xs text-green-400 font-medium">{success}</p>
        </div>
      )}

      {!status.vanilla_helpers && (
        <p className="mt-2 text-xs text-yellow-500">
          HD Patches require VanillaHelpers to work!
        </p>
      )}
    </div>
  );
}
