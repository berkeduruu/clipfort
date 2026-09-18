<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import type { AppSettings, VaultStatus, ThemePreset } from '$lib/types';
  import { THEME_PRESETS, getEffectiveTheme, hexToRgba } from '$lib/theme';
  import {
    SHORTCUT_DEFINITIONS,
    DEFAULT_SHORTCUTS,
    parseKeyboardEvent,
    parseShortcutString,
    getShortcutDisplayParts,
    findShortcutConflicts,
    type ShortcutDefinition,
  } from '$lib/shortcuts';

  let {
    isOpen = false,
    settings,
    onClose,
    onSave,
  }: {
    isOpen: boolean;
    settings: AppSettings;
    onClose: () => void;
    onSave: (newSettings: AppSettings) => void;
  } = $props();

  let activeTab = $state<'general' | 'shortcuts' | 'appearance'>('general');

  let tempSettings = $state<AppSettings>({
    global_shortcut: 'Alt+Shift+Q',
    max_history: 100,
    close_on_copy: true,
    number_keys_copy: true,
    play_sound: false,
    run_at_startup: false,
    max_vault_file_size_mb: 20,
    window_width: 640,
    window_height: 560,
    theme_preset: 'light',
    custom_bg_color: '#f8fafc',
    custom_card_color: '#ffffff',
    custom_accent_color: '#10b981',
    custom_text_color: '#0f172a',
    custom_secondary_text_color: '#64748b',
    custom_border_color: '#e2e8f0',
    bg_opacity: 95,
    bg_image: null,
    bg_image_opacity: 25,
    bg_blur: 'sm',
    ...DEFAULT_SHORTCUTS,
  });

  let recordingShortcutId = $state<keyof AppSettings | null>(null);
  const conflicts = $derived(findShortcutConflicts(tempSettings));
  let vaultStatus = $state<VaultStatus | null>(null);
  let currentWindowWidth = $state(640);
  let currentWindowHeight = $state(560);
  let sizeSaveFeedback = $state('');

  async function updateCurrentWindowSize() {
    try {
      const win = getCurrentWindow();
      const factor = await win.scaleFactor();
      const size = await win.innerSize();
      currentWindowWidth = Math.round(size.width / factor);
      currentWindowHeight = Math.round(size.height / factor);
    } catch {
      currentWindowWidth = window.innerWidth;
      currentWindowHeight = window.innerHeight;
    }
  }

  async function handleSaveCurrentSizeAsDefault() {
    try {
      const [w, h] = await invoke<[number, number]>('save_current_window_size');
      currentWindowWidth = Math.round(w);
      currentWindowHeight = Math.round(h);
      tempSettings.window_width = currentWindowWidth;
      tempSettings.window_height = currentWindowHeight;
      sizeSaveFeedback = `Saved as default (${currentWindowWidth}×${currentWindowHeight})`;
      setTimeout(() => {
        sizeSaveFeedback = '';
      }, 2500);
    } catch (err) {
      console.error('Failed to save window size:', err);
    }
  }

  async function handleResetDefaultSize() {
    try {
      const [w, h] = await invoke<[number, number]>('apply_window_size', {
        width: 640.0,
        height: 560.0,
        saveAsDefault: true,
      });
      currentWindowWidth = Math.round(w);
      currentWindowHeight = Math.round(h);
      tempSettings.window_width = currentWindowWidth;
      tempSettings.window_height = currentWindowHeight;
      sizeSaveFeedback = 'Reset to default (640×560)';
      setTimeout(() => {
        sizeSaveFeedback = '';
      }, 2500);
    } catch (err) {
      console.error('Failed to reset window size:', err);
    }
  }

  function selectThemePreset(presetKey: ThemePreset) {
    const preset = THEME_PRESETS[presetKey];
    if (!preset) return;
    tempSettings.theme_preset = presetKey;
    tempSettings.custom_bg_color = preset.bg_color;
    tempSettings.custom_card_color = preset.card_color;
    tempSettings.custom_accent_color = preset.accent_color;
    tempSettings.custom_text_color = preset.text_color;
    tempSettings.custom_secondary_text_color = preset.secondary_text_color;
    tempSettings.custom_border_color = preset.border_color;
  }

  let modalTheme = $derived(getEffectiveTheme(tempSettings));

  function handleColorChange() {
    tempSettings.theme_preset = 'custom';
  }

  function handleBgImageUpload(e: Event) {
    const target = e.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;

    if (file.size > 5 * 1024 * 1024) {
      alert("Please select an image smaller than 5 MB.");
      return;
    }

    const reader = new FileReader();
    reader.onload = () => {
      if (typeof reader.result === 'string') {
        tempSettings.bg_image = reader.result;
      }
    };
    reader.readAsDataURL(file);
  }

  function handleRemoveBgImage() {
    tempSettings.bg_image = null;
  }

  $effect(() => {
    if (isOpen) {
      tempSettings = {
        theme_preset: 'light',
        custom_bg_color: '#f8fafc',
        custom_card_color: '#ffffff',
        custom_accent_color: '#10b981',
        custom_text_color: '#0f172a',
        custom_secondary_text_color: '#64748b',
        custom_border_color: '#e2e8f0',
        bg_opacity: 95,
        bg_image: null,
        bg_image_opacity: 25,
        bg_blur: 'sm',
        ...DEFAULT_SHORTCUTS,
        ...settings,
      };
      recordingShortcutId = null;
      sizeSaveFeedback = '';
      updateCurrentWindowSize();

      invoke<VaultStatus>('get_vault_status')
        .then((s) => {
          vaultStatus = s;
        })
        .catch(() => {});

      const handleResize = () => {
        updateCurrentWindowSize();
      };
      window.addEventListener('resize', handleResize);
      return () => {
        window.removeEventListener('resize', handleResize);
      };
    }
  });

  function handleSave() {
    onSave({ ...tempSettings });
    onClose();
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (!recordingShortcutId) return;
    e.preventDefault();
    e.stopPropagation();

    const isGlobal = recordingShortcutId === 'global_shortcut';
    const parsed = parseKeyboardEvent(e, isGlobal);
    if (parsed) {
      (tempSettings[recordingShortcutId] as any) = parsed;
      recordingShortcutId = null;
    }
  }

  $effect(() => {
    if (recordingShortcutId) {
      const onKey = (e: KeyboardEvent) => {
        handleKeyDown(e);
      };
      window.addEventListener('keydown', onKey, true);
      return () => {
        window.removeEventListener('keydown', onKey, true);
      };
    }
  });

  function handleResetAllShortcuts() {
    for (const def of SHORTCUT_DEFINITIONS) {
      (tempSettings[def.id] as any) = def.defaultKey;
    }
  }
