export interface ClipItem {
  id: string;
  item_type: 'text' | 'image';
  content: string;
  preview: string;
  timestamp: number;
  pinned: boolean;
  char_count?: number;
  word_count?: number;
  image_width?: number;
  image_height?: number;
  file_size_bytes?: number;
}

export type ThemePreset = 'light' | 'dark' | 'midnight' | 'emerald' | 'cyberpunk' | 'sunset' | 'custom';

export interface AppSettings {
  global_shortcut: string;
  max_history: number;
  close_on_copy: boolean;
  number_keys_copy: boolean;
  play_sound: boolean;
  run_at_startup: boolean;
  max_vault_file_size_mb: number;
  window_width?: number;
  window_height?: number;
  // Theme & Customization
  theme_preset?: ThemePreset | string;
  custom_bg_color?: string;
  custom_card_color?: string;
  custom_accent_color?: string;
  custom_text_color?: string;
  custom_secondary_text_color?: string;
  custom_border_color?: string;
  bg_opacity?: number;
  bg_image?: string | null;
  bg_image_opacity?: number;
  bg_blur?: 'none' | 'sm' | 'md' | 'lg' | string;
}

export type FilterCategory = 'all' | 'text' | 'image' | 'pinned';

export interface VaultItem {
  id: string;
  title: string;
  secret: string;
  item_type: 'text' | 'password' | 'file' | 'note' | 'link' | string;
  tab: string;
  group: string;
  pinned: boolean;
  file_name?: string;
  file_size_bytes?: number;
  file_id?: string;
  file_path?: string;
  notes?: string;
  created_at: number;
  updated_at: number;
}

export interface VaultStatus {
  is_initialized: boolean;
  is_unlocked: boolean;
  has_pin: boolean;
  vault_path: string;
  item_count: number;
  tabs: string[];
}

export type AppMode = 'clipboard' | 'vault';

export type ResizeDirection =
  | 'East'
  | 'North'
  | 'NorthEast'
  | 'NorthWest'
  | 'South'
  | 'SouthEast'
  | 'SouthWest'
  | 'West';
