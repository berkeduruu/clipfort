import type { AppSettings } from './types';

export interface ShortcutDefinition {
  id: keyof AppSettings;
  label: string;
  description: string;
  category: 'global' | 'actions' | 'navigation';
  defaultKey: string;
  isGlobal?: boolean;
}

export const DEFAULT_SHORTCUTS: Record<string, string> = {
  global_shortcut: 'Alt+Shift+Q',
  shortcut_copy: 'Enter',
  shortcut_delete: 'Delete',
  shortcut_pin: 'P',
  shortcut_move_up: 'Alt+ArrowUp',
  shortcut_move_down: 'Alt+ArrowDown',
  shortcut_search: 'Ctrl+F',
  shortcut_clear: 'Ctrl+Delete',
  shortcut_close: 'Escape',
  shortcut_toggle_vault: 'Ctrl+Tab',
  shortcut_settings: 'Ctrl+,',
  shortcut_export: 'Ctrl+E',
};

export const SHORTCUT_DEFINITIONS: ShortcutDefinition[] = [
  // 1. Global
  {
    id: 'global_shortcut',
    label: 'Global Show / Hide',
    description: 'Toggle ClipFort window from anywhere in the system',
    category: 'global',
    defaultKey: 'Alt+Shift+Q',
    isGlobal: true,
  },
  // 2. Clipboard Actions
  {
    id: 'shortcut_copy',
    label: 'Copy Selected Item',
    description: 'Copy selected clip to clipboard and hide window',
    category: 'actions',
    defaultKey: 'Enter',
  },
  {
    id: 'shortcut_delete',
    label: 'Delete Item',
    description: 'Delete the selected clip from history',
    category: 'actions',
    defaultKey: 'Delete',
  },
  {
    id: 'shortcut_pin',
    label: 'Pin / Unpin Item',
    description: 'Toggle pin on the selected clip to keep it at top',
    category: 'actions',
    defaultKey: 'P',
  },
  {
    id: 'shortcut_move_up',
    label: 'Move Item Up',
    description: 'Reorder selected clip higher in the list',
    category: 'actions',
    defaultKey: 'Alt+ArrowUp',
  },
  {
    id: 'shortcut_move_down',
    label: 'Move Item Down',
    description: 'Reorder selected clip lower in the list',
    category: 'actions',
    defaultKey: 'Alt+ArrowDown',
  },
  {
    id: 'shortcut_search',
    label: 'Focus Search',
    description: 'Focus the search box to search clips immediately',
    category: 'actions',
    defaultKey: 'Ctrl+F',
  },
  {
    id: 'shortcut_clear',
    label: 'Clear Unpinned History',
    description: 'Delete all unpinned items from clipboard history',
    category: 'actions',
    defaultKey: 'Ctrl+Delete',
  },
  // 3. Navigation & Modes
  {
    id: 'shortcut_close',
    label: 'Close / Hide Window',
    description: 'Hide ClipFort or close active settings modal',
    category: 'navigation',
    defaultKey: 'Escape',
  },
  {
    id: 'shortcut_toggle_vault',
    label: 'Switch Clipboard / Vault',
    description: 'Toggle between Clipboard History and Encrypted Vault',
    category: 'navigation',
    defaultKey: 'Ctrl+Tab',
  },
  {
    id: 'shortcut_settings',
    label: 'Open Settings',
    description: 'Open ClipFort settings dialog',
    category: 'navigation',
    defaultKey: 'Ctrl+,',
  },
  {
    id: 'shortcut_export',
    label: 'Export History',
    description: 'Open export modal for clipboard history',
    category: 'navigation',
    defaultKey: 'Ctrl+E',
  },
];

/**
 * Standardize single key names into clean canonical representations.
 */
export function canonicalizeKey(key: string): string {
  if (!key) return '';

  const lower = key.toLowerCase();
  switch (lower) {
    case 'arrowup':
    case 'up':
      return 'ArrowUp';
    case 'arrowdown':
    case 'down':
      return 'ArrowDown';
    case 'arrowleft':
    case 'left':
      return 'ArrowLeft';
    case 'arrowright':
    case 'right':
      return 'ArrowRight';
    case 'escape':
    case 'esc':
      return 'Escape';
    case 'enter':
    case 'return':
      return 'Enter';
    case 'delete':
    case 'del':
      return 'Delete';
    case 'backspace':
      return 'Backspace';
    case 'tab':
      return 'Tab';
    case ' ':
    case 'space':
    case 'spacebar':
      return 'Space';
    default:
      // Function keys (F1-F12)
      if (/^f[1-9][0-2]?$/i.test(key)) {
        return key.toUpperCase();
      }
      // Single character keys: uppercase letter or literal char
      if (key.length === 1) {
        return key.toUpperCase();
      }
      return key;
  }
}

