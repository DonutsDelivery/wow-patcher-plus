import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { PRESETS, PresetKey } from '@/lib/presets';

interface Props {
  onSelect: (preset: PresetKey) => void;
}

export function PresetSelector({ onSelect }: Props) {
  return (
    <div className="space-y-2">
      <label className="text-sm font-medium">Quality Preset</label>
      <Select onValueChange={(v) => onSelect(v as PresetKey)}>
        <SelectTrigger className="w-full">
          <SelectValue placeholder="Select a preset..." />
        </SelectTrigger>
        <SelectContent>
          {(Object.entries(PRESETS) as [PresetKey, typeof PRESETS[PresetKey]][]).map(([key, preset]) => (
            <SelectItem key={key} value={key}>
              <div className="flex flex-col">
                <span>{preset.name}</span>
                <span className="text-xs text-muted-foreground">{preset.description}</span>
              </div>
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