</script>

{#if isOpen}
  <!-- Backdrop -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 bg-slate-900/40 backdrop-blur-sm z-50 flex items-center justify-center p-3 animate-in fade-in duration-100"
    onclick={onClose}
  >
    <!-- Compact Modal Card -->
    <div
      class="w-[420px] max-h-[88vh] flex flex-col rounded-2xl shadow-2xl overflow-hidden text-xs transition-colors duration-150"
      style="
        background-color: {modalTheme.cardColor};
        border: 1px solid {modalTheme.borderColor};
        color: {modalTheme.textColor};
      "
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div
        class="px-4 py-2.5 border-b flex items-center justify-between shrink-0"
        style="
          background-color: {hexToRgba(modalTheme.bgColor, 80)};
          border-color: {hexToRgba(modalTheme.borderColor, 70)};
        "
      >
        <h3 class="font-semibold flex items-center gap-2 text-xs" style="color: {modalTheme.textColor};">
          <img src="/logo.svg" alt="ClipFort" class="w-4 h-4 object-contain select-none pointer-events-none shrink-0" />
          ClipFort Settings
        </h3>
        <button
          onclick={onClose}
          aria-label="Close"
          class="p-0.5 rounded transition-colors cursor-pointer hover:opacity-80"
          style="color: {modalTheme.secondaryTextColor};"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Tab Navigation -->
      <div
        class="flex items-center border-b px-4 pt-1 gap-3 text-xs shrink-0 select-none overflow-x-auto"
        style="
          background-color: {hexToRgba(modalTheme.bgColor, 50)};
          border-color: {hexToRgba(modalTheme.borderColor, 70)};
        "
      >
        <button
          type="button"
          onclick={() => (activeTab = 'general')}
          class="pb-2 px-1 font-medium border-b-2 transition-all cursor-pointer flex items-center gap-1.5 shrink-0"
          style="
            border-color: {activeTab === 'general' ? modalTheme.accentColor : 'transparent'};
            color: {activeTab === 'general' ? modalTheme.accentColor : modalTheme.secondaryTextColor};
          "
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
          General
        </button>

        <button
          type="button"
          onclick={() => (activeTab = 'shortcuts')}
          class="pb-2 px-1 font-medium border-b-2 transition-all cursor-pointer flex items-center gap-1.5 shrink-0"
          style="
            border-color: {activeTab === 'shortcuts' ? modalTheme.accentColor : 'transparent'};
            color: {activeTab === 'shortcuts' ? modalTheme.accentColor : modalTheme.secondaryTextColor};
          "
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M3 10h18M3 14h18m-9-4v8m-4-4v4m8-4v4M4 6h16a2 2 0 012 2v8a2 2 0 01-2 2H4a2 2 0 01-2-2V8a2 2 0 012-2z" />
          </svg>
          Shortcuts
          {#if conflicts.size > 0}
            <span class="w-1.5 h-1.5 rounded-full bg-amber-500"></span>
          {/if}
        </button>

        <button
          type="button"
          onclick={() => (activeTab = 'appearance')}
          class="pb-2 px-1 font-medium border-b-2 transition-all cursor-pointer flex items-center gap-1.5 shrink-0"
          style="
            border-color: {activeTab === 'appearance' ? modalTheme.accentColor : 'transparent'};
            color: {activeTab === 'appearance' ? modalTheme.accentColor : modalTheme.secondaryTextColor};
          "
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M7 21a4 4 0 01-4-4 5 5 0 015-5h1a4 4 0 014 4v1a4 4 0 01-4 4H7zm11-14a3 3 0 11-6 0 3 3 0 016 0zM17 11h2a2 2 0 012 2v1a2 2 0 01-2 2h-2" />
          </svg>
          Appearance & Themes
        </button>
      </div>

      <!-- Settings Content -->
      <div class="p-4 space-y-3.5 overflow-y-auto flex-1">
        {#if activeTab === 'general'}
          <!-- Shortcut Setting -->
          <div class="flex items-center justify-between">
            <div>
              <label class="font-medium" style="color: {modalTheme.textColor};" for="shortcut-btn">
                Global Shortcut:
              </label>
              <p class="text-[10px]" style="color: {modalTheme.secondaryTextColor};">Toggle window from anywhere</p>
            </div>
            <div class="flex items-center gap-2">
              <button
                id="shortcut-btn"
                onclick={() => (recordingShortcutId = recordingShortcutId === 'global_shortcut' ? null : 'global_shortcut')}
                class="px-2.5 py-1 rounded-lg border font-mono text-[11px] font-semibold transition-all cursor-pointer flex items-center gap-1 {recordingShortcutId === 'global_shortcut'
                  ? 'bg-rose-500/15 border-rose-500 text-rose-500 animate-pulse ring-1 ring-rose-500'
                  : 'hover:opacity-85'}"
                style="{recordingShortcutId !== 'global_shortcut' ? `background-color: ${hexToRgba(modalTheme.bgColor, 80)}; border-color: ${hexToRgba(modalTheme.borderColor, 80)}; color: ${modalTheme.textColor};` : ''}"
                title="Click to record a new global shortcut"
              >
                {#if recordingShortcutId === 'global_shortcut'}
                  <span>Press keys...</span>
                {:else}
                  {#each getShortcutDisplayParts(tempSettings.global_shortcut) as part, idx}
                    {#if idx > 0}<span class="text-[9px] opacity-40 font-sans">+</span>{/if}
                    <kbd
                      class="px-1.5 py-0.5 rounded text-[10px] font-semibold border shadow-2xs"
                      style="
                        background-color: {hexToRgba(modalTheme.cardColor, 90)};
                        border-color: {hexToRgba(modalTheme.borderColor, 90)};
                        color: {modalTheme.textColor};
                      "
                    >{part}</kbd>
                  {/each}
                {/if}
              </button>
              <button
                type="button"
                onclick={() => (activeTab = 'shortcuts')}
                class="text-[10px] font-medium underline transition-opacity hover:opacity-80 cursor-pointer"
                style="color: {modalTheme.accentColor};"
              >
                All Shortcuts →
              </button>
            </div>
          </div>

          <!-- Max History Limit -->
          <div>
            <div class="flex items-center justify-between mb-1">
              <span class="font-medium" style="color: {modalTheme.textColor};">History Limit:</span>
              <span class="font-semibold" style="color: {modalTheme.accentColor};">{tempSettings.max_history} items</span>
            </div>
            <input
              id="max-history"
              type="range"
              min="20"
              max="200"
              step="10"
              bind:value={tempSettings.max_history}
              class="w-full h-1.5 rounded-lg cursor-pointer"
              style="accent-color: {modalTheme.accentColor};"
            />
          </div>

          <!-- Max Vault File Size (MB) -->
          <div>
            <div class="flex items-center justify-between mb-1">
              <span class="font-medium" style="color: {modalTheme.textColor};">Vault Max File Size:</span>
              <span class="font-semibold" style="color: {modalTheme.accentColor};">{tempSettings.max_vault_file_size_mb || 20} MB</span>
            </div>
            <input
              id="max-vault-file-size"
              type="range"
              min="5"
              max="100"
              step="5"
              bind:value={tempSettings.max_vault_file_size_mb}
              class="w-full h-1.5 rounded-lg cursor-pointer"
              style="accent-color: {modalTheme.accentColor};"
            />
          </div>

          <!-- Window Size Setting -->
          <div
            class="rounded-xl p-2.5 space-y-2 border"
            style="
              background-color: {hexToRgba(modalTheme.bgColor, 55)};
              border-color: {hexToRgba(modalTheme.borderColor, 75)};
            "
          >
            <div class="flex items-center justify-between">
              <span class="font-medium flex items-center gap-1.5" style="color: {modalTheme.textColor};">
                <svg class="w-3.5 h-3.5" style="color: {modalTheme.secondaryTextColor};" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
                </svg>
                Window Dimensions:
              </span>
              <span
                class="text-[11px] font-mono font-semibold px-2 py-0.5 rounded border shadow-2xs"
                style="
                  background-color: {hexToRgba(modalTheme.cardColor, 90)};
                  border-color: {hexToRgba(modalTheme.borderColor, 75)};
                  color: {modalTheme.textColor};
                "
              >
                {currentWindowWidth} × {currentWindowHeight} px
              </span>
            </div>

            <p class="text-[10px] leading-tight" style="color: {modalTheme.secondaryTextColor};">
              Drag borders or corners to resize window, then click below to save as startup default.
            </p>

            {#if sizeSaveFeedback}
              <div
                class="text-[11px] font-medium rounded-md px-2 py-1 flex items-center gap-1.5 animate-in fade-in border"
                style="
                  background-color: {hexToRgba(modalTheme.accentColor, 15)};
                  border-color: {hexToRgba(modalTheme.accentColor, 35)};
                  color: {modalTheme.accentColor};
                "
              >
                <svg class="w-3.5 h-3.5 shrink-0" style="color: {modalTheme.accentColor};" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
                <span>{sizeSaveFeedback}</span>
              </div>
            {/if}

            <div class="flex items-center gap-1.5 pt-0.5">
              <button
                type="button"
                onclick={handleSaveCurrentSizeAsDefault}
                class="flex-1 py-1.5 px-2 rounded-lg text-[11px] font-semibold flex items-center justify-center gap-1.5 transition-colors shadow-2xs cursor-pointer border"
                style="
                  background-color: {hexToRgba(modalTheme.accentColor, 18)};
                  border-color: {hexToRgba(modalTheme.accentColor, 40)};
                  color: {modalTheme.accentColor};
                "
                title="Saves current size as startup default"
              >
                <svg class="w-3.5 h-3.5" style="color: {modalTheme.accentColor};" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
                </svg>
                <span>Set Current as Default</span>
              </button>
              <button
                type="button"
                onclick={handleResetDefaultSize}
                class="py-1.5 px-2.5 rounded-lg text-[11px] font-medium transition-colors cursor-pointer border hover:opacity-80"
                style="
                  background-color: {hexToRgba(modalTheme.bgColor, 70)};
                  border-color: {hexToRgba(modalTheme.borderColor, 75)};
                  color: {modalTheme.textColor};
                "
                title="Resets to default size (640×560)"
              >
                ↺ Reset
              </button>
            </div>
          </div>

          <!-- Vault Encryption Mode Info -->
          <div
            class="rounded-xl p-2.5 space-y-1 border"
            style="
              background-color: {hexToRgba(modalTheme.bgColor, 55)};
              border-color: {hexToRgba(modalTheme.borderColor, 75)};
            "
          >
            <div class="flex items-center justify-between">
              <span class="font-medium" style="color: {modalTheme.textColor};">Vault Security Mode:</span>
              {#if vaultStatus?.has_pin}
                <span class="text-[11px] font-semibold text-amber-500 flex items-center gap-1">🔒 PIN Protected</span>
              {:else if vaultStatus?.is_initialized}
                <span class="text-[11px] font-semibold flex items-center gap-1" style="color: {modalTheme.accentColor};">🛡️ Device-Bound Encryption</span>
              {:else}
                <span class="text-[11px]" style="color: {modalTheme.secondaryTextColor};">Not Initialized</span>
              {/if}
            </div>
            <p class="text-[10px] leading-tight" style="color: {modalTheme.secondaryTextColor};">
              {#if vaultStatus?.has_pin}
                Master PIN is required to unlock sessions.
              {:else if vaultStatus?.is_initialized}
                Encrypted on disk via AES-256-GCM, unlocks instantly on this machine.
              {:else}
                You can initialize the vault from the Secure Vault tab.
              {/if}
            </p>
          </div>

          <div class="border-t pt-2 space-y-2" style="border-color: {hexToRgba(modalTheme.borderColor, 60)};">
            <!-- Close on copy checkbox -->
            <label class="flex items-center justify-between cursor-pointer select-none">
              <span style="color: {modalTheme.textColor};">Hide window after copying</span>
              <input
                type="checkbox"
                bind:checked={tempSettings.close_on_copy}
                class="w-3.5 h-3.5 rounded cursor-pointer"
                style="accent-color: {modalTheme.accentColor};"
              />
            </label>

            <!-- 1-9 Quick slot copy checkbox -->
            <label class="flex items-center justify-between cursor-pointer select-none">
              <span style="color: {modalTheme.textColor};">Quick copy with number keys (1-9)</span>
              <input
                type="checkbox"
                bind:checked={tempSettings.number_keys_copy}
                class="w-3.5 h-3.5 rounded cursor-pointer"
                style="accent-color: {modalTheme.accentColor};"
              />
            </label>

            <!-- Autostart on boot checkbox -->
            <label class="flex items-center justify-between cursor-pointer select-none">
              <span style="color: {modalTheme.textColor};">Launch automatically on system boot</span>
              <input
                type="checkbox"
                bind:checked={tempSettings.run_at_startup}
                class="w-3.5 h-3.5 rounded cursor-pointer"
                style="accent-color: {modalTheme.accentColor};"
              />
            </label>

            <!-- Reset Window Position button -->
            <div class="pt-1">
              <button
                type="button"
                onclick={() => invoke('reset_window_position')}
                class="w-full py-1.5 px-2 rounded-lg text-xs font-medium flex items-center justify-center gap-1.5 transition-colors cursor-pointer border hover:opacity-85"
                style="
                  background-color: {hexToRgba(modalTheme.bgColor, 70)};
                  border-color: {hexToRgba(modalTheme.borderColor, 75)};
                  color: {modalTheme.textColor};
                "
                title="Positions window at bottom-right corner"
              >
                <svg class="w-3.5 h-3.5" style="color: {modalTheme.secondaryTextColor};" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
                </svg>
                <span>Reset Position to Bottom-Right</span>
              </button>
            </div>

            <!-- Keyboard Shortcuts Reference / Link to Shortcuts tab -->
            <div class="pt-2 border-t" style="border-color: {hexToRgba(modalTheme.borderColor, 60)};">
              <div class="flex items-center justify-between mb-1.5">
                <span class="text-[11px] font-semibold flex items-center gap-1" style="color: {modalTheme.textColor};">
                  ⌨️ Keyboard Shortcuts
                </span>
                <button
                  type="button"
                  onclick={() => (activeTab = 'shortcuts')}
                  class="text-[10px] font-semibold underline cursor-pointer hover:opacity-85 transition-opacity"
                  style="color: {modalTheme.accentColor};"
                >
                  Customize All →
                </button>
              </div>
              <div
                class="grid grid-cols-2 gap-1.5 text-[10px] p-2 rounded-xl border cursor-pointer hover:opacity-90 transition-opacity"
                onclick={() => (activeTab = 'shortcuts')}
                style="
                  background-color: {hexToRgba(modalTheme.bgColor, 50)};
                  border-color: {hexToRgba(modalTheme.borderColor, 70)};
                  color: {modalTheme.secondaryTextColor};
                "
              >
                <div class="flex items-center justify-between"><kbd class="px-1 py-0.5 border rounded font-mono" style="background-color: {hexToRgba(modalTheme.cardColor, 90)}; border-color: {hexToRgba(modalTheme.borderColor, 75)}; color: {modalTheme.textColor};">↑ / ↓</kbd> <span>Navigate</span></div>
                <div class="flex items-center justify-between"><kbd class="px-1 py-0.5 border rounded font-mono" style="background-color: {hexToRgba(modalTheme.cardColor, 90)}; border-color: {hexToRgba(modalTheme.borderColor, 75)}; color: {modalTheme.textColor};">{tempSettings.shortcut_copy || 'Enter'}</kbd> <span>Copy</span></div>
                <div class="flex items-center justify-between"><kbd class="px-1 py-0.5 border rounded font-mono" style="background-color: {hexToRgba(modalTheme.cardColor, 90)}; border-color: {hexToRgba(modalTheme.borderColor, 75)}; color: {modalTheme.textColor};">1-9</kbd> <span>Quick Select</span></div>
                <div class="flex items-center justify-between"><kbd class="px-1 py-0.5 border rounded font-mono" style="background-color: {hexToRgba(modalTheme.cardColor, 90)}; border-color: {hexToRgba(modalTheme.borderColor, 75)}; color: {modalTheme.textColor};">{tempSettings.shortcut_pin || 'P'}</kbd> <span>Pin / Unpin</span></div>
                <div class="flex items-center justify-between"><kbd class="px-1 py-0.5 border rounded font-mono" style="background-color: {hexToRgba(modalTheme.cardColor, 90)}; border-color: {hexToRgba(modalTheme.borderColor, 75)}; color: {modalTheme.textColor};">{tempSettings.shortcut_delete || 'Del'}</kbd> <span>Delete</span></div>
                <div class="flex items-center justify-between"><kbd class="px-1 py-0.5 border rounded font-mono" style="background-color: {hexToRgba(modalTheme.cardColor, 90)}; border-color: {hexToRgba(modalTheme.borderColor, 75)}; color: {modalTheme.textColor};">{tempSettings.shortcut_search || 'Ctrl+F'}</kbd> <span>Search</span></div>
              </div>
            </div>
          </div>
        {:else if activeTab === 'shortcuts'}
          <!-- KEYBOARD SHORTCUTS TAB -->
          <div class="space-y-4">
            <!-- Header bar with Reset All button -->
            <div
              class="flex items-center justify-between pb-2 border-b"
              style="border-color: {hexToRgba(modalTheme.borderColor, 70)};"
            >
              <div>
                <p class="font-medium text-xs" style="color: {modalTheme.textColor};">Customize Keybindings</p>
                <p class="text-[10px]" style="color: {modalTheme.secondaryTextColor};">Click any key badge to record a new key combination</p>
              </div>
              <button
                type="button"
                onclick={handleResetAllShortcuts}
                class="text-[10px] font-medium px-2 py-1 rounded-lg border transition-all hover:opacity-80 active:scale-95 cursor-pointer shrink-0"
                style="
                  background-color: {hexToRgba(modalTheme.bgColor, 60)};
                  border-color: {hexToRgba(modalTheme.borderColor, 80)};
                  color: {modalTheme.secondaryTextColor};
                "
              >
                Reset All to Defaults
              </button>
            </div>

            <!-- Conflicts Alert Banner -->
            {#if conflicts.size > 0}
              <div class="p-2.5 rounded-xl border bg-amber-500/10 border-amber-500/30 text-amber-500 text-[11px] flex items-start gap-2">
                <svg class="w-4 h-4 shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                </svg>
                <div class="min-w-0">
                  <p class="font-semibold text-xs">Conflict Detected</p>
                  <p class="text-[10px] opacity-90 mt-0.5">The following shortcuts are assigned to multiple actions:</p>
                  <ul class="mt-1 space-y-0.5 text-[10px]">
                    {#each Array.from(conflicts.entries()) as [key, labels]}
                      <li>• <span class="font-mono font-bold">{key}</span>: {labels.join(' & ')}</li>
                    {/each}
                  </ul>
                </div>
              </div>
            {/if}

            <!-- 1. Global Group -->
            <div class="space-y-2">
              <h4 class="text-[10px] font-bold uppercase tracking-wider flex items-center gap-1" style="color: {modalTheme.accentColor};">
                <span>🌐</span> Global (System-Wide)
              </h4>
              {#each SHORTCUT_DEFINITIONS.filter(d => d.category === 'global') as def}
                {@render shortcutRow(def)}
              {/each}
            </div>

            <!-- 2. Clipboard Actions Group -->
            <div class="space-y-2">
              <h4 class="text-[10px] font-bold uppercase tracking-wider flex items-center gap-1" style="color: {modalTheme.accentColor};">
                <span>📋</span> Clipboard Actions
              </h4>
              {#each SHORTCUT_DEFINITIONS.filter(d => d.category === 'actions') as def}
                {@render shortcutRow(def)}
              {/each}
            </div>

            <!-- 3. Navigation & Modes Group -->
            <div class="space-y-2">
              <h4 class="text-[10px] font-bold uppercase tracking-wider flex items-center gap-1" style="color: {modalTheme.accentColor};">
                <span>🪟</span> Window & Navigation
              </h4>
              {#each SHORTCUT_DEFINITIONS.filter(d => d.category === 'navigation') as def}
                {@render shortcutRow(def)}
              {/each}
            </div>
          </div>
        {:else if activeTab === 'appearance'}
          <!-- APPEARANCE & THEME TAB -->

          <!-- 1. Theme Presets -->
          <div>
            <div class="flex items-center justify-between mb-2">
              <span class="font-medium flex items-center gap-1.5" style="color: {modalTheme.textColor};">
                <span>🎨</span> Theme Presets
              </span>
              {#if tempSettings.theme_preset === 'custom'}
                <span class="text-[10px] px-1.5 py-0.5 rounded font-semibold border" style="background-color: {hexToRgba(modalTheme.accentColor, 20)}; color: {modalTheme.accentColor}; border-color: {hexToRgba(modalTheme.accentColor, 40)};">Custom</span>
              {/if}
            </div>

            <div class="grid grid-cols-2 gap-2">
              {#each Object.values(THEME_PRESETS).filter(p => p.id !== 'custom') as preset}
                <button
                  type="button"
                  onclick={() => selectThemePreset(preset.id)}
                  class="p-2 rounded-xl border text-left flex items-center justify-between transition-all cursor-pointer shadow-2xs"
                  style="
                    background-color: {tempSettings.theme_preset === preset.id
                      ? hexToRgba(modalTheme.accentColor, 18)
                      : hexToRgba(modalTheme.bgColor, 60)};
                    border-color: {tempSettings.theme_preset === preset.id
                      ? modalTheme.accentColor
                      : hexToRgba(modalTheme.borderColor, 75)};
                  "
                >
                  <div class="flex items-center gap-2">
                    <!-- Dots preview -->
                    <div class="flex -space-x-1.5 items-center">
                      <span class="w-3.5 h-3.5 rounded-full border border-black/20 shadow-2xs" style="background-color: {preset.bg_color}"></span>
                      <span class="w-3.5 h-3.5 rounded-full border border-black/20 shadow-2xs" style="background-color: {preset.card_color}"></span>
                      <span class="w-3.5 h-3.5 rounded-full border border-black/20 shadow-2xs" style="background-color: {preset.accent_color}"></span>
                    </div>
                    <span class="font-medium text-[11px] truncate" style="color: {modalTheme.textColor};">{preset.name}</span>
                  </div>
                  {#if tempSettings.theme_preset === preset.id}
                    <svg class="w-3.5 h-3.5 shrink-0" style="color: {modalTheme.accentColor};" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2.5">
                      <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                    </svg>
                  {/if}
                </button>
              {/each}
            </div>
          </div>

          <!-- 2. Individual Color Pickers -->
          <div
            class="rounded-xl p-2.5 space-y-2.5 border"
            style="
              background-color: {hexToRgba(modalTheme.bgColor, 55)};
              border-color: {hexToRgba(modalTheme.borderColor, 75)};
            "
          >
            <span class="font-medium flex items-center gap-1.5" style="color: {modalTheme.textColor};">
              <span>🖌️</span> Customize Colors
            </span>

            <div class="space-y-2">
              <!-- Background Color -->
              <div class="flex items-center justify-between">
                <span class="text-[11px] flex items-center gap-1.5" style="color: {modalTheme.textColor};">
                  <span class="w-2.5 h-2.5 rounded-full border border-black/20" style="background-color: {tempSettings.custom_bg_color}"></span>
                  Background Color:
                </span>
                <div class="flex items-center gap-1.5">
                  <input
                    type="color"
                    bind:value={tempSettings.custom_bg_color}
                    oninput={handleColorChange}
                    class="w-6 h-6 rounded border cursor-pointer bg-transparent p-0"
                    style="border-color: {hexToRgba(modalTheme.borderColor, 80)};"
                  />
                  <input
                    type="text"
                    bind:value={tempSettings.custom_bg_color}
                    oninput={handleColorChange}
                    class="w-16 px-1.5 py-0.5 text-[11px] font-mono border rounded uppercase"
                    style="
                      background-color: {hexToRgba(modalTheme.cardColor, 90)};
                      color: {modalTheme.textColor};
                      border-color: {hexToRgba(modalTheme.borderColor, 75)};
                    "
                  />
                </div>
              </div>

              <!-- Card Color -->
              <div class="flex items-center justify-between">
                <span class="text-[11px] flex items-center gap-1.5" style="color: {modalTheme.textColor};">
                  <span class="w-2.5 h-2.5 rounded-full border border-black/20" style="background-color: {tempSettings.custom_card_color}"></span>
                  Card / Panel Color:
                </span>
                <div class="flex items-center gap-1.5">
                  <input
                    type="color"
                    bind:value={tempSettings.custom_card_color}
                    oninput={handleColorChange}
                    class="w-6 h-6 rounded border cursor-pointer bg-transparent p-0"
                    style="border-color: {hexToRgba(modalTheme.borderColor, 80)};"
                  />
                  <input
                    type="text"
                    bind:value={tempSettings.custom_card_color}
                    oninput={handleColorChange}
                    class="w-16 px-1.5 py-0.5 text-[11px] font-mono border rounded uppercase"
                    style="
                      background-color: {hexToRgba(modalTheme.cardColor, 90)};
                      color: {modalTheme.textColor};
                      border-color: {hexToRgba(modalTheme.borderColor, 75)};
                    "
                  />
                </div>
              </div>

              <!-- Accent Color -->
              <div class="flex items-center justify-between">
                <span class="text-[11px] flex items-center gap-1.5" style="color: {modalTheme.textColor};">
                  <span class="w-2.5 h-2.5 rounded-full border border-black/20" style="background-color: {tempSettings.custom_accent_color}"></span>
                  Accent (Button) Color:
                </span>
                <div class="flex items-center gap-1.5">
                  <input
                    type="color"
                    bind:value={tempSettings.custom_accent_color}
                    oninput={handleColorChange}
                    class="w-6 h-6 rounded border cursor-pointer bg-transparent p-0"
                    style="border-color: {hexToRgba(modalTheme.borderColor, 80)};"
                  />
                  <input
                    type="text"
                    bind:value={tempSettings.custom_accent_color}
                    oninput={handleColorChange}
                    class="w-16 px-1.5 py-0.5 text-[11px] font-mono border rounded uppercase"
                    style="
                      background-color: {hexToRgba(modalTheme.cardColor, 90)};
                      color: {modalTheme.textColor};
                      border-color: {hexToRgba(modalTheme.borderColor, 75)};
                    "
                  />
                </div>
              </div>

              <!-- Text Color -->
              <div class="flex items-center justify-between">
                <span class="text-[11px] flex items-center gap-1.5" style="color: {modalTheme.textColor};">
                  <span class="w-2.5 h-2.5 rounded-full border border-black/20" style="background-color: {tempSettings.custom_text_color}"></span>
                  Text Color:
                </span>
                <div class="flex items-center gap-1.5">
                  <input
                    type="color"
                    bind:value={tempSettings.custom_text_color}
                    oninput={handleColorChange}
                    class="w-6 h-6 rounded border cursor-pointer bg-transparent p-0"
                    style="border-color: {hexToRgba(modalTheme.borderColor, 80)};"
                  />
                  <input
                    type="text"
                    bind:value={tempSettings.custom_text_color}
                    oninput={handleColorChange}
                    class="w-16 px-1.5 py-0.5 text-[11px] font-mono border rounded uppercase"
                    style="
                      background-color: {hexToRgba(modalTheme.cardColor, 90)};
                      color: {modalTheme.textColor};
                      border-color: {hexToRgba(modalTheme.borderColor, 75)};
                    "
                  />
                </div>
              </div>

              <!-- Secondary / Muted Text & Icon Color -->
              <div class="flex items-center justify-between">
                <span class="text-[11px] flex items-center gap-1.5" style="color: {modalTheme.textColor};">
                  <span class="w-2.5 h-2.5 rounded-full border border-black/20" style="background-color: {tempSettings.custom_secondary_text_color || '#64748b'}"></span>
                  Secondary Text & Icons:
                </span>
                <div class="flex items-center gap-1.5">
                  <input
                    type="color"
                    bind:value={tempSettings.custom_secondary_text_color}
                    oninput={handleColorChange}
                    class="w-6 h-6 rounded border cursor-pointer bg-transparent p-0"
                    style="border-color: {hexToRgba(modalTheme.borderColor, 80)};"
                  />
                  <input
                    type="text"
                    bind:value={tempSettings.custom_secondary_text_color}
                    oninput={handleColorChange}
                    class="w-16 px-1.5 py-0.5 text-[11px] font-mono border rounded uppercase"
                    style="
                      background-color: {hexToRgba(modalTheme.cardColor, 90)};
                      color: {modalTheme.textColor};
                      border-color: {hexToRgba(modalTheme.borderColor, 75)};
                    "
                  />
                </div>
              </div>

              <!-- Border Color -->
              <div class="flex items-center justify-between">
                <span class="text-[11px] flex items-center gap-1.5" style="color: {modalTheme.textColor};">
                  <span class="w-2.5 h-2.5 rounded-full border border-black/20" style="background-color: {tempSettings.custom_border_color}"></span>
                  Border Color:
                </span>
                <div class="flex items-center gap-1.5">
                  <input
                    type="color"
                    bind:value={tempSettings.custom_border_color}
                    oninput={handleColorChange}
                    class="w-6 h-6 rounded border cursor-pointer bg-transparent p-0"
                    style="border-color: {hexToRgba(modalTheme.borderColor, 80)};"
                  />
                  <input
                    type="text"
                    bind:value={tempSettings.custom_border_color}
                    oninput={handleColorChange}
                    class="w-16 px-1.5 py-0.5 text-[11px] font-mono border rounded uppercase"
                    style="
                      background-color: {hexToRgba(modalTheme.cardColor, 90)};
                      color: {modalTheme.textColor};
                      border-color: {hexToRgba(modalTheme.borderColor, 75)};
                    "
                  />
                </div>
              </div>
            </div>
          </div>

          <!-- 3. Background Image -->
          <div
            class="rounded-xl p-2.5 space-y-2 border"
            style="
              background-color: {hexToRgba(modalTheme.bgColor, 55)};
              border-color: {hexToRgba(modalTheme.borderColor, 75)};
            "
          >
            <span class="font-medium flex items-center gap-1.5" style="color: {modalTheme.textColor};">
              <span>🖼️</span> Background Wallpaper
            </span>

            <div class="flex items-center gap-3">
              {#if tempSettings.bg_image}
                <div class="relative w-14 h-12 rounded-lg border overflow-hidden shrink-0 shadow-2xs" style="border-color: {hexToRgba(modalTheme.borderColor, 80)};">
                  <img
                    src={tempSettings.bg_image}
                    alt="Background preview"
                    class="w-full h-full object-cover"
                  />
                </div>
                <div class="flex-1 flex flex-col gap-1">
                  <span class="text-[11px] font-medium" style="color: {modalTheme.accentColor};">Image Loaded</span>
                  <button
                    type="button"
                    onclick={handleRemoveBgImage}
                    class="py-1 px-2 text-[10px] font-medium text-rose-500 hover:text-rose-400 rounded-md transition-colors self-start cursor-pointer border"
                    style="background-color: {hexToRgba(modalTheme.bgColor, 70)}; border-color: rgba(244, 63, 94, 0.4);"
                  >
                    Remove Image
                  </button>
                </div>
              {:else}
                <div class="flex-1">
                  <p class="text-[10px] mb-1.5" style="color: {modalTheme.secondaryTextColor};">
                    Select a custom background image to blend with window transparency.
                  </p>
                  <label
                    for="bg-file-upload"
                    class="inline-flex items-center gap-1.5 py-1.5 px-3 rounded-lg text-[11px] font-medium cursor-pointer transition-colors shadow-2xs border hover:opacity-85"
                    style="
                      background-color: {hexToRgba(modalTheme.cardColor, 90)};
                      border-color: {hexToRgba(modalTheme.borderColor, 75)};
                      color: {modalTheme.textColor};
                    "
                  >
                    <svg class="w-3.5 h-3.5" style="color: {modalTheme.secondaryTextColor};" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                      <path stroke-linecap="round" stroke-linejoin="round" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                    </svg>
                    <span>Choose Image (JPG, PNG)</span>
                  </label>
                  <input
                    id="bg-file-upload"
                    type="file"
                    accept="image/*"
                    onchange={handleBgImageUpload}
                    class="hidden"
                  />
                </div>
              {/if}
            </div>
          </div>

          <!-- 4. Opacity & Blur Controls -->
          <div
            class="rounded-xl p-2.5 space-y-3 border"
            style="
              background-color: {hexToRgba(modalTheme.bgColor, 55)};
              border-color: {hexToRgba(modalTheme.borderColor, 75)};
            "
          >
            <span class="font-medium flex items-center gap-1.5" style="color: {modalTheme.textColor};">
              <span>🎚️</span> Opacity & Glass Effect
            </span>

            <!-- Background Opacity -->
            <div>
              <div class="flex items-center justify-between mb-1">
                <span class="text-[11px]" style="color: {modalTheme.textColor};">Background Opacity:</span>
                <span class="font-semibold font-mono text-[11px]" style="color: {modalTheme.accentColor};">{tempSettings.bg_opacity ?? 95}%</span>
              </div>
              <input
                id="bg-opacity"
                type="range"
                min="20"
                max="100"
                step="5"
                bind:value={tempSettings.bg_opacity}
                class="w-full h-1.5 rounded-lg cursor-pointer"
                style="accent-color: {modalTheme.accentColor};"
              />
            </div>

            <!-- Background Image Opacity -->
            {#if tempSettings.bg_image}
              <div>
                <div class="flex items-center justify-between mb-1">
                  <span class="text-[11px]" style="color: {modalTheme.textColor};">Wallpaper Visibility:</span>
                  <span class="font-semibold font-mono text-[11px]" style="color: {modalTheme.accentColor};">{tempSettings.bg_image_opacity ?? 25}%</span>
                </div>
                <input
                  id="bg-img-opacity"
                  type="range"
                  min="5"
                  max="100"
                  step="5"
                  bind:value={tempSettings.bg_image_opacity}
                  class="w-full h-1.5 rounded-lg cursor-pointer"
                  style="accent-color: {modalTheme.accentColor};"
                />
              </div>
            {/if}

            <!-- Blur Effect -->
            <div>
              <span class="text-[11px] block mb-1.5" style="color: {modalTheme.textColor};">Blur Intensity (Glassmorphism):</span>
              <div class="grid grid-cols-4 gap-1.5">
                {#each [
                  { id: 'none', label: 'None' },
                  { id: 'sm', label: 'Light' },
                  { id: 'md', label: 'Medium' },
                  { id: 'lg', label: 'Strong' },
                ] as blurOpt}
                  <button
                    type="button"
                    onclick={() => (tempSettings.bg_blur = blurOpt.id)}
                    class="py-1 text-center rounded-lg text-[10px] font-medium transition-all cursor-pointer border"
                    style="
                      background-color: {tempSettings.bg_blur === blurOpt.id
                        ? modalTheme.accentColor
                        : hexToRgba(modalTheme.cardColor, 90)};
                      color: {tempSettings.bg_blur === blurOpt.id
                        ? modalTheme.accentTextColor
                        : modalTheme.secondaryTextColor};
                      border-color: {tempSettings.bg_blur === blurOpt.id
                        ? modalTheme.accentColor
                        : hexToRgba(modalTheme.borderColor, 75)};
                    "
                  >
                    {blurOpt.label}
                  </button>
                {/each}
              </div>
            </div>
          </div>
        {/if}
      </div>

      {#snippet shortcutRow(def: ShortcutDefinition)}
        {@const currentVal = (tempSettings[def.id] as string) || def.defaultKey}
        {@const isRecording = recordingShortcutId === def.id}
        {@const parsed = parseShortcutString(currentVal)}
        {@const normKey = parsed ? `${parsed.ctrl ? 'Ctrl+' : ''}${parsed.alt ? 'Alt+' : ''}${parsed.shift ? 'Shift+' : ''}${parsed.meta ? 'Super+' : ''}${parsed.key.toUpperCase()}` : ''}
        {@const hasConflict = !def.isGlobal && normKey ? conflicts.has(normKey) : false}
        <div
          class="flex items-center justify-between p-2 rounded-xl border transition-all"
          style="
            background-color: {hexToRgba(modalTheme.bgColor, 45)};
            border-color: {isRecording ? modalTheme.accentColor : hasConflict ? '#f59e0b' : hexToRgba(modalTheme.borderColor, 60)};
          "
        >
          <div class="flex-1 min-w-0 pr-2">
            <div class="flex items-center gap-1.5">
              <span class="font-medium text-xs truncate" style="color: {modalTheme.textColor};">
                {def.label}
              </span>
              {#if hasConflict}
                <span class="text-[9px] px-1.5 py-0.2 rounded font-semibold bg-amber-500/20 text-amber-500 border border-amber-500/30">
                  Conflict
                </span>
              {/if}
            </div>
            <p class="text-[10px] truncate mt-0.5" style="color: {modalTheme.secondaryTextColor};">
              {def.description}
            </p>
          </div>

          <div class="flex items-center gap-1.5 shrink-0">
            <button
              type="button"
              onclick={() => (recordingShortcutId = isRecording ? null : def.id)}
              class="px-2 py-1 rounded-lg border font-mono text-[11px] font-semibold transition-all cursor-pointer flex items-center gap-1 select-none {isRecording
                ? 'bg-rose-500/15 border-rose-500 text-rose-500 animate-pulse ring-1 ring-rose-500'
                : 'hover:opacity-85'}"
              style="{!isRecording ? `background-color: ${hexToRgba(modalTheme.bgColor, 80)}; border-color: ${hexToRgba(modalTheme.borderColor, 80)}; color: ${modalTheme.textColor};` : ''}"
              title={isRecording ? 'Press your key combination or click to cancel' : 'Click to record new shortcut'}
            >
              {#if isRecording}
                <span class="animate-pulse">Press keys...</span>
              {:else}
                {#each getShortcutDisplayParts(currentVal) as part, idx}
                  {#if idx > 0}<span class="text-[9px] opacity-40 font-sans">+</span>{/if}
                  <kbd
                    class="px-1.5 py-0.5 rounded text-[10px] font-semibold border shadow-2xs"
                    style="
                      background-color: {hexToRgba(modalTheme.cardColor, 90)};
                      border-color: {hexToRgba(modalTheme.borderColor, 90)};
                      color: {modalTheme.textColor};
                    "
                  >{part}</kbd>
                {/each}
              {/if}
            </button>

            {#if currentVal !== def.defaultKey}
              <button
                type="button"
                onclick={() => { (tempSettings[def.id] as any) = def.defaultKey; }}
                class="p-1 rounded-lg border hover:opacity-80 transition-all cursor-pointer"
                style="
                  background-color: {hexToRgba(modalTheme.bgColor, 80)};
                  border-color: {hexToRgba(modalTheme.borderColor, 80)};
                  color: {modalTheme.secondaryTextColor};
                "
                title="Reset to default ({def.defaultKey})"
              >
                <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                </svg>
              </button>
            {/if}
          </div>
        </div>
      {/snippet}

      <!-- Footer Buttons -->
      <div
        class="px-4 py-2.5 border-t flex items-center justify-end space-x-2 shrink-0"
        style="
          background-color: {hexToRgba(modalTheme.bgColor, 80)};
          border-color: {hexToRgba(modalTheme.borderColor, 70)};
        "
      >
        <button
          onclick={onClose}
          class="px-3 py-1.5 text-xs font-medium rounded-lg cursor-pointer hover:opacity-80 transition-opacity"
          style="color: {modalTheme.secondaryTextColor};"
        >
          Cancel
        </button>
        <button
          onclick={handleSave}
          class="px-4 py-1.5 text-xs font-semibold rounded-lg shadow-sm transition-all active:scale-[0.98] cursor-pointer flex items-center gap-1.5"
          style="
            background-color: {modalTheme.accentColor};
            color: {modalTheme.accentTextColor};
          "
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2.5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
          </svg>
          <span>Save Settings</span>
        </button>
      </div>
    </div>
  </div>
{/if}
