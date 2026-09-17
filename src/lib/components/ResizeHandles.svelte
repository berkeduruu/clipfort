<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { invoke } from '@tauri-apps/api/core';
  import type { ResizeDirection } from '$lib/types';

  const appWindow = typeof window !== 'undefined' ? getCurrentWindow() : null;

  function handleResizeStart(direction: ResizeDirection, e: MouseEvent) {
    if (e.button !== 0) return;
    e.preventDefault();
    e.stopPropagation();

    if (appWindow) {
      invoke('notify_user_dragged').catch(() => {});
      appWindow.startResizeDragging(direction).catch((err) => {
        console.error('startResizeDragging error:', err);
      });
    }
  }
</script>

<!-- TOP EDGE -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="absolute top-0 left-3 right-3 h-2 cursor-n-resize z-[60] select-none group/resize-n"
  onmousedown={(e) => handleResizeStart('North', e)}
  title="Pencere yüksekliğini değiştirmek için sürükleyin"
>
  <div class="w-full h-full group-hover/resize-n:bg-amber-500/20 transition-colors"></div>
</div>

<!-- BOTTOM EDGE -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="absolute bottom-0 left-3 right-3 h-2 cursor-s-resize z-[60] select-none group/resize-s"
  onmousedown={(e) => handleResizeStart('South', e)}
  title="Pencere yüksekliğini değiştirmek için sürükleyin"
>
  <div class="w-full h-full group-hover/resize-s:bg-amber-500/20 transition-colors"></div>
</div>

<!-- LEFT EDGE -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="absolute left-0 top-3 bottom-3 w-2 cursor-w-resize z-[60] select-none group/resize-w"
  onmousedown={(e) => handleResizeStart('West', e)}
  title="Pencere genişliğini değiştirmek için sürükleyin"
>
  <div class="w-full h-full group-hover/resize-w:bg-amber-500/20 transition-colors"></div>
</div>

<!-- RIGHT EDGE -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="absolute right-0 top-3 bottom-3 w-2 cursor-e-resize z-[60] select-none group/resize-e"
  onmousedown={(e) => handleResizeStart('East', e)}
  title="Pencere genişliğini değiştirmek için sürükleyin"
>
  <div class="w-full h-full group-hover/resize-e:bg-amber-500/20 transition-colors"></div>
</div>

<!-- TOP-LEFT CORNER -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="absolute top-0 left-0 w-3.5 h-3.5 cursor-nw-resize z-[60] rounded-tl-2xl select-none group/resize-nw"
  onmousedown={(e) => handleResizeStart('NorthWest', e)}
  title="Boyutu değiştirmek için sürükleyin"
>
  <div class="w-full h-full group-hover/resize-nw:bg-amber-500/30 rounded-tl-2xl transition-colors"></div>
</div>

<!-- TOP-RIGHT CORNER -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="absolute top-0 right-0 w-3.5 h-3.5 cursor-ne-resize z-[60] rounded-tr-2xl select-none group/resize-ne"
  onmousedown={(e) => handleResizeStart('NorthEast', e)}
  title="Boyutu değiştirmek için sürükleyin"
>
  <div class="w-full h-full group-hover/resize-ne:bg-amber-500/30 rounded-tr-2xl transition-colors"></div>
</div>

<!-- BOTTOM-LEFT CORNER -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="absolute bottom-0 left-0 w-3.5 h-3.5 cursor-sw-resize z-[60] rounded-bl-2xl select-none group/resize-sw"
  onmousedown={(e) => handleResizeStart('SouthWest', e)}
  title="Boyutu değiştirmek için sürükleyin"
>
  <div class="w-full h-full group-hover/resize-sw:bg-amber-500/30 rounded-bl-2xl transition-colors"></div>
</div>

<!-- BOTTOM-RIGHT CORNER -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="absolute bottom-0 right-0 w-3.5 h-3.5 cursor-se-resize z-[60] rounded-br-2xl select-none group/resize-se"
  onmousedown={(e) => handleResizeStart('SouthEast', e)}
  title="Boyutu değiştirmek için sürükleyin"
>
  <div class="w-full h-full group-hover/resize-se:bg-amber-500/30 rounded-br-2xl transition-colors"></div>
</div>
