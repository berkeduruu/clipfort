<script lang="ts">
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
  import { invoke } from '@tauri-apps/api/core';
  import type { FilterCategory, AppMode } from '$lib/types';
  import { hexToRgba } from '$lib/theme';

  const appWindow = typeof window !== 'undefined' ? getCurrentWebviewWindow() : null;

  function handleStartDrag(e: MouseEvent) {
    if (e.button !== 0) return;
    if ((e.target as HTMLElement).closest('button, input, select')) return;

    if (appWindow) {
      invoke('notify_user_dragged').catch(() => {});
      appWindow.startDragging().catch((err) => console.error('Start dragging error:', err));
    }
  }

  let {
    appMode = $bindable<AppMode>('clipboard'),
    searchQuery = $bindable(''),
    activeCategory = $bindable<FilterCategory>('all'),
    itemCount = 0,
    currentTheme,
    onClearAll,
    onOpenSettings,
    onCloseWindow,
    onOpenExportModal,
  }: {
    appMode: AppMode;
    searchQuery: string;
    activeCategory: FilterCategory;
    itemCount: number;
    currentTheme?: any;
    onClearAll: () => void;
    onOpenSettings: () => void;
    onCloseWindow: () => void;
    onOpenExportModal?: () => void;
  } = $props();

  const categories: { id: FilterCategory; label: string; icon: string }[] = [
    { id: 'all', label: 'All', icon: 'M4 6h16M4 12h16M4 18h16' },
    { id: 'text', label: 'Text', icon: 'M4 6h16M4 12h10M4 18h7' },
    { id: 'image', label: 'Images', icon: 'M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z' },
    { id: 'pinned', label: 'Pinned', icon: 'M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z' },
  ];
</script>

<div
  class="px-4 pt-3 pb-2.5 transition-colors duration-200 select-none border-b relative z-30"
  style="
    background-color: {currentTheme
      ? hexToRgba(currentTheme.cardColor, 80)
      : '#f8fafc'};
    border-color: {currentTheme
      ? hexToRgba(currentTheme.borderColor, 70)
      : '#e2e8f0'};
    color: {currentTheme
      ? currentTheme.textColor
      : '#1e293b'};
  "