/**
 * Convert a KeyboardEvent into a standardized shortcut string representation.
 * Returns null if only modifier keys were pressed.
 */
export function parseKeyboardEvent(e: KeyboardEvent, isGlobal = false): string | null {
  const isModifierOnly = ['Control', 'Alt', 'Shift', 'Meta'].includes(e.key);
  if (isModifierOnly) return null;

  const parts: string[] = [];
  if (e.ctrlKey) parts.push('Ctrl');
  if (e.altKey) parts.push('Alt');
  if (e.shiftKey) parts.push('Shift');
  if (e.metaKey) parts.push('Super');

  const mainKey = canonicalizeKey(e.key);
  if (!mainKey) return null;

  // Global hotkeys in OS require at least one modifier key (or a Function key)
  if (isGlobal && parts.length === 0 && !/^F\d+$/.test(mainKey)) {
    return null;
  }

  parts.push(mainKey);
  return parts.join('+');
}

/**
 * Parse a shortcut string (e.g. "Ctrl+Shift+Q" or "Alt+ArrowUp" or "Escape") into structured parts.
 */
export interface ParsedShortcut {
  ctrl: boolean;
  alt: boolean;
  shift: boolean;
  meta: boolean;
  key: string;
}

export function parseShortcutString(str: string | undefined): ParsedShortcut | null {
  if (!str || typeof str !== 'string') return null;
  const rawParts = str.split('+').map((p) => p.trim()).filter(Boolean);
  if (rawParts.length === 0) return null;

  let ctrl = false;
  let alt = false;
  let shift = false;
  let meta = false;
  let key = '';

  for (const part of rawParts) {
    const p = part.toLowerCase();
    if (p === 'ctrl' || p === 'control') {
      ctrl = true;
    } else if (p === 'alt' || p === 'option') {
      alt = true;
    } else if (p === 'shift') {
      shift = true;
    } else if (p === 'super' || p === 'meta' || p === 'cmd' || p === 'command') {
      meta = true;
    } else {
      key = canonicalizeKey(part);
    }
  }

  if (!key) return null;
  return { ctrl, alt, shift, meta, key };
}

/**
 * Check if a KeyboardEvent matches a configured shortcut string.
 */
export function matchesShortcut(e: KeyboardEvent, shortcutStr: string | undefined): boolean {
  const parsed = parseShortcutString(shortcutStr);
  if (!parsed) return false;

  if (e.ctrlKey !== parsed.ctrl) return false;
  if (e.altKey !== parsed.alt) return false;
  if (e.metaKey !== parsed.meta) return false;

  // For Shift: if the shortcut explicitly requires Shift, event must have shiftKey.
  if (parsed.shift && !e.shiftKey) return false;
  if (!parsed.shift && e.shiftKey) {
    // If user defined a shortcut without Shift (e.g. 'P'), but typed Shift+P, they don't match.
    return false;
  }

  const eventKey = canonicalizeKey(e.key);
  return eventKey.toLowerCase() === parsed.key.toLowerCase();
}

/**
 * Pretty formatted key badges for UI display.
 */
export function getShortcutDisplayParts(shortcutStr: string | undefined): string[] {
  if (!shortcutStr) return ['None'];
  const parts = shortcutStr.split('+').map((p) => p.trim());
  return parts.map((part) => {
    switch (part) {
      case 'ArrowUp':
        return '↑';
      case 'ArrowDown':
        return '↓';
      case 'ArrowLeft':
        return '←';
      case 'ArrowRight':
        return '→';
      case 'Escape':
        return 'Esc';
      case 'Enter':
        return '↵ Enter';
      case 'Delete':
        return 'Del';
      default:
        return part;
    }
  });
}

/**
 * Detects conflicts where multiple actions are assigned the exact same shortcut.
 * Returns a map of normalizedShortcut -> array of action labels.
 */
export function findShortcutConflicts(settings: AppSettings): Map<string, string[]> {
  const conflicts = new Map<string, string[]>();
  const map = new Map<string, string[]>();

  for (const def of SHORTCUT_DEFINITIONS) {
    // Skip global shortcut from in-app conflicts because global handles window visibility
    if (def.isGlobal) continue;

    const val = (settings[def.id] as string) || def.defaultKey;
    const parsed = parseShortcutString(val);
    if (!parsed) continue;

    const norm = `${parsed.ctrl ? 'Ctrl+' : ''}${parsed.alt ? 'Alt+' : ''}${parsed.shift ? 'Shift+' : ''}${parsed.meta ? 'Super+' : ''}${parsed.key.toUpperCase()}`;
    const existing = map.get(norm) || [];
    existing.push(def.label);
    map.set(norm, existing);
  }

  for (const [key, labels] of map.entries()) {
    if (labels.length > 1) {
      conflicts.set(key, labels);
    }
  }

  return conflicts;
}
