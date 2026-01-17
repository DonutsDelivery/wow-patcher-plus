import { Checkbox } from '@/components/ui/checkbox';
import { PatchModule } from '@/lib/tauri';
import { Lock, Link } from 'lucide-react';

interface Props {
  modules: PatchModule[];
  selected: Set<string>;
  onToggle: (moduleId: string) => void;
  variantSelections: Map<string, number>;
  onVariantChange: (patchId: string, index: number) => void;
}

// B, D, E must be installed together - they're a linked group
const BDE_GROUP = ['B', 'D', 'E'];

// Check if all dependencies for a module are satisfied
function getDependencyStatus(mod: PatchModule, selected: Set<string>): { satisfied: boolean; missing: string[] } {
  const missing = mod.dependencies.filter(dep => !selected.has(dep));
  return { satisfied: missing.length === 0, missing };
}

// Group definitions for organizing patches
const groups = [
  {
    name: 'Core Visuals',
    description: 'Standalone patches',
    ids: ['A', 'C', 'G', 'I', 'M', 'V'],
  },
  {
    name: 'Environment Pack',
    description: 'Linked - toggles together',
    ids: ['B', 'D', 'E'],
    linked: true,
  },
  {
    name: 'Audio & Atmosphere',
    description: 'Sound and lighting options',
    ids: ['S', 'N', 'O'],
  },
  {
    name: 'Character Extras',
    description: 'Require character patches (A, G)',
    ids: ['L', 'U'],
  },
];

interface ModuleItemProps {
  mod: PatchModule;
  selected: Set<string>;
  onToggle: (moduleId: string) => void;
  isLinkedGroup?: boolean;
  variantIndex: number;
  onVariantChange: (index: number) => void;
}

function ModuleItem({ mod, selected, onToggle, isLinkedGroup, variantIndex, onVariantChange }: ModuleItemProps) {
  const { satisfied, missing } = getDependencyStatus(mod, selected);
  const isDisabled = !satisfied && mod.dependencies.length > 0;

  // Check if this patch has named variants
  const hasVariants = mod.variants && mod.variants.length > 1;

  return (
    <div className={`flex items-start space-x-2 py-1 ${isDisabled ? 'opacity-50' : ''}`}>
      <Checkbox
        id={mod.id}
        checked={selected.has(mod.id)}
        onCheckedChange={() => onToggle(mod.id)}
        disabled={isDisabled}
        className="mt-0.5"
      />
      <div className="min-w-0 flex-1">
        <label
          htmlFor={mod.id}
          className={`text-sm font-medium leading-tight ${isDisabled ? 'cursor-not-allowed' : 'cursor-pointer'}`}
        >
          <span className="font-bold">{mod.id}</span>: {mod.name}
          {isDisabled && <Lock className="inline-block ml-1 h-3 w-3 text-yellow-500" />}
          {isLinkedGroup && <Link className="inline-block ml-1 h-3 w-3 text-blue-400" />}
        </label>
        {isDisabled && (
          <p className="text-xs text-yellow-500">
            Needs: {missing.join(', ')}
          </p>
        )}
        {hasVariants && selected.has(mod.id) && (
          <select
            value={variantIndex}
            onChange={(e) => onVariantChange(parseInt(e.target.value))}
            className="mt-1 w-full text-xs bg-background border rounded px-1 py-0.5"
            onClick={(e) => e.stopPropagation()}
          >
            {mod.variants!.map((variant, idx) => (
              <option key={idx} value={idx}>
                {variant}
              </option>
            ))}
          </select>
        )}
      </div>
    </div>
  );
}

export function ModuleList({ modules, selected, onToggle, variantSelections, onVariantChange }: Props) {
  const moduleMap = new Map(modules.map(m => [m.id, m]));

  // Wrap onToggle to handle B/D/E group
  const handleToggle = (moduleId: string) => {
    if (BDE_GROUP.includes(moduleId)) {
      // Toggle all B, D, E together
      const anySelected = BDE_GROUP.some(id => selected.has(id));
      if (anySelected) {
        // Deselect all
        BDE_GROUP.forEach(id => {
          if (selected.has(id)) onToggle(id);
        });
      } else {
        // Select all
        BDE_GROUP.forEach(id => {
          if (!selected.has(id)) onToggle(id);
        });
      }
    } else {
      onToggle(moduleId);
    }
  };

  return (
    <div className="grid grid-cols-2 gap-4">
      {groups.map((group) => {
        const groupModules = group.ids
          .map(id => moduleMap.get(id))
          .filter((m): m is PatchModule => m !== undefined);

        if (groupModules.length === 0) return null;

        return (
          <div key={group.name} className="rounded-lg border p-3 bg-card">
            <div className="mb-2 pb-2 border-b">
              <h3 className="text-sm font-semibold">
                {group.name}
                {group.linked && <Link className="inline-block ml-1 h-3 w-3 text-blue-400" />}
              </h3>
              <p className="text-xs text-muted-foreground">{group.description}</p>
            </div>
            <div className="space-y-1">
              {groupModules.map((mod) => (
                <ModuleItem
                  key={mod.id}
                  mod={mod}
                  selected={selected}
                  onToggle={handleToggle}
                  isLinkedGroup={group.linked}
                  variantIndex={variantSelections.get(mod.id) ?? 0}
                  onVariantChange={(index) => onVariantChange(mod.id, index)}
                />
              ))}
            </div>
          </div>
        );
      })}
    </div>
  );
}
