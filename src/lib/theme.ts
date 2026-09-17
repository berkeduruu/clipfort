import type { AppSettings, ThemePreset } from '$lib/types';

export interface ThemeDefinition {
  id: ThemePreset;
  name: string;
  bg_color: string;
  card_color: string;
  accent_color: string;
  text_color: string;
  secondary_text_color: string;
  border_color: string;
  is_dark: boolean;
}

export const THEME_PRESETS: Record<ThemePreset, ThemeDefinition> = {
  light: {
    id: 'light',
    name: 'Light Minimal',
    bg_color: '#f8fafc',
    card_color: '#ffffff',
    accent_color: '#10b981',
    text_color: '#0f172a',
    secondary_text_color: '#475569',
    border_color: '#e2e8f0',
    is_dark: false,
  },
  dark: {
    id: 'dark',
    name: 'Modern Dark',
    bg_color: '#0f172a',
    card_color: '#1e293b',
    accent_color: '#10b981',
    text_color: '#f1f5f9',
    secondary_text_color: '#94a3b8',
    border_color: '#334155',
    is_dark: true,
  },
  midnight: {
    id: 'midnight',
    name: 'Midnight Blue',
    bg_color: '#0b132b',
    card_color: '#1c2541',
    accent_color: '#38bdf8',
    text_color: '#f8fafc',
    secondary_text_color: '#94a3b8',
    border_color: '#273b60',
    is_dark: true,
  },
  emerald: {
    id: 'emerald',
    name: 'Emerald Forest',
    bg_color: '#06281e',
    card_color: '#0c3b2e',
    accent_color: '#34d399',
    text_color: '#ecfdf5',
    secondary_text_color: '#a7f3d0',
    border_color: '#165946',
    is_dark: true,
  },
  cyberpunk: {
    id: 'cyberpunk',
    name: 'Cyber Violet',
    bg_color: '#140c24',
    card_color: '#241442',
    accent_color: '#e879f9',
    text_color: '#fdf4ff',
    secondary_text_color: '#c084fc',
    border_color: '#472175',
    is_dark: true,
  },
  sunset: {
    id: 'sunset',
    name: 'Warm Amber',
    bg_color: '#1c140d',
    card_color: '#2e1f14',
    accent_color: '#f59e0b',
    text_color: '#fffbeb',
    secondary_text_color: '#fcd34d',
    border_color: '#4a3321',
    is_dark: true,
  },
  custom: {
    id: 'custom',
    name: 'Custom Colors',
    bg_color: '#0f172a',
    card_color: '#1e293b',
    accent_color: '#10b981',
    text_color: '#f1f5f9',
    secondary_text_color: '#94a3b8',
    border_color: '#334155',
    is_dark: true,
  },
};

export function hexToRgba(hex: string, alphaPercent: number = 100): string {
  let cleanHex = hex.replace('#', '').trim();
  if (cleanHex.length === 3) {
    cleanHex = cleanHex.split('').map((c) => c + c).join('');
  }
  if (cleanHex.length !== 6) {
    return hex;
  }
  const r = parseInt(cleanHex.substring(0, 2), 16);
  const g = parseInt(cleanHex.substring(2, 4), 16);
  const b = parseInt(cleanHex.substring(4, 6), 16);
  const a = Math.max(0, Math.min(1, alphaPercent / 100));
  return `rgba(${r}, ${g}, ${b}, ${a})`;
}

export function getBlurClass(blur?: string): string {
  switch (blur) {
    case 'none':
      return 'backdrop-blur-none';
    case 'md':
      return 'backdrop-blur-md';
    case 'lg':
      return 'backdrop-blur-xl';
    case 'sm':
    default:
      return 'backdrop-blur-sm';
  }
}

export function getEffectiveTheme(settings: AppSettings): {
  bgColor: string;
  cardColor: string;
  accentColor: string;
  textColor: string;
  secondaryTextColor: string;
  borderColor: string;
  bgOpacity: number;
  bgImage: string | null;
  bgImageOpacity: number;
  bgBlur: string;
  isDark: boolean;
} {
  const presetKey = (settings.theme_preset as ThemePreset) || 'light';
  const preset = THEME_PRESETS[presetKey] || THEME_PRESETS.light;

  const bgColor = settings.custom_bg_color || preset.bg_color;
  const cardColor = settings.custom_card_color || preset.card_color;
  const accentColor = settings.custom_accent_color || preset.accent_color;
  const textColor = settings.custom_text_color || preset.text_color;
  const secondaryTextColor = settings.custom_secondary_text_color || preset.secondary_text_color || '#64748b';
  const borderColor = settings.custom_border_color || preset.border_color;
  const bgOpacity = settings.bg_opacity !== undefined ? settings.bg_opacity : 95;
  const bgImage = settings.bg_image || null;
  const bgImageOpacity = settings.bg_image_opacity !== undefined ? settings.bg_image_opacity : 25;
  const bgBlur = settings.bg_blur || 'sm';

  // Calculate perceived brightness of background to decide if it's dark
  let isDark = preset.is_dark;
  const cleanHex = bgColor.replace('#', '').trim();
  if (cleanHex.length === 6) {
    const r = parseInt(cleanHex.substring(0, 2), 16);
    const g = parseInt(cleanHex.substring(2, 4), 16);
    const b = parseInt(cleanHex.substring(4, 6), 16);
    const luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    isDark = luma < 128;
  }

  return {
    bgColor,
    cardColor,
    accentColor,
    textColor,
    secondaryTextColor,
    borderColor,
    bgOpacity,
    bgImage,
    bgImageOpacity,
    bgBlur,
    isDark,
  };
}
