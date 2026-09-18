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
  accent_text_color?: string;
  is_dark: boolean;
}

export const THEME_PRESETS: Record<ThemePreset, ThemeDefinition> = {
  light: {
    id: 'light',
    name: 'Light Minimal',
    bg_color: '#f8fafc',
    card_color: '#ffffff',
    accent_color: '#059669',
    text_color: '#0f172a',
    secondary_text_color: '#64748b',
    border_color: '#e2e8f0',
    accent_text_color: '#ffffff',
    is_dark: false,
  },
  dark: {
    id: 'dark',
    name: 'Modern Dark',
    bg_color: '#0f172a',
    card_color: '#1e293b',
    accent_color: '#10b981',
    text_color: '#f8fafc',
    secondary_text_color: '#94a3b8',
    border_color: '#334155',
    accent_text_color: '#0f172a',
    is_dark: true,
  },
  midnight: {
    id: 'midnight',
    name: 'Midnight Navy',
    bg_color: '#0a0f1d',
    card_color: '#131c31',
    accent_color: '#38bdf8',
    text_color: '#f8fafc',
    secondary_text_color: '#94a3b8',
    border_color: '#1e2e4a',
    accent_text_color: '#0a0f1d',
    is_dark: true,
  },
  emerald: {
    id: 'emerald',
    name: 'Emerald Forest',
    bg_color: '#091410',
    card_color: '#11231c',
    accent_color: '#10b981',
    text_color: '#f0fdf4',
    secondary_text_color: '#8fa89b',
    border_color: '#1c3b2f',
    accent_text_color: '#091410',
    is_dark: true,
  },
  cyberpunk: {
    id: 'cyberpunk',
    name: 'Cyber Violet',
    bg_color: '#110e1b',
    card_color: '#1c162b',
    accent_color: '#c084fc',
    text_color: '#f8fafc',
    secondary_text_color: '#9d9ab4',
    border_color: '#2f2347',
    accent_text_color: '#110e1b',
    is_dark: true,
  },
  sunset: {
    id: 'sunset',
    name: 'Warm Amber',
    bg_color: '#151311',
    card_color: '#211d19',
    accent_color: '#f59e0b',
    text_color: '#faf6f0',
    secondary_text_color: '#a39e97',
    border_color: '#38302a',
    accent_text_color: '#151311',
    is_dark: true,
  },
  nord: {
    id: 'nord',
    name: 'Nord Frost',
    bg_color: '#242933',
    card_color: '#2e3440',
    accent_color: '#88c0d0',
    text_color: '#eceff4',
    secondary_text_color: '#9aaec4',
    border_color: '#3b4252',
    accent_text_color: '#242933',
    is_dark: true,
  },
  oled: {
    id: 'oled',
    name: 'Pure Black (OLED)',
    bg_color: '#000000',
    card_color: '#121212',
    accent_color: '#10b981',
    text_color: '#f4f4f5',
    secondary_text_color: '#a1a1aa',
    border_color: '#27272a',
    accent_text_color: '#000000',
    is_dark: true,
  },
  custom: {
    id: 'custom',
    name: 'Custom Colors',
    bg_color: '#0f172a',
    card_color: '#1e293b',
    accent_color: '#10b981',
    text_color: '#f8fafc',
    secondary_text_color: '#94a3b8',
    border_color: '#334155',
    accent_text_color: '#0f172a',
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

export function getContrastTextColor(hexColor: string): string {
  let cleanHex = hexColor.replace('#', '').trim();
  if (cleanHex.length === 3) {
    cleanHex = cleanHex.split('').map((c) => c + c).join('');
  }
  if (cleanHex.length !== 6) {
    return '#ffffff';
  }
  const r = parseInt(cleanHex.substring(0, 2), 16);
  const g = parseInt(cleanHex.substring(2, 4), 16);
  const b = parseInt(cleanHex.substring(4, 6), 16);
  // WCAG relative luminance
  const luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
  return luma > 138 ? '#0f172a' : '#ffffff';
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
  accentTextColor: string;
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

  const accentTextColor = getContrastTextColor(accentColor);

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
    accentTextColor,
    bgOpacity,
    bgImage,
    bgImageOpacity,
    bgBlur,
    isDark,
  };
}
