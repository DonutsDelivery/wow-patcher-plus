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
  F: 'Fog Pushback',
  P: 'Pretty Night Sky',
  T: 'Dark UI Theme',
  Y: 'Blood Enhanced',
  K: 'New Combat Sounds',
  H: 'Neon Pink Herbs',
  J: 'Glow Down (Elf Eye Glow)',
  W: 'Boneless Undead',
  X: 'High Elf Modifications',
  Q: 'Alt HD Environment',
  R: 'Faithful Upscale',
} as const;

export type PresetKey = keyof typeof PRESETS;
