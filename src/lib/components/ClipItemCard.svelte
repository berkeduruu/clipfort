<script lang="ts">
  import type { ClipItem } from '$lib/types';
  import { formatRelativeTime, formatBytes, parseColorCode, parseUrl } from '$lib/utils';
  import { hexToRgba } from '$lib/theme';
  import { invoke } from '@tauri-apps/api/core';

  let {
    item,
    index,
    isSelected = false,
    currentTheme,
    onSelect,
    onDelete,
    onTogglePin,
    onMoveUp,
    onMoveDown,
  }: {
    item: ClipItem;
    index: number;
    isSelected: boolean;
    currentTheme?: any;
    onSelect: () => void;
    onDelete: (e: MouseEvent) => void;
    onTogglePin: (e: MouseEvent) => void;
    onMoveUp: (e: MouseEvent) => void;
    onMoveDown: (e: MouseEvent) => void;
  } = $props();

  let colorInfo = $derived(item.item_type === 'text' ? parseColorCode(item.content) : null);
  let urlInfo = $derived(item.item_type === 'text' ? parseUrl(item.content) : null);
  let colorPickerRef: HTMLInputElement | null = $state(null);

  async function openUrl(e: MouseEvent, url: string) {
    e.stopPropagation();
    try {
      await invoke('open_external_url', { url });
    } catch (err: any) {
      alert(typeof err === 'string' ? err : 'Failed to open link');
    }
  }

  function triggerColorPicker(e: MouseEvent) {
    e.stopPropagation();
    if (colorPickerRef) {
      colorPickerRef.click();
    }
  }
</script>

<!-- Card Container -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  id="clip-item-{index}"
  onclick={onSelect}
  class="group relative flex items-start gap-3 p-3 mx-2 my-1 rounded-xl transition-all duration-150 cursor-pointer border select-none {isSelected
    ? 'shadow-xs'
    : 'hover:brightness-95'}"
  style="
    background-color: {currentTheme
      ? (isSelected ? hexToRgba(currentTheme.accentColor, 18) : hexToRgba(currentTheme.cardColor, 88))
      : (isSelected ? 'rgba(16, 185, 129, 0.15)' : '#ffffff')};
    border-color: {currentTheme
      ? (isSelected ? currentTheme.accentColor : hexToRgba(currentTheme.borderColor, 70))
      : (isSelected ? '#10b981' : '#e2e8f0')};
    box-shadow: {isSelected ? `0 0 0 1px ${hexToRgba(currentTheme ? currentTheme.accentColor : '#10b981', 35)}` : 'none'};
    color: {currentTheme ? currentTheme.textColor : '#0f172a'};
  "