>
  <!-- Top Bar: Mode Switcher, Draggable Region & Global Actions -->
  <div class="flex items-center justify-between mb-2">
    <!-- App Brand & Segmented Mode Switcher -->
    <div class="flex items-center space-x-2">
      <img src="/logo.svg" alt="ClipFort" class="w-5 h-5 object-contain select-none pointer-events-none shrink-0" title="ClipFort" />
      <div
        class="flex items-center space-x-1 p-0.5 rounded-lg text-xs font-semibold"
      style="
        background-color: {currentTheme
          ? hexToRgba(currentTheme.textColor, 8)
          : 'rgba(226, 232, 240, 0.8)'};
      "
    >
      <button
        onclick={() => (appMode = 'clipboard')}
        class="flex items-center space-x-1.5 px-3 py-1 rounded-md transition-all duration-150 cursor-pointer"
        style="
          background-color: {appMode === 'clipboard'
            ? (currentTheme ? currentTheme.accentColor : '#3b82f6')
            : 'transparent'};
          color: {appMode === 'clipboard'
            ? (currentTheme ? currentTheme.accentTextColor : '#ffffff')
            : (currentTheme ? currentTheme.secondaryTextColor : '#64748b')};
        "
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
        </svg>
        <span>Clipboard</span>
      </button>

      <button
        onclick={() => (appMode = 'vault')}
        class="flex items-center space-x-1.5 px-3 py-1 rounded-md transition-all duration-150 cursor-pointer"
        style="
          background-color: {appMode === 'vault'
            ? (currentTheme ? currentTheme.accentColor : '#f59e0b')
            : 'transparent'};
          color: {appMode === 'vault'
            ? (currentTheme ? currentTheme.accentTextColor : '#ffffff')
            : (currentTheme ? currentTheme.secondaryTextColor : '#64748b')};
        "
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
        </svg>
        <span>Secure Vault</span>
      </button>
    </div>
  </div>

    <!-- Drag Region (Window Moving Handle - Highlighted Area from User Screenshot) -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      data-tauri-drag-region
      onmousedown={handleStartDrag}
      class="flex-1 h-8 mx-2 rounded-xl cursor-grab active:cursor-grabbing flex items-center justify-center group/drag transition-all hover:bg-black/5 active:bg-black/10"
      title="Click and drag to move window"
    >
      <div
        data-tauri-drag-region
        class="w-20 h-2 rounded-full transition-all duration-150 flex items-center justify-center space-x-1.5 pointer-events-none group-hover/drag:h-2.5"
        style="background-color: {currentTheme ? hexToRgba(currentTheme.secondaryTextColor, 35) : '#cbd5e1'};"
      >
        <span class="w-1 h-1 rounded-full bg-white/40"></span>
        <span class="w-1 h-1 rounded-full bg-white/40"></span>
        <span class="w-1 h-1 rounded-full bg-white/40"></span>
      </div>
    </div>

    <!-- Actions: Settings & Close -->
    <div class="flex items-center space-x-1">
      <button
        onclick={onOpenSettings}
        class="p-1.5 rounded-lg transition-colors hover:opacity-80"
        style="color: {currentTheme ? currentTheme.secondaryTextColor : '#64748b'};"
        title="Settings"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
          <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
      </button>

      <button
        onclick={onCloseWindow}
        class="p-1.5 rounded-lg transition-colors ml-0.5 hover:opacity-80"
        style="color: {currentTheme ? currentTheme.secondaryTextColor : '#64748b'};"
        title="Hide (Esc)"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>
  </div>

  {#if appMode === 'clipboard'}
    <!-- Search input bar -->
    <div class="relative flex items-center mb-2.5">
      <div class="absolute left-3.5 pointer-events-none flex items-center" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
        </svg>
      </div>

      <input
        id="search-input"
        type="text"
        bind:value={searchQuery}
        placeholder="Search clipboard history... (↑↓ navigate, 1-9 to select)"
        onkeydown={(e) => {
          if (e.key === 'ArrowDown' || e.key === 'ArrowUp' || e.key === 'Enter') {
            e.preventDefault();
          }
        }}
        class="w-full pl-10 pr-20 py-2 text-sm rounded-xl border shadow-xs focus:outline-none transition-all duration-150"
        style="
          background-color: {currentTheme ? hexToRgba(currentTheme.cardColor, 90) : '#ffffff'};
          color: {currentTheme ? currentTheme.textColor : '#0f172a'};
          border-color: {currentTheme ? hexToRgba(currentTheme.borderColor, 80) : '#e2e8f0'};
          --placeholder-color: {currentTheme ? hexToRgba(currentTheme.secondaryTextColor, 70) : '#94a3b8'};
        "
        autocomplete="off"
        spellcheck="false"
      />

      <!-- Quick clear search & total count badge -->
      <div class="absolute right-2.5 flex items-center space-x-1.5 text-xs">
        {#if searchQuery}
          <button
            onclick={() => (searchQuery = '')}
            class="p-1 rounded-md transition-colors cursor-pointer hover:opacity-80"
            style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};"
            title="Clear search"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        {/if}
        <span
          class="px-1.5 py-0.5 rounded font-medium text-[11px]"
          style="
            background-color: {currentTheme ? hexToRgba(currentTheme.secondaryTextColor, 15) : '#f1f5f9'};
            color: {currentTheme ? currentTheme.secondaryTextColor : '#64748b'};
          "
        >
          {itemCount}
        </span>
      </div>
    </div>

    <!-- Filter tabs and action buttons -->
    <div class="flex items-center justify-between">
      <!-- Categories -->
      <div
        class="flex items-center space-x-1 p-0.5 rounded-lg text-xs font-medium"
        style="background-color: {currentTheme ? hexToRgba(currentTheme.textColor, 8) : 'rgba(226, 232, 240, 0.6)'};"
      >
        {#each categories as cat}
          <button
            onclick={() => (activeCategory = cat.id)}
            class="flex items-center space-x-1.5 px-2.5 py-1 rounded-md transition-all duration-150 cursor-pointer"
            style="
              background-color: {activeCategory === cat.id
                ? (currentTheme ? currentTheme.accentColor : '#ffffff')
                : 'transparent'};
              color: {activeCategory === cat.id
                ? (currentTheme ? currentTheme.accentTextColor : '#ffffff')
                : (currentTheme ? currentTheme.secondaryTextColor : '#64748b')};
            "
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d={cat.icon} />
            </svg>
            <span>{cat.label}</span>
          </button>
        {/each}
      </div>

      <!-- Action: Export & Clear Unpinned -->
      <div class="flex items-center space-x-1">
        <!-- Export Button -->
        <button
          onclick={() => onOpenExportModal?.()}
          class="p-1.5 hover:bg-black/5 dark:hover:bg-white/10 rounded-lg transition-colors flex items-center space-x-1 text-xs cursor-pointer"
          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#64748b'};"
          title="Export clipboard history (.md / .txt)"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
          </svg>
          <span class="text-[11px]">Export</span>
        </button>

        <button
          onclick={onClearAll}
          class="p-1.5 hover:text-rose-500 hover:bg-rose-500/10 rounded-lg transition-colors flex items-center space-x-1 text-xs cursor-pointer"
          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#64748b'};"
          title="Clear unpinned items"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
          <span class="text-[11px]">Clear</span>
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  #search-input::placeholder {
    color: var(--placeholder-color, #94a3b8);
  }
</style>
