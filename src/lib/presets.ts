export const PRESETS = {
  low: {
    name: 'Low',
    description: 'Minimal HD patches for low-end systems',
    modules: ['I', 'M'],
  },
  medium: {
    name: 'Medium',
    description: 'Core visual improvements',
    modules: ['A', 'C', 'G', 'I', 'M', 'V'],
  },
  high: {
    name: 'High',
    description: 'Comprehensive HD overhaul',
    modules: ['A', 'B', 'C', 'D', 'E', 'G', 'I', 'M', 'S', 'V'],
  },
  ultra: {
    name: 'Ultra',
    description: 'Maximum quality with 4K textures',
    modules: ['A', 'B', 'C', 'D', 'E', 'G', 'I', 'M', 'S', 'U', 'V'],
  },
} as const;

export const OPTIONAL_MODULES = {
  L: 'A Little Extra for Females',
  N: 'Darker Nights',
  O: 'Raid Visuals Mod',
} as const;

export type PresetKey = keyof typeof PRESETS;