>
  <!-- Index Badge or Type Icon -->
  <div class="flex-shrink-0 pt-0.5">
    {#if index < 9}
      <span
        class="inline-flex items-center justify-center w-5 h-5 text-[11px] font-semibold rounded-md transition-colors"
        style="
          background-color: {isSelected
            ? (currentTheme ? currentTheme.accentColor : '#10b981')
            : (currentTheme ? hexToRgba(currentTheme.textColor, 12) : '#f1f5f9')};
          color: {isSelected
            ? (currentTheme ? currentTheme.accentTextColor : '#ffffff')
            : (currentTheme ? currentTheme.textColor : '#64748b')};
        "
        title="Press {index + 1} to select"
      >
        {index + 1}
      </span>
    {:else}
      <span class="inline-flex items-center justify-center w-5 h-5 text-slate-400">
        {#if item.item_type === 'image'}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.75">
            <path stroke-linecap="round" stroke-linejoin="round" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
          </svg>
        {:else}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.75">
            <path stroke-linecap="round" stroke-linejoin="round" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
        {/if}
      </span>
    {/if}
  </div>

  <!-- Content Section -->
  <div class="flex-1 min-w-0">
    {#if item.item_type === 'image'}
      <!-- Image preview thumbnail & metadata -->
      <div class="flex items-center gap-3">
        <div class="relative w-24 h-16 rounded-lg overflow-hidden border border-slate-200 bg-slate-100 flex-shrink-0 flex items-center justify-center">
          <img
            src={item.preview}
            alt="Preview"
            class="w-full h-full object-cover"
            loading="lazy"
          />
        </div>
        <div class="flex flex-col justify-center space-y-1 text-xs">
          <div class="font-medium flex items-center gap-1.5" style="color: {currentTheme ? currentTheme.textColor : '#334155'};">
            <span>Image</span>
            {#if item.image_width && item.image_height}
              <span class="font-normal" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#64748b'};">({item.image_width} × {item.image_height})</span>
            {/if}
          </div>
          <div class="text-[11px] flex items-center gap-2" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#64748b'};">
            {#if item.file_size_bytes}
              <span>{formatBytes(item.file_size_bytes)}</span>
              <span>•</span>
            {/if}
            <span>{formatRelativeTime(item.timestamp)}</span>
          </div>
        </div>
      </div>
    {:else}
      <!-- Text content preview -->
      <p
        class="text-xs line-clamp-2 leading-relaxed break-words font-normal select-text"
        style="color: {currentTheme ? currentTheme.textColor : '#0f172a'};"
      >
        {item.preview}
      </p>

      <!-- Color Swatch Badge -->
      {#if colorInfo}
        <div class="mt-1.5 flex items-center gap-2">
          <button
            type="button"
            onclick={triggerColorPicker}
            class="relative flex items-center gap-1.5 px-2 py-0.5 rounded-md border text-[11px] font-mono shadow-xs hover:scale-105 transition-transform"
            style="
              background-color: {currentTheme ? hexToRgba(currentTheme.cardColor, 95) : '#f8fafc'};
              border-color: {currentTheme ? hexToRgba(currentTheme.borderColor, 80) : '#e2e8f0'};
              color: {currentTheme ? currentTheme.textColor : '#0f172a'};
            "
            title="Live Color Preview - Click to open color palette"
          >
            <span
              class="w-3.5 h-3.5 rounded-full border border-black/20 shadow-inner flex-shrink-0 inline-block"
              style="background-color: {colorInfo.color};"
            ></span>
            <span>{colorInfo.color}</span>
            <input
              bind:this={colorPickerRef}
              type="color"
              value={colorInfo.hex}
              class="sr-only"
              tabindex="-1"
            />
          </button>
        </div>
      {/if}

      <!-- URL Preview Badge -->
      {#if urlInfo}
        <div class="mt-1.5 flex items-center gap-1.5">
          <span
            class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md border text-[11px] font-medium"
            style="
              background-color: {currentTheme ? hexToRgba(currentTheme.accentColor, 12) : '#ecfdf5'};
              border-color: {currentTheme ? hexToRgba(currentTheme.accentColor, 35) : '#a7f3d0'};
              color: {currentTheme ? currentTheme.accentColor : '#059669'};
            "
          >
            <img
              src="https://www.google.com/s2/favicons?domain={urlInfo.domain}&sz=32"
              alt=""
              class="w-3.5 h-3.5 rounded-xs flex-shrink-0 object-contain"
              onerror={(e) => { (e.currentTarget as HTMLElement).style.display = 'none'; }}
            />
            <span class="truncate max-w-[170px]">{urlInfo.domain}</span>
          </span>
          <button
            type="button"
            onclick={(e) => openUrl(e, urlInfo.url)}
            class="px-1.5 py-0.5 rounded-md text-[11px] font-medium hover:opacity-80 transition-opacity flex items-center gap-1"
            style="
              background-color: {currentTheme ? hexToRgba(currentTheme.accentColor, 18) : '#e0e7ff'};
              color: {currentTheme ? currentTheme.accentColor : '#3730a3'};
            "
            title="Open in Browser ({urlInfo.url})"
          >
            🌐 Open
          </button>
        </div>
      {/if}

      <div
        class="mt-1 flex items-center gap-2 text-[11px]"
        style="color: {currentTheme ? currentTheme.secondaryTextColor : '#64748b'};"
      >
        {#if item.char_count}
          <span>{item.char_count} chars</span>
          <span>•</span>
        {/if}
        {#if item.word_count}
          <span>{item.word_count} words</span>
          <span>•</span>
        {/if}
        <span>{formatRelativeTime(item.timestamp)}</span>
      </div>
    {/if}
  </div>

  <!-- Action controls (hover or selected) -->
  <div class="flex-shrink-0 flex items-center space-x-1 opacity-0 group-hover:opacity-100 {isSelected ? 'opacity-100' : ''} transition-opacity duration-150">
    <!-- Move Up Button -->
    <button
      onclick={onMoveUp}
      class="p-1 hover:opacity-75 rounded-md transition-all"
      style="color: {currentTheme ? currentTheme.secondaryTextColor : '#64748b'};"
      title="Move up (Alt + Up)"
    >
      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M5 15l7-7 7 7" />
      </svg>
    </button>

    <!-- Move Down Button -->
    <button
      onclick={onMoveDown}
      class="p-1 hover:opacity-75 rounded-md transition-all"
      style="color: {currentTheme ? currentTheme.secondaryTextColor : '#64748b'};"
      title="Move down (Alt + Down)"
    >
      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M19 9l-7 7-7-7" />
      </svg>
    </button>

    <!-- Pin Toggle Button -->
    <button
      onclick={onTogglePin}
      class="p-1 rounded-md transition-all hover:opacity-75"
      style="color: {item.pinned ? '#f59e0b' : (currentTheme ? currentTheme.secondaryTextColor : '#64748b')};"
      title={item.pinned ? 'Unpin (P)' : 'Pin to top (P)'}
    >
      <svg
        class="w-3.5 h-3.5"
        fill={item.pinned ? 'currentColor' : 'none'}
        stroke="currentColor"
        viewBox="0 0 24 24"
        stroke-width="2"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z"
        />
      </svg>
    </button>

    <!-- Delete Button -->
    <button
      onclick={onDelete}
      class="p-1 hover:text-rose-500 rounded-md transition-all hover:opacity-75"
      style="color: {currentTheme ? currentTheme.secondaryTextColor : '#64748b'};"
      title="Delete (Delete)"
    >
      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
      </svg>
    </button>
  </div>

  <!-- Pinned visual badge when not hovered -->
  {#if item.pinned}
    <div class="absolute top-2 right-2 group-hover:hidden text-amber-500">
      <svg class="w-3 h-3 fill-current" viewBox="0 0 24 24">
        <path d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
      </svg>
    </div>
  {/if}
</div>
