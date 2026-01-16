import { Button } from '@/components/ui/button';
import { Folder, Check, AlertCircle } from 'lucide-react';

interface Props {
  path: string | null;
  loading: boolean;
  onPick: () => void;
}

export function FolderPicker({ path, loading, onPick }: Props) {
  return (
    <div className="space-y-2">
      <label className="text-sm font-medium">WoW Installation</label>
      <div className="flex items-center gap-2">
        <Button variant="outline" onClick={onPick} disabled={loading}>
          <Folder className="mr-2 h-4 w-4" />
          {loading ? 'Detecting...' : 'Select Folder'}
        </Button>
        {path && (
          <div className="flex items-center text-sm text-muted-foreground">
            <Check className="mr-1 h-4 w-4 text-green-500" />
            <span className="truncate max-w-[200px]">{path}</span>
          </div>
        )}
        {!path && !loading && (
          <div className="flex items-center text-sm text-muted-foreground">
            <AlertCircle className="mr-1 h-4 w-4 text-yellow-500" />
            <span>No folder selected</span>
          </div>
        )}
      </div>
    </div>
  );
}
