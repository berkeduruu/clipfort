<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import type { ClipItem, AppSettings, FilterCategory, AppMode } from '$lib/types';
  import Header from '$lib/components/Header.svelte';
  import ClipItemCard from '$lib/components/ClipItemCard.svelte';
  import SettingsModal from '$lib/components/SettingsModal.svelte';
  import VaultView from '$lib/components/VaultView.svelte';
  import ResizeHandles from '$lib/components/ResizeHandles.svelte';
  import { getEffectiveTheme, hexToRgba, getBlurClass } from '$lib/theme';

  let appMode = $state<AppMode>('clipboard');
  let clips = $state<ClipItem[]>([]);
  let searchQuery = $state('');
  let activeCategory = $state<FilterCategory>('all');
  let selectedIndex = $state(0);
  let isSettingsOpen = $state(false);
  let isExportModalOpen = $state(false);

  let settings = $state<AppSettings>({
    global_shortcut: 'Alt+Shift+Q',
    max_history: 100,
    close_on_copy: true,
    number_keys_copy: true,
    play_sound: false,
    run_at_startup: false,
    max_vault_file_size_mb: 20,
    window_width: 640,
    window_height: 560,
  });

  let currentTheme = $derived(getEffectiveTheme(settings));


  // Filter clips based on activeCategory and searchQuery
  let filteredClips = $derived(
    clips.filter((item) => {
      // Category filter
      if (activeCategory === 'text' && item.item_type !== 'text') return false;
      if (activeCategory === 'image' && item.item_type !== 'image') return false;
      if (activeCategory === 'pinned' && !item.pinned) return false;

      // Search query filter
      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase();
        if (item.item_type === 'text') {
          return item.content.toLowerCase().includes(q);
        } else {
          return 'görsel resim image'.includes(q);
        }
      }
      return true;
    })
  );

  // Keep selectedIndex valid when filteredClips changes
  $effect(() => {
    if (filteredClips.length === 0) {
      selectedIndex = 0;
    } else if (selectedIndex >= filteredClips.length) {
      selectedIndex = filteredClips.length - 1;
    }
  });

  async function loadClips() {
    try {
      clips = await invoke<ClipItem[]>('get_clips');
    } catch (e) {
      console.error('Failed to load clips:', e);
    }
  }

  async function loadSettings() {
    try {
      settings = await invoke<AppSettings>('get_settings');
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  }

  let exportToast = $state<string | null>(null);

  async function handleExportHistory(format: 'markdown' | 'text') {
    try {
      const savedPath = await invoke<string>('export_clipboard_history', { format });
      exportToast = `Geçmiş dışa aktarıldı:\n${savedPath}`;
      setTimeout(() => {
        exportToast = null;
      }, 4000);
    } catch (e: any) {
      alert(`Dışa aktarma hatası: ${e}`);
    }
  }

  async function copyClip(id: string) {
    try {
      await invoke('copy_clip_to_clipboard', { id });
      await loadClips();
      selectedIndex = 0;
      scrollToSelected(0);
    } catch (e) {
      console.error('Failed to copy clip:', e);
    } finally {
      if (settings.close_on_copy) {
        try {
          await invoke('hide_window');
        } catch (err) {
          console.error('Failed to hide window:', err);
        }
      }
    }
  }

  async function deleteClip(id: string) {
    try {
      await invoke('delete_clip', { id });
      await loadClips();
    } catch (e) {
      console.error('Failed to delete clip:', e);
    }
  }

  async function togglePin(id: string) {
    try {
      await invoke('toggle_pin_clip', { id });
      await loadClips();
    } catch (e) {
      console.error('Failed to toggle pin:', e);
    }
  }

  async function moveItem(fromFilteredIndex: number, direction: 'up' | 'down') {
    const toFilteredIndex = direction === 'up' ? fromFilteredIndex - 1 : fromFilteredIndex + 1;
    if (toFilteredIndex < 0 || toFilteredIndex >= filteredClips.length) return;

    const itemA = filteredClips[fromFilteredIndex];
    const itemB = filteredClips[toFilteredIndex];

    const idxA = clips.findIndex((it) => it.id === itemA.id);
    const idxB = clips.findIndex((it) => it.id === itemB.id);

    if (idxA !== -1 && idxB !== -1) {
      const newClips = [...clips];
      const temp = newClips[idxA];
      newClips[idxA] = newClips[idxB];
      newClips[idxB] = temp;

      clips = newClips;
      selectedIndex = toFilteredIndex;

      try {
        await invoke('reorder_clips', { ids: newClips.map((c) => c.id) });
      } catch (e) {
        console.error('Failed to save reordered clips:', e);
      }
    }
  }

  async function handleClearAll() {
    try {
      await invoke('clear_clips', { keepPinned: true });
      await loadClips();
    } catch (e) {
      console.error('Failed to clear clips:', e);
    }
  }

  async function handleSaveSettings(newSettings: AppSettings) {
    try {
      await invoke('save_settings', { settings: newSettings });
      settings = newSettings;
    } catch (e) {
      console.error('Failed to save settings:', e);
    }
  }

  function scrollToSelected(index: number) {
    tick().then(() => {
      const el = document.getElementById(`clip-item-${index}`);
      if (el) {
        el.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
      }
    });
  }

  function handleGlobalKeyDown(e: KeyboardEvent) {
    if (isSettingsOpen) {
      if (e.key === 'Escape') {
        isSettingsOpen = false;
        e.preventDefault();
      }
      return;
    }

    if (appMode === 'vault') {
      if (e.key === 'Escape') {
        e.preventDefault();
        invoke('hide_window');
      }
      return;
    }

    const isSearchFocused = document.activeElement?.id === 'search-input';

    // 1. Arrow Navigation - ALWAYS WORKS regardless of search focus
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (e.altKey) {
        moveItem(selectedIndex, 'down');
      } else {
        if (selectedIndex < filteredClips.length - 1) {
          selectedIndex++;
          scrollToSelected(selectedIndex);
        }
      }
      return;
    }

    if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (e.altKey) {
        moveItem(selectedIndex, 'up');
      } else {
        if (selectedIndex > 0) {
          selectedIndex--;
          scrollToSelected(selectedIndex);
        }
      }
      return;
    }

    // 2. Enter to copy selected and close window
    if (e.key === 'Enter') {
      if (filteredClips.length > 0 && selectedIndex < filteredClips.length) {
        e.preventDefault();
        copyClip(filteredClips[selectedIndex].id);
      }
      return;
    }

    // 3. Escape to close window
    if (e.key === 'Escape') {
      e.preventDefault();
      invoke('hide_window');
      return;
    }

    // 4. 1-9 direct slot keys:
    // If search is empty (or Alt is pressed), 1-9 directly copies the item!
    const num = parseInt(e.key, 10);
    if (!isNaN(num) && num >= 1 && num <= 9 && !e.ctrlKey) {
      if (settings.number_keys_copy && (!searchQuery.trim() || e.altKey)) {
        const targetIdx = num - 1;
        if (targetIdx < filteredClips.length) {
          e.preventDefault();
          copyClip(filteredClips[targetIdx].id);
          return;
        }
      }
    }

    // 5. Delete key to delete selected item
    if (e.key === 'Delete' || (e.key === 'Backspace' && !isSearchFocused)) {
      if (filteredClips.length > 0 && selectedIndex < filteredClips.length) {
        e.preventDefault();
        deleteClip(filteredClips[selectedIndex].id);
      }
      return;
    }

    // 6. 'P' key to toggle pin (when search input is not actively typing)
    if ((e.key === 'p' || e.key === 'P') && !isSearchFocused && !e.ctrlKey && !e.altKey) {
      if (filteredClips.length > 0 && selectedIndex < filteredClips.length) {
        e.preventDefault();
        togglePin(filteredClips[selectedIndex].id);
      }
      return;
    }
  }

  onMount(() => {
    loadClips();
    loadSettings();
    setTimeout(() => {
      const input = document.getElementById('search-input') as HTMLInputElement | null;
      input?.focus();
    }, 50);

    const unlistenUpdated = listen('clipboard-updated', () => {
      loadClips();
    });

    const unlistenOpened = listen('window-opened', () => {
      loadClips();
      searchQuery = '';
      selectedIndex = 0;
      setTimeout(() => {
        const input = document.getElementById('search-input') as HTMLInputElement | null;
        input?.focus();
      }, 50);
    });

    window.addEventListener('keydown', handleGlobalKeyDown);

    return () => {
      unlistenUpdated.then((fn) => fn());
      unlistenOpened.then((fn) => fn());
      window.removeEventListener('keydown', handleGlobalKeyDown);
    };
  });
