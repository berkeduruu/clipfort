<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import type { AppSettings, VaultStatus, ThemePreset } from '$lib/types';
  import { THEME_PRESETS } from '$lib/theme';

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

  let activeTab = $state<'general' | 'appearance'>('general');

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
  });

  let isRecordingShortcut = $state(false);
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
        ...settings,
      };
      isRecordingShortcut = false;
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
    if (!isRecordingShortcut) return;
    e.preventDefault();
    e.stopPropagation();

    const parts: string[] = [];
    if (e.ctrlKey) parts.push('Ctrl');
    if (e.altKey) parts.push('Alt');
    if (e.shiftKey) parts.push('Shift');
    if (e.metaKey) parts.push('Super');

    const key = e.key.toUpperCase();
    if (!['CONTROL', 'ALT', 'SHIFT', 'META'].includes(key)) {
      parts.push(key);
      tempSettings.global_shortcut = parts.join('+');
      isRecordingShortcut = false;
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
      class="w-[360px] max-h-[88vh] flex flex-col bg-white rounded-2xl shadow-2xl border border-slate-200 overflow-hidden text-xs"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="px-4 py-2.5 border-b border-slate-100 flex items-center justify-between bg-slate-50 shrink-0">
        <h3 class="font-semibold text-slate-800 flex items-center gap-2 text-xs">
          <img src="/logo.svg" alt="ClipFort" class="w-4 h-4 object-contain select-none pointer-events-none shrink-0" />
          ClipFort Settings
        </h3>
        <button
          onclick={onClose}
          aria-label="Close"
          class="text-slate-400 hover:text-slate-600 p-0.5 rounded hover:bg-slate-200 transition-colors cursor-pointer"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Tab Navigation -->
      <div class="flex items-center border-b border-slate-200 bg-slate-50 px-4 pt-1 gap-4 text-xs shrink-0 select-none">
        <button
          type="button"
          onclick={() => (activeTab = 'general')}
          class="pb-2 px-1 font-medium border-b-2 transition-all cursor-pointer flex items-center gap-1.5 {activeTab === 'general'
            ? 'border-emerald-600 text-emerald-700 font-semibold'
            : 'border-transparent text-slate-500 hover:text-slate-800'}"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
          General
        </button>

        <button
          type="button"
          onclick={() => (activeTab = 'appearance')}
          class="pb-2 px-1 font-medium border-b-2 transition-all cursor-pointer flex items-center gap-1.5 {activeTab === 'appearance'
            ? 'border-emerald-600 text-emerald-700 font-semibold'
            : 'border-transparent text-slate-500 hover:text-slate-800'}"
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
            <label class="font-medium text-slate-700" for="shortcut-btn">
              Global Shortcut:
            </label>
            <button
              id="shortcut-btn"
              onclick={() => (isRecordingShortcut = true)}
              onkeydown={handleKeyDown}
              class="px-2.5 py-1 rounded-md border font-mono text-[11px] font-semibold transition-all cursor-pointer {isRecordingShortcut
                ? 'bg-rose-50 border-rose-300 text-rose-600 animate-pulse ring-1 ring-rose-400'
                : 'bg-slate-50 border-slate-200 text-slate-700 hover:bg-slate-100'}"
              title="Click to record a new global shortcut"
            >
              {isRecordingShortcut ? 'Press keys...' : tempSettings.global_shortcut}
            </button>
          </div>

          <!-- Max History Limit -->
          <div>
            <div class="flex items-center justify-between mb-1">
              <span class="font-medium text-slate-700">History Limit:</span>
              <span class="text-emerald-600 font-semibold">{tempSettings.max_history} items</span>
            </div>
            <input
              id="max-history"
              type="range"
              min="20"
              max="200"
              step="10"
              bind:value={tempSettings.max_history}
              class="w-full h-1.5 bg-slate-200 rounded-lg accent-emerald-600 cursor-pointer"
            />
          </div>

          <!-- Max Vault File Size (MB) -->
          <div>
            <div class="flex items-center justify-between mb-1">
              <span class="font-medium text-slate-700">Vault Max File Size:</span>
              <span class="text-emerald-600 font-semibold">{tempSettings.max_vault_file_size_mb || 20} MB</span>
            </div>
            <input
              id="max-vault-file-size"
              type="range"
              min="5"
              max="100"
              step="5"
              bind:value={tempSettings.max_vault_file_size_mb}
              class="w-full h-1.5 bg-slate-200 rounded-lg accent-emerald-600 cursor-pointer"
            />
          </div>

          <!-- Window Size Setting -->
          <div class="bg-slate-50 border border-slate-200 rounded-lg p-2.5 space-y-2">
            <div class="flex items-center justify-between">
              <span class="font-medium text-slate-700 flex items-center gap-1.5">
                <svg class="w-3.5 h-3.5 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
                </svg>
                Window Dimensions:
              </span>
              <span class="text-[11px] font-mono font-semibold px-2 py-0.5 rounded bg-white border border-slate-200 text-slate-700 shadow-2xs">
                {currentWindowWidth} × {currentWindowHeight} px
              </span>
            </div>

            <p class="text-[10px] text-slate-500 leading-tight">
              Drag borders or corners to resize window, then click below to save as startup default.
            </p>

            {#if sizeSaveFeedback}
              <div class="text-[11px] font-medium text-emerald-700 bg-emerald-50 border border-emerald-200 rounded-md px-2 py-1 flex items-center gap-1.5 animate-in fade-in">
                <svg class="w-3.5 h-3.5 text-emerald-600 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
                <span>{sizeSaveFeedback}</span>
              </div>
            {/if}

            <div class="flex items-center gap-1.5 pt-0.5">
              <button
                type="button"
                onclick={handleSaveCurrentSizeAsDefault}
                class="flex-1 py-1.5 px-2 bg-emerald-50 hover:bg-emerald-100 active:bg-emerald-200 text-emerald-700 border border-emerald-300 rounded-lg text-[11px] font-semibold flex items-center justify-center gap-1.5 transition-colors shadow-2xs cursor-pointer"
                title="Saves current size as startup default"
              >
                <svg class="w-3.5 h-3.5 text-emerald-600" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
                </svg>
                <span>Set Current as Default</span>
              </button>
              <button
                type="button"
                onclick={handleResetDefaultSize}
                class="py-1.5 px-2.5 bg-slate-100 hover:bg-slate-200 active:bg-slate-300 text-slate-600 border border-slate-200 rounded-lg text-[11px] font-medium transition-colors cursor-pointer"
                title="Resets to default size (640×560)"
              >
                ↺ Reset
              </button>
            </div>
          </div>

          <!-- Vault Encryption Mode Info -->
          <div class="bg-slate-50 border border-slate-200 rounded-lg p-2.5 space-y-1">
            <div class="flex items-center justify-between">
              <span class="font-medium text-slate-700">Vault Security Mode:</span>
              {#if vaultStatus?.has_pin}
                <span class="text-[11px] font-semibold text-amber-600 flex items-center gap-1">🔒 PIN Protected</span>
              {:else if vaultStatus?.is_initialized}
                <span class="text-[11px] font-semibold text-emerald-600 flex items-center gap-1">🛡️ Device-Bound Encryption</span>
              {:else}
                <span class="text-[11px] text-slate-400">Not Initialized</span>
              {/if}
            </div>
            <p class="text-[10px] text-slate-500 leading-tight">
              {#if vaultStatus?.has_pin}
                Master PIN is required to unlock sessions.
              {:else if vaultStatus?.is_initialized}
                Encrypted on disk via AES-256-GCM, unlocks instantly on this machine.
              {:else}
                You can initialize the vault from the Secure Vault tab.
              {/if}
            </p>
          </div>

          <div class="border-t border-slate-100 pt-2 space-y-2">
            <!-- Close on copy checkbox -->
            <label class="flex items-center justify-between cursor-pointer select-none">
              <span class="text-slate-700">Hide window after copying</span>
              <input
                type="checkbox"
                bind:checked={tempSettings.close_on_copy}
                class="w-3.5 h-3.5 accent-emerald-600 rounded cursor-pointer"
              />
            </label>

            <!-- 1-9 Quick slot copy checkbox -->
            <label class="flex items-center justify-between cursor-pointer select-none">
              <span class="text-slate-700">Quick copy with number keys (1-9)</span>
              <input
                type="checkbox"
                bind:checked={tempSettings.number_keys_copy}
                class="w-3.5 h-3.5 accent-emerald-600 rounded cursor-pointer"
              />
            </label>

            <!-- Autostart on boot checkbox -->
            <label class="flex items-center justify-between cursor-pointer select-none">
              <span class="text-slate-700">Launch automatically on system boot</span>
              <input
                type="checkbox"
                bind:checked={tempSettings.run_at_startup}
                class="w-3.5 h-3.5 accent-emerald-600 rounded cursor-pointer"
              />
            </label>

            <!-- Reset Window Position button -->
            <div class="pt-1">
              <button
                type="button"
                onclick={() => invoke('reset_window_position')}
                class="w-full py-1.5 px-2 bg-slate-100 hover:bg-slate-200 active:bg-slate-300 text-slate-600 hover:text-slate-800 rounded-lg text-xs font-medium flex items-center justify-center gap-1.5 transition-colors cursor-pointer"
                title="Positions window at bottom-right corner"
              >
                <svg class="w-3.5 h-3.5 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
                </svg>
                <span>Reset Position to Bottom-Right</span>
              </button>
            </div>

            <!-- Keyboard Shortcuts Reference -->
            <div class="pt-2 border-t border-slate-100">
              <span class="text-[11px] font-semibold text-slate-700 block mb-1.5">⌨️ Keyboard Shortcuts Reference</span>
              <div class="grid grid-cols-2 gap-1.5 text-[10px] text-slate-600 bg-slate-50 p-2 rounded-xl border border-slate-200/70">
                <div class="flex items-center justify-between"><kbd class="px-1 py-0.5 bg-white border border-slate-200 rounded font-mono">↑ / ↓</kbd> <span>Navigate</span></div>
                <div class="flex items-center justify-between"><kbd class="px-1 py-0.5 bg-white border border-slate-200 rounded font-mono">Enter</kbd> <span>Copy</span></div>
                <div class="flex items-center justify-between"><kbd class="px-1 py-0.5 bg-white border border-slate-200 rounded font-mono">1-9</kbd> <span>Quick Select</span></div>
                <div class="flex items-center justify-between"><kbd class="px-1 py-0.5 bg-white border border-slate-200 rounded font-mono">Alt+↑/↓</kbd> <span>Reorder</span></div>
                <div class="flex items-center justify-between"><kbd class="px-1 py-0.5 bg-white border border-slate-200 rounded font-mono">P</kbd> <span>Pin / Unpin</span></div>
                <div class="flex items-center justify-between"><kbd class="px-1 py-0.5 bg-white border border-slate-200 rounded font-mono">Del</kbd> <span>Delete</span></div>
              </div>
            </div>
          </div>
        {:else if activeTab === 'appearance'}
          <!-- APPEARANCE & THEME TAB -->

          <!-- 1. Theme Presets -->
          <div>
            <div class="flex items-center justify-between mb-2">
              <span class="font-medium text-slate-800 flex items-center gap-1.5">
                <span>🎨</span> Theme Presets
              </span>
              {#if tempSettings.theme_preset === 'custom'}
                <span class="text-[10px] px-1.5 py-0.5 bg-amber-50 text-amber-600 border border-amber-200 rounded font-semibold">Custom</span>
              {/if}
            </div>

            <div class="grid grid-cols-2 gap-2">
              {#each Object.values(THEME_PRESETS).filter(p => p.id !== 'custom') as preset}
                <button
                  type="button"
                  onclick={() => selectThemePreset(preset.id)}
                  class="p-2 rounded-xl border text-left flex items-center justify-between transition-all cursor-pointer {tempSettings.theme_preset === preset.id
                    ? 'border-emerald-500 bg-emerald-50/50 shadow-2xs ring-1 ring-emerald-400/30'
                    : 'border-slate-200 bg-slate-50 hover:bg-slate-100/80'}"
                >
                  <div class="flex items-center gap-2">
                    <!-- Dots preview -->
                    <div class="flex -space-x-1.5 items-center">
                      <span class="w-3.5 h-3.5 rounded-full border border-black/10 shadow-2xs" style="background-color: {preset.bg_color}"></span>
                      <span class="w-3.5 h-3.5 rounded-full border border-black/10 shadow-2xs" style="background-color: {preset.card_color}"></span>
                      <span class="w-3.5 h-3.5 rounded-full border border-black/10 shadow-2xs" style="background-color: {preset.accent_color}"></span>
                    </div>
                    <span class="font-medium text-[11px] text-slate-700">{preset.name}</span>
                  </div>
                  {#if tempSettings.theme_preset === preset.id}
                    <svg class="w-3.5 h-3.5 text-emerald-600 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2.5">
                      <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                    </svg>
                  {/if}
                </button>
              {/each}
            </div>
          </div>

          <!-- 2. Individual Color Pickers -->
          <div class="bg-slate-50 border border-slate-200 rounded-xl p-2.5 space-y-2.5">
            <span class="font-medium text-slate-800 flex items-center gap-1.5">
              <span>🖌️</span> Customize Colors
            </span>

            <div class="space-y-2">
              <!-- Background Color -->
              <div class="flex items-center justify-between">
                <span class="text-slate-600 text-[11px] flex items-center gap-1.5">
                  <span class="w-2.5 h-2.5 rounded-full" style="background-color: {tempSettings.custom_bg_color}"></span>
                  Background Color:
                </span>
                <div class="flex items-center gap-1.5">
                  <input
                    type="color"
                    bind:value={tempSettings.custom_bg_color}
                    oninput={handleColorChange}
                    class="w-6 h-6 rounded border border-slate-300 cursor-pointer bg-transparent p-0"
                  />
                  <input
                    type="text"
                    bind:value={tempSettings.custom_bg_color}
                    oninput={handleColorChange}
                    class="w-16 px-1.5 py-0.5 text-[11px] font-mono border border-slate-200 rounded bg-white text-slate-700 uppercase"
                  />
                </div>
              </div>

              <!-- Card Color -->
              <div class="flex items-center justify-between">
                <span class="text-slate-600 text-[11px] flex items-center gap-1.5">
                  <span class="w-2.5 h-2.5 rounded-full" style="background-color: {tempSettings.custom_card_color}"></span>
                  Card / Panel Color:
                </span>
                <div class="flex items-center gap-1.5">
                  <input
                    type="color"
                    bind:value={tempSettings.custom_card_color}
                    oninput={handleColorChange}
                    class="w-6 h-6 rounded border border-slate-300 cursor-pointer bg-transparent p-0"
                  />
                  <input
                    type="text"
                    bind:value={tempSettings.custom_card_color}
                    oninput={handleColorChange}
                    class="w-16 px-1.5 py-0.5 text-[11px] font-mono border border-slate-200 rounded bg-white text-slate-700 uppercase"
                  />
                </div>
              </div>

              <!-- Accent Color -->
              <div class="flex items-center justify-between">
                <span class="text-slate-600 text-[11px] flex items-center gap-1.5">
                  <span class="w-2.5 h-2.5 rounded-full" style="background-color: {tempSettings.custom_accent_color}"></span>
                  Accent (Button) Color:
                </span>
                <div class="flex items-center gap-1.5">
                  <input
                    type="color"
                    bind:value={tempSettings.custom_accent_color}
                    oninput={handleColorChange}
                    class="w-6 h-6 rounded border border-slate-300 cursor-pointer bg-transparent p-0"
                  />
                  <input
                    type="text"
                    bind:value={tempSettings.custom_accent_color}
                    oninput={handleColorChange}
                    class="w-16 px-1.5 py-0.5 text-[11px] font-mono border border-slate-200 rounded bg-white text-slate-700 uppercase"
                  />
                </div>
              </div>

              <!-- Text Color -->
              <div class="flex items-center justify-between">
                <span class="text-slate-600 text-[11px] flex items-center gap-1.5">
                  <span class="w-2.5 h-2.5 rounded-full" style="background-color: {tempSettings.custom_text_color}"></span>
                  Text Color:
                </span>
                <div class="flex items-center gap-1.5">
                  <input
                    type="color"
                    bind:value={tempSettings.custom_text_color}
                    oninput={handleColorChange}
                    class="w-6 h-6 rounded border border-slate-300 cursor-pointer bg-transparent p-0"
                  />
                  <input
                    type="text"
                    bind:value={tempSettings.custom_text_color}
                    oninput={handleColorChange}
                    class="w-16 px-1.5 py-0.5 text-[11px] font-mono border border-slate-200 rounded bg-white text-slate-700 uppercase"
                  />
                </div>
              </div>

              <!-- Secondary / Muted Text & Icon Color -->
              <div class="flex items-center justify-between">
                <span class="text-slate-600 text-[11px] flex items-center gap-1.5">
                  <span class="w-2.5 h-2.5 rounded-full" style="background-color: {tempSettings.custom_secondary_text_color || '#64748b'}"></span>
                  Secondary Text & Icons:
                </span>
                <div class="flex items-center gap-1.5">
                  <input
                    type="color"
                    bind:value={tempSettings.custom_secondary_text_color}
                    oninput={handleColorChange}
                    class="w-6 h-6 rounded border border-slate-300 cursor-pointer bg-transparent p-0"
                  />
                  <input
                    type="text"
                    bind:value={tempSettings.custom_secondary_text_color}
                    oninput={handleColorChange}
                    class="w-16 px-1.5 py-0.5 text-[11px] font-mono border border-slate-200 rounded bg-white text-slate-700 uppercase"
                  />
                </div>
              </div>

              <!-- Border Color -->
              <div class="flex items-center justify-between">
                <span class="text-slate-600 text-[11px] flex items-center gap-1.5">
                  <span class="w-2.5 h-2.5 rounded-full" style="background-color: {tempSettings.custom_border_color}"></span>
                  Border Color:
                </span>
                <div class="flex items-center gap-1.5">
                  <input
                    type="color"
                    bind:value={tempSettings.custom_border_color}
                    oninput={handleColorChange}
                    class="w-6 h-6 rounded border border-slate-300 cursor-pointer bg-transparent p-0"
                  />
                  <input
                    type="text"
                    bind:value={tempSettings.custom_border_color}
                    oninput={handleColorChange}
                    class="w-16 px-1.5 py-0.5 text-[11px] font-mono border border-slate-200 rounded bg-white text-slate-700 uppercase"
                  />
                </div>
              </div>
            </div>
          </div>

          <!-- 3. Background Image -->
          <div class="bg-slate-50 border border-slate-200 rounded-xl p-2.5 space-y-2">
            <span class="font-medium text-slate-800 flex items-center gap-1.5">
              <span>🖼️</span> Background Wallpaper
            </span>

            <div class="flex items-center gap-3">
              {#if tempSettings.bg_image}
                <div class="relative w-14 h-12 rounded-lg border border-slate-300 overflow-hidden shrink-0 shadow-2xs">
                  <img
                    src={tempSettings.bg_image}
                    alt="Background preview"
                    class="w-full h-full object-cover"
                  />
                </div>
                <div class="flex-1 flex flex-col gap-1">
                  <span class="text-[11px] text-emerald-600 font-medium">Image Loaded</span>
                  <button
                    type="button"
                    onclick={handleRemoveBgImage}
                    class="py-1 px-2 text-[10px] font-medium text-rose-600 hover:text-rose-700 bg-rose-50 hover:bg-rose-100 border border-rose-200 rounded-md transition-colors self-start cursor-pointer"
                  >
                    Remove Image
                  </button>
                </div>
              {:else}
                <div class="flex-1">
                  <p class="text-[10px] text-slate-500 mb-1.5">
                    Select a custom background image to blend with window transparency.
                  </p>
                  <label
                    for="bg-file-upload"
                    class="inline-flex items-center gap-1.5 py-1.5 px-3 bg-white hover:bg-slate-100 border border-slate-200 rounded-lg text-[11px] font-medium text-slate-700 cursor-pointer transition-colors shadow-2xs"
                  >
                    <svg class="w-3.5 h-3.5 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
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
          <div class="bg-slate-50 border border-slate-200 rounded-xl p-2.5 space-y-3">
            <span class="font-medium text-slate-800 flex items-center gap-1.5">
              <span>🎚️</span> Opacity & Glass Effect
            </span>

            <!-- Background Opacity -->
            <div>
              <div class="flex items-center justify-between mb-1">
                <span class="text-slate-600 text-[11px]">Background Opacity:</span>
                <span class="text-emerald-600 font-semibold font-mono text-[11px]">{tempSettings.bg_opacity ?? 95}%</span>
              </div>
              <input
                id="bg-opacity"
                type="range"
                min="20"
                max="100"
                step="5"
                bind:value={tempSettings.bg_opacity}
                class="w-full h-1.5 bg-slate-200 rounded-lg accent-emerald-600 cursor-pointer"
              />
            </div>

            <!-- Background Image Opacity -->
            {#if tempSettings.bg_image}
              <div>
                <div class="flex items-center justify-between mb-1">
                  <span class="text-slate-600 text-[11px]">Wallpaper Visibility:</span>
                  <span class="text-emerald-600 font-semibold font-mono text-[11px]">{tempSettings.bg_image_opacity ?? 25}%</span>
                </div>
                <input
                  id="bg-img-opacity"
                  type="range"
                  min="5"
                  max="100"
                  step="5"
                  bind:value={tempSettings.bg_image_opacity}
                  class="w-full h-1.5 bg-slate-200 rounded-lg accent-emerald-600 cursor-pointer"
                />
              </div>
            {/if}

            <!-- Blur Effect -->
            <div>
              <span class="text-slate-600 text-[11px] block mb-1.5">Blur Intensity (Glassmorphism):</span>
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
                    class="py-1 text-center rounded-lg text-[10px] font-medium transition-all cursor-pointer {tempSettings.bg_blur === blurOpt.id
                      ? 'bg-emerald-600 text-white shadow-2xs font-semibold'
                      : 'bg-white border border-slate-200 text-slate-600 hover:bg-slate-100'}"
                  >
                    {blurOpt.label}
                  </button>
                {/each}
              </div>
            </div>
          </div>
        {/if}
      </div>

      <!-- Footer Buttons -->
      <div class="px-4 py-2.5 bg-slate-50 border-t border-slate-100 flex items-center justify-end space-x-2 shrink-0">
        <button
          onclick={onClose}
          class="px-3 py-1.5 text-slate-600 hover:text-slate-800 text-xs font-medium rounded-lg hover:bg-slate-200 transition-colors cursor-pointer"
        >
          Cancel
        </button>
        <button
          onclick={handleSave}
          class="px-4 py-1.5 bg-emerald-600 hover:bg-emerald-700 text-white text-xs font-semibold rounded-lg shadow-sm transition-colors cursor-pointer flex items-center gap-1.5"
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
