import { Progress } from '@/components/ui/progress';
import { InstallState } from '@/hooks/useInstall';
import { HardDrive, CheckCircle, XCircle } from 'lucide-react';

interface Props {
  installs: Map<string, InstallState>;
}

export function InstallProgress({ installs }: Props) {
  if (installs.size === 0) return null;

  return (
    <div className="space-y-3">
      <h3 className="text-sm font-medium">Installing</h3>
      {Array.from(installs.values()).map((inst) => (
        <div key={inst.patchId} className="space-y-1">
          <div className="flex items-center justify-between text-sm">
            <div className="flex items-center gap-2">
              {inst.status === 'completed' && <CheckCircle className="h-4 w-4 text-green-500" />}
              {inst.status === 'failed' && <XCircle className="h-4 w-4 text-red-500" />}
              {inst.status === 'installing' && <HardDrive className="h-4 w-4 animate-pulse" />}
              {inst.status === 'pending' && <HardDrive className="h-4 w-4 text-muted-foreground" />}
              <span>Module {inst.patchId}</span>
            </div>
            <span className="text-muted-foreground">
              {inst.percent.toFixed(0)}%
            </span>
          </div>
          <Progress value={inst.percent} className="h-2" />
          {inst.error && <p className="text-xs text-red-500">{inst.error}</p>}
        </div>
      ))}
    </div>
  );
}
