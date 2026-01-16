import { ScrollArea } from '@/components/ui/scroll-area';
import { Checkbox } from '@/components/ui/checkbox';
import { PatchModule } from '@/lib/tauri';

interface Props {
  modules: PatchModule[];
  selected: Set<string>;
  onToggle: (moduleId: string) => void;
}

export function ModuleList({ modules, selected, onToggle }: Props) {
  return (
    <ScrollArea className="h-[300px] rounded-md border p-4">
      <div className="space-y-4">
        {modules.map((mod) => (
          <div key={mod.id} className="flex items-start space-x-3">
            <Checkbox
              id={mod.id}
              checked={selected.has(mod.id)}
              onCheckedChange={() => onToggle(mod.id)}
            />
            <div className="grid gap-1.5 leading-none">
              <label htmlFor={mod.id} className="text-sm font-medium cursor-pointer">
                {mod.id}: {mod.name}
              </label>
              <p className="text-xs text-muted-foreground">{mod.description}</p>
              {mod.dependencies.length > 0 && (
                <p className="text-xs text-muted-foreground">
                  Requires: {mod.dependencies.join(', ')}
                </p>
              )}
            </div>
          </div>
        ))}
      </div>
    </ScrollArea>
  );
}
