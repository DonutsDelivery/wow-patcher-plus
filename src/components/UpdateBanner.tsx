import { useEffect, useState } from 'react';
import { checkForUpdates, UpdateInfo } from '@/lib/tauri';
import { openUrl } from '@tauri-apps/plugin-opener';
import { Download, X } from 'lucide-react';
import { Button } from '@/components/ui/button';

export function UpdateBanner() {
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [dismissed, setDismissed] = useState(false);

  useEffect(() => {
    checkForUpdates()
      .then(info => {
        if (info.updateAvailable) {
          setUpdateInfo(info);
        }
      })
      .catch(() => {
        // Silently fail - no update check available
      });
  }, []);

  if (!updateInfo || !updateInfo.updateAvailable || dismissed) {
    return null;
  }

  const handleDownload = () => {
    if (updateInfo.downloadUrl) {
      openUrl(updateInfo.downloadUrl);
    }
  };

  return (
    <div className="bg-blue-500/20 border border-blue-500/50 rounded-lg p-3 flex items-center justify-between">
      <div className="flex items-center gap-3">
        <Download className="h-5 w-5 text-blue-400" />
        <div>
          <p className="text-sm font-medium">
            Update available: v{updateInfo.latestVersion}
          </p>
          <p className="text-xs text-muted-foreground">
            Current: v{updateInfo.currentVersion}
          </p>
        </div>
      </div>
      <div className="flex items-center gap-2">
        {updateInfo.downloadUrl && (
          <Button size="sm" variant="outline" onClick={handleDownload}>
            Download
          </Button>
        )}
        <Button size="sm" variant="ghost" onClick={() => setDismissed(true)}>
          <X className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}
