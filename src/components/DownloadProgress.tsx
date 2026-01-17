import { Progress } from '@/components/ui/progress';
import { DownloadState } from '@/hooks/useDownload';
import { Download, CheckCircle, XCircle } from 'lucide-react';

interface Props {
  downloads: Map<string, DownloadState>;
}

function formatBytes(bytes: number): string {
  if (!bytes || bytes <= 0 || !isFinite(bytes)) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(k)), sizes.length - 1);
  if (i < 0 || !isFinite(i)) return '0 B';
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

function formatSpeed(bps: number): string {
  if (!bps || bps <= 0 || !isFinite(bps)) return '-- B/s';
  return formatBytes(bps) + '/s';
}

export function DownloadProgress({ downloads }: Props) {
  if (downloads.size === 0) return null;

  return (
    <div className="space-y-3">
      <h3 className="text-sm font-medium">Downloads</h3>
      {Array.from(downloads.values()).map((dl) => (
        <div key={dl.downloadId} className="space-y-1">
          <div className="flex items-center justify-between text-sm">
            <div className="flex items-center gap-2">
              {dl.status === 'completed' && <CheckCircle className="h-4 w-4 text-green-500" />}
              {dl.status === 'failed' && <XCircle className="h-4 w-4 text-red-500" />}
              {dl.status === 'downloading' && <Download className="h-4 w-4 animate-pulse" />}
              {dl.status === 'pending' && <Download className="h-4 w-4 text-muted-foreground" />}
              <span className="truncate max-w-[150px]">{dl.fileName || 'Starting...'}</span>
            </div>
            <span className="text-muted-foreground">
              {dl.status === 'downloading' && formatSpeed(dl.speedBps)}
              {dl.status === 'completed' && 'Done'}
              {dl.status === 'failed' && 'Failed'}
            </span>
          </div>
          <Progress value={isFinite(dl.percent) ? dl.percent : 0} className="h-2" />
          {dl.error && <p className="text-xs text-red-500">{dl.error}</p>}
        </div>
      ))}
    </div>
  );
}