</script>

<main
  class="w-full h-screen flex flex-col rounded-2xl shadow-2xl overflow-hidden font-sans select-none relative transition-colors duration-200 {getBlurClass(currentTheme.bgBlur)}"
  style="
    background-color: {hexToRgba(currentTheme.bgColor, currentTheme.bgOpacity)};
    color: {currentTheme.textColor};
    border: 1px solid {hexToRgba(currentTheme.borderColor, 80)};
  "
>
  <!-- Edge and corner resize handles -->
  <ResizeHandles />

  <!-- Custom Background Image Layer -->
  {#if currentTheme.bgImage}
    <div
      class="absolute inset-0 bg-cover bg-center pointer-events-none z-0 transition-opacity duration-300"
      style="
        background-image: url('{currentTheme.bgImage}');
        opacity: {currentTheme.bgImageOpacity / 100};
      "
    ></div>
  {/if}

  <div class="relative z-10 flex flex-col flex-1 overflow-hidden">
    <!-- Header with search and categories -->
    <Header
      bind:appMode
      bind:searchQuery
      bind:activeCategory
      {currentTheme}
      itemCount={filteredClips.length}
      onClearAll={handleClearAll}
      onOpenSettings={() => (isSettingsOpen = true)}
      onCloseWindow={() => invoke('hide_window')}
      onOpenExportModal={() => (isExportModalOpen = true)}
    />

    {#if appMode === 'clipboard'}
      <!-- Clips List -->
      <div class="flex-1 overflow-y-auto py-1 px-1">
        {#if filteredClips.length === 0}
          <!-- Empty State -->
          <div class="h-full flex flex-col items-center justify-center text-center p-6 text-slate-400 select-none">
            <div
              class="w-12 h-12 rounded-2xl flex items-center justify-center mb-3"
              style="background-color: {hexToRgba(currentTheme.textColor, 8)}; color: {hexToRgba(currentTheme.textColor, 50)};"
            >
              <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.75">
                <path stroke-linecap="round" stroke-linejoin="round" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
              </svg>
            </div>
            {#if searchQuery}
              <p class="text-sm font-medium" style="color: {currentTheme.textColor};">Eşleşen sonuç bulunamadı</p>
              <p class="text-xs mt-1" style="color: {hexToRgba(currentTheme.textColor, 60)};">"{searchQuery}" için kayıtlı bir pano öğesi yok.</p>
            {:else if activeCategory === 'pinned'}
              <p class="text-sm font-medium" style="color: {currentTheme.textColor};">Henüz sabitlenmiş öğe yok</p>
              <p class="text-xs mt-1" style="color: {hexToRgba(currentTheme.textColor, 60)};">Önemli notları sabitlemek için üzerindeki iğneye veya 'P' tuşuna basın.</p>
            {:else}
              <p class="text-sm font-medium" style="color: {currentTheme.textColor};">Pano geçmişiniz boş</p>
              <p class="text-xs mt-1" style="color: {hexToRgba(currentTheme.textColor, 60)};">Herhangi bir metin veya görsel kopyaladığınızda burada görünecektir.</p>
            {/if}
          </div>
        {:else}
          <!-- Render Clip Items -->
          {#each filteredClips as item, idx (item.id)}
            <ClipItemCard
              {item}
              index={idx}
              isSelected={idx === selectedIndex}
              {currentTheme}
            onSelect={() => {
              selectedIndex = idx;
              copyClip(item.id);
            }}
            onDelete={(e) => {
              e.stopPropagation();
              deleteClip(item.id);
            }}
            onTogglePin={(e) => {
              e.stopPropagation();
              togglePin(item.id);
            }}
            onMoveUp={(e) => {
              e.stopPropagation();
              moveItem(idx, 'up');
            }}
            onMoveDown={(e) => {
              e.stopPropagation();
              moveItem(idx, 'down');
            }}
          />
        {/each}
      {/if}
    </div>
  {:else}
    <!-- Secure Vault View -->
    <div class="flex-1 overflow-hidden">
      <VaultView {settings} {currentTheme} onCloseWindow={() => invoke('hide_window')} />
    </div>
  {/if}
  </div>

  <!-- Settings Modal -->
  <SettingsModal
    isOpen={isSettingsOpen}
    {settings}
    onClose={() => (isSettingsOpen = false)}
    onSave={handleSaveSettings}
  />

  {#if exportToast}
    <div class="absolute bottom-4 left-1/2 -translate-x-1/2 z-50 px-4 py-2.5 bg-slate-900/95 text-white border border-slate-700/80 rounded-xl shadow-2xl text-xs flex items-center gap-2 max-w-[90%] whitespace-pre-line animate-in fade-in slide-in-from-bottom-2">
      <span class="text-emerald-400 font-bold">✓</span>
      <span class="font-mono text-[11px] truncate">{exportToast}</span>
      <button
        onclick={() => (exportToast = null)}
        class="ml-2 text-slate-400 hover:text-white text-xs cursor-pointer"
        aria-label="Kapat"
      >
        ✕
      </button>
    </div>
  {/if}

  <!-- Clipboard Export Modal -->
  {#if isExportModalOpen}
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4 animate-in fade-in duration-150">
      <div
        class="rounded-2xl shadow-2xl w-full max-w-sm p-4 space-y-3.5"
        style="
          background-color: {currentTheme ? currentTheme.cardColor : '#ffffff'};
          border: 1px solid {currentTheme ? currentTheme.borderColor : '#e2e8f0'};
          color: {currentTheme ? currentTheme.textColor : '#0f172a'};
        "
      >
        <div class="flex items-center justify-between pb-2" style="border-bottom: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : '#e2e8f0'};">
          <h3 class="text-sm font-semibold flex items-center gap-1.5" style="color: {currentTheme ? currentTheme.textColor : '#0f172a'};">
            <span>📥</span>
            <span>Pano Geçmişini Dışa Aktar</span>
          </h3>
          <button
            onclick={() => (isExportModalOpen = false)}
            class="p-1 cursor-pointer hover:opacity-80"
            style="color: {currentTheme ? currentTheme.secondaryTextColor : '#64748b'};"
            title="Kapat"
            aria-label="Kapat"
          >
            ✕
          </button>
        </div>

        <p class="text-xs leading-relaxed" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#64748b'};">
          Tüm kopyalanan metin geçmişinizi İndirilenler klasörüne dosya olarak aktarabilirsiniz:
        </p>

        <div class="grid grid-cols-2 gap-2.5 pt-1">
          <button
            onclick={() => {
              isExportModalOpen = false;
              handleExportHistory('markdown');
            }}
            class="p-3 rounded-xl border text-left flex flex-col justify-between transition-all hover:brightness-95 active:scale-[0.98] cursor-pointer"
            style="
              background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 60) : '#f8fafc'};
              border-color: {currentTheme ? hexToRgba(currentTheme.borderColor, 80) : '#cbd5e1'};
            "
          >
            <span class="text-lg mb-1">📝</span>
            <span class="font-semibold text-xs" style="color: {currentTheme ? currentTheme.accentColor : '#10b981'};">Markdown (.md)</span>
            <span class="text-[10px] opacity-70 mt-0.5">Not defteri formatı</span>
          </button>

          <button
            onclick={() => {
              isExportModalOpen = false;
              handleExportHistory('text');
            }}
            class="p-3 rounded-xl border text-left flex flex-col justify-between transition-all hover:brightness-95 active:scale-[0.98] cursor-pointer"
            style="
              background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 60) : '#f8fafc'};
              border-color: {currentTheme ? hexToRgba(currentTheme.borderColor, 80) : '#cbd5e1'};
            "
          >
            <span class="text-lg mb-1">📄</span>
            <span class="font-semibold text-xs" style="color: {currentTheme ? currentTheme.accentColor : '#10b981'};">Düz Metin (.txt)</span>
            <span class="text-[10px] opacity-70 mt-0.5">Salt metin formatı</span>
          </button>
        </div>

        <div class="flex justify-end pt-2" style="border-top: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : '#e2e8f0'};">
          <button
            onclick={() => (isExportModalOpen = false)}
            class="px-3.5 py-1.5 text-xs rounded-lg cursor-pointer hover:opacity-80"
            style="
              background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 80) : '#f1f5f9'};
              color: {currentTheme ? currentTheme.secondaryTextColor : '#64748b'};
            "
          >
            Vazgeç
          </button>
        </div>
      </div>
    </div>
  {/if}
</main>
