<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { VaultItem, VaultStatus, AppSettings } from '$lib/types';
  import { hexToRgba } from '$lib/theme';

  let {
    settings,
    currentTheme,
    onCloseWindow,
  }: {
    settings: AppSettings;
    currentTheme?: any;
    onCloseWindow: () => void;
  } = $props();

  let status = $state<VaultStatus | null>(null);
  let items = $state<VaultItem[]>([]);
  let tabs = $state<string[]>([]);
  let activeTab = $state<string>('');
  let isLoading = $state(true);

  // PIN inputs
  let pinInput = $state('');
  let confirmPinInput = $state('');
  let errorMessage = $state('');
  let successMessage = $state('');

  // PIN management modals
  let isPinModalOpen = $state(false);
  let newPin = $state('');
  let confirmNewPin = $state('');
  let pinModalError = $state('');
  let isRemovePinConfirmOpen = $state(false);

  // Search & Filtering
  let searchQuery = $state('');

  // Secrets visibility and copy feedback
  let revealedSecrets = $state<Record<string, boolean>>({});
  let copiedId = $state<string | null>(null);
  let actionMenuId = $state<string | null>(null);

  // Add/Edit Item Modal
  let isEditModalOpen = $state(false);
  let isEditingExisting = $state(false);
  let currentEditId = $state<string | null>(null);
  let formTitle = $state('');
  let formSecret = $state('');
  let formItemType = $state<'text' | 'password' | 'file' | 'note' | 'link'>('text');
  let formTab = $state('');
  let formGroup = $state('');
  let formPinned = $state(false);
  let formNotes = $state('');

  // File attachment state in modal
  let formFileName = $state<string | null>(null);
  let formFileBase64 = $state<string | null>(null);
  let formFileSizeBytes = $state<number | null>(null);
  let formFilePath = $state<string | null>(null);
  let formCopyToVault = $state(false); // false: shortcut (original path), true: copy to vault folder
  let fileError = $state('');

  // Add Tab Modal
  let isAddTabModalOpen = $state(false);
  let newTabName = $state('');

  // Tabs container ref & scroll helper
  let tabsContainer = $state<HTMLDivElement | null>(null);

  function scrollTabs(direction: 'left' | 'right') {
    if (tabsContainer) {
      tabsContainer.scrollBy({ left: direction === 'left' ? -180 : 180, behavior: 'smooth' });
    }
  }

  // File Path Modal
  let showPathModal = $state(false);

  // Pinned items derived across all tabs
  let pinnedItems = $derived(items.filter((i) => i.pinned));

  // Current tab items or search results
  let tabItems = $derived(
    items.filter((item) => {
      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase();
        return (
          item.title.toLowerCase().includes(q) ||
          item.group.toLowerCase().includes(q) ||
          (item.notes && item.notes.toLowerCase().includes(q)) ||
          (item.file_name && item.file_name.toLowerCase().includes(q))
        );
      }
      return item.tab === activeTab;
    })
  );

  // Group items by section within the active tab
  let groupedSections = $derived.by(() => {
    const map = new Map<string, VaultItem[]>();
    for (const item of tabItems) {
      const g = item.group || 'Genel';
      if (!map.has(g)) map.set(g, []);
      map.get(g)!.push(item);
    }
    return Array.from(map.entries()).map(([groupName, groupItems]) => ({
      groupName,
      items: groupItems,
    }));
  });

  function formatBytes(bytes?: number): string {
    if (!bytes) return '';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  async function refreshStatus() {
    isLoading = true;
    errorMessage = '';
    try {
      status = await invoke<VaultStatus>('get_vault_status');
      // If initialized, not unlocked, and has NO pin (Auto mode), auto-unlock immediately!
      if (status.is_initialized && !status.is_unlocked && !status.has_pin) {
        try {
          await invoke('auto_unlock_vault');
          status = await invoke<VaultStatus>('get_vault_status');
        } catch (autoErr) {
          console.error('Auto unlock error:', autoErr);
        }
      }

      if (status.is_unlocked) {
        items = await invoke<VaultItem[]>('get_vault_items');
        tabs = await invoke<string[]>('get_vault_tabs');
        if (!activeTab || !tabs.includes(activeTab)) {
          activeTab = tabs[0] || 'Kişisel Bilgiler';
        }
      } else {
        items = [];
        tabs = [];
      }
    } catch (err: any) {
      errorMessage = typeof err === 'string' ? err : 'Kasa durumu alınamadı';
    } finally {
      isLoading = false;
    }
  }

  async function handleInitAutoVault() {
    errorMessage = '';
    try {
      await invoke('init_vault', { pin: null });
      await refreshStatus();
    } catch (err: any) {
      errorMessage = typeof err === 'string' ? err : 'Kasa oluşturulamadı.';
    }
  }

  async function handleInitPinVault() {
    errorMessage = '';
    if (pinInput.trim().length < 4) {
      errorMessage = 'PIN kodu en az 4 karakter olmalıdır.';
      return;
    }
    if (pinInput !== confirmPinInput) {
      errorMessage = 'Girilen PIN kodları birbiriyle uyuşmuyor!';
      return;
    }

    try {
      await invoke('init_vault', { pin: pinInput.trim() });
      pinInput = '';
      confirmPinInput = '';
      await refreshStatus();
    } catch (err: any) {
      errorMessage = typeof err === 'string' ? err : 'Kasa oluşturulamadı.';
    }
  }

  async function handleSetPin() {
    pinModalError = '';
    if (newPin.trim().length < 4) {
      pinModalError = 'PIN kodu en az 4 karakter olmalıdır.';
      return;
    }
    if (newPin !== confirmNewPin) {
      pinModalError = 'Girilen PIN kodları uyuşmuyor!';
      return;
    }

    try {
      await invoke('set_vault_pin', { pin: newPin.trim() });
      newPin = '';
      confirmNewPin = '';
      isPinModalOpen = false;
      successMessage = 'PIN koruması başarıyla etkinleştirildi.';
      setTimeout(() => { successMessage = ''; }, 3000);
      await refreshStatus();
    } catch (err: any) {
      pinModalError = typeof err === 'string' ? err : 'PIN ayarlanamadı.';
    }
  }

  async function handleRemovePin() {
    try {
      await invoke('remove_vault_pin');
      isRemovePinConfirmOpen = false;
      successMessage = 'PIN koruması kaldırıldı. Kasa artık otomatik cihaz şifrelemesiyle açılacak.';
      setTimeout(() => { successMessage = ''; }, 3500);
      await refreshStatus();
    } catch (err: any) {
      alert(typeof err === 'string' ? err : 'PIN kaldırılamadı.');
    }
  }

  async function handleUnlockVault() {
    errorMessage = '';
    if (!pinInput.trim()) {
      errorMessage = 'Lütfen PIN kodunuzu girin.';
      return;
    }

    try {
      await invoke<VaultItem[]>('unlock_vault', { pin: pinInput });
      pinInput = '';
      await refreshStatus();
    } catch (err: any) {
      errorMessage = typeof err === 'string' ? err : 'Hatalı PIN kodu! Kasa açılamadı.';
    }
  }

  async function handleLockVault() {
    try {
      await invoke('lock_vault');
      revealedSecrets = {};
      actionMenuId = null;
      await refreshStatus();
    } catch (err) {
      console.error(err);
    }
  }

  async function handleAddTab() {
    if (!newTabName.trim()) return;
    try {
      tabs = await invoke<string[]>('add_vault_tab', { name: newTabName.trim() });
      activeTab = newTabName.trim();
      newTabName = '';
      isAddTabModalOpen = false;
    } catch (err: any) {
      alert(typeof err === 'string' ? err : 'Sekme eklenemedi');
    }
  }

  async function handleDeleteTab(tabName: string) {
    if (tabs.length <= 1) {
      alert('En az bir sekme bulunmalıdır.');
      return;
    }
    if (!confirm(`"${tabName}" sekmesini ve bu sekmedeki görünümü silmek istediğinize emin misiniz?`)) {
      return;
    }
    try {
      tabs = await invoke<string[]>('delete_vault_tab', { name: tabName });
      if (activeTab === tabName) {
        activeTab = tabs[0];
      }
    } catch (err: any) {
      alert(typeof err === 'string' ? err : 'Sekme silinemedi');
    }
  }

  async function handleTogglePin(id: string) {
    try {
      await invoke('toggle_vault_item_pin', { id });
      await refreshStatus();
    } catch (err: any) {
      console.error('Toggle pin failed:', err);
    }
  }

  function openAddModal(defaultGroup = 'Kişisel Veriler') {
    isEditingExisting = false;
    currentEditId = null;
    formTitle = '';
    formSecret = '';
    formItemType = 'text';
    formTab = activeTab || (tabs[0] || 'Kişisel Bilgiler');
    formGroup = defaultGroup;
    formPinned = false;
    formNotes = '';
    formFileName = null;
    formFileBase64 = null;
    formFileSizeBytes = null;
    formFilePath = null;
    formCopyToVault = false;
    fileError = '';
    errorMessage = '';
    isEditModalOpen = true;
    tick().then(() => {
      document.getElementById('vault-form-title')?.focus();
    });
  }

  function openEditModal(item: VaultItem) {
    isEditingExisting = true;
    currentEditId = item.id;
    formTitle = item.title;
    formSecret = item.secret;
    formItemType = (item.item_type as any) || 'text';
    formTab = item.tab || activeTab;
    formGroup = item.group || 'Genel';
    formPinned = item.pinned;
    formNotes = item.notes || '';
    formFileName = item.file_name || null;
    formFileBase64 = null; // unchanged unless new file chosen
    formFileSizeBytes = item.file_size_bytes || null;
    formFilePath = item.file_path || null;
    formCopyToVault = false;
    fileError = '';
    errorMessage = '';
    isEditModalOpen = true;
  }

  async function handlePickNativeFile() {
    fileError = '';
    try {
      const picked = await invoke<{ path: string; name: string; size_bytes: number } | null>('pick_vault_file');
      if (picked) {
        const maxBytes = (settings.max_vault_file_size_mb || 20) * 1024 * 1024;
        if (picked.size_bytes > maxBytes) {
          fileError = `Dosya boyutu ${settings.max_vault_file_size_mb || 20} MB sınırını aşıyor! (${(picked.size_bytes / 1048576).toFixed(1)} MB)`;
          return;
        }
        formFileName = picked.name;
        formFilePath = picked.path;
        formFileSizeBytes = picked.size_bytes;
        formFileBase64 = null;
        if (!formTitle.trim()) {
          formTitle = picked.name;
        }
      }
    } catch (err: any) {
      fileError = typeof err === 'string' ? err : 'Dosya seçici açılamadı.';
    }
  }

  function handleFileSelected(e: Event) {
    fileError = '';
    const input = e.target as HTMLInputElement;
    if (!input.files || input.files.length === 0) return;
    const file = input.files[0];

    const maxBytes = (settings.max_vault_file_size_mb || 20) * 1024 * 1024;
    if (file.size > maxBytes) {
      fileError = `Dosya boyutu ${settings.max_vault_file_size_mb || 20} MB sınırını aşıyor! (${(file.size / 1048576).toFixed(1)} MB)`;
      input.value = '';
      return;
    }

    formFileName = file.name;
    formFileSizeBytes = file.size;
    formFilePath = null;
    if (!formTitle.trim()) {
      formTitle = file.name;
    }

    const reader = new FileReader();
    reader.onload = () => {
      const result = reader.result as string;
      // Strip data URL scheme prefix if present
      const base64 = result.includes(',') ? result.split(',')[1] : result;
      formFileBase64 = base64;
    };
    reader.onerror = () => {
      fileError = 'Dosya okunamadı.';
    };
    reader.readAsDataURL(file);
  }

  async function handleSaveItem() {
    if (!formTitle.trim()) {
      errorMessage = 'Lütfen bir başlık girin.';
      return;
    }
    if (formItemType !== 'file' && !formSecret.trim()) {
      errorMessage = 'Lütfen gizli bilgiyi / değeri girin.';
      return;
    }
    if (formItemType === 'file' && !formFileName && !formFilePath && !isEditingExisting) {
      errorMessage = 'Lütfen bir dosya seçin.';
      return;
    }

    try {
      await invoke('save_vault_item', {
        id: currentEditId,
        title: formTitle.trim(),
        secret: formSecret.trim(),
        itemType: formItemType,
        tab: formTab || activeTab,
        group: formGroup.trim() || 'Genel',
        pinned: formPinned,
        fileName: formFileName,
        fileBase64: formFileBase64,
        filePath: formFilePath,
        copyToVault: formCopyToVault,
        notes: formNotes.trim() ? formNotes.trim() : null,
      });
      isEditModalOpen = false;
      await refreshStatus();
    } catch (err: any) {
      errorMessage = typeof err === 'string' ? err : 'Öğe kaydedilemedi.';
    }
  }

  async function handleDeleteItem(id: string) {
    if (!confirm('Bu kaydı kalıcı olarak silmek istediğinize emin misiniz?')) return;
    try {
      await invoke('delete_vault_item', { id });
      await refreshStatus();
    } catch (err: any) {
      alert(typeof err === 'string' ? err : 'Öğe silinemedi.');
    }
  }

  async function handleItemClick(item: VaultItem) {
    if (item.item_type === 'file') {
      await handleCopyFile(item);
    } else {
      await handleCopySecret(item.id);
    }
  }

  async function handleCopySecret(id: string) {
    try {
      await invoke('copy_vault_secret', { id });
      copiedId = id;
      setTimeout(() => {
        if (copiedId === id) copiedId = null;
      }, 1500);

      if (settings.close_on_copy) {
        onCloseWindow();
      }
    } catch (err: any) {
      alert(typeof err === 'string' ? err : 'Kopyalama başarısız');
    }
  }

  async function handleCopyFile(item: VaultItem) {
    try {
      await invoke('copy_vault_file', { id: item.id });
      copiedId = item.id;
      successMessage = `"${item.title || item.file_name || 'Belge'}" panoya kopyalandı.`;
      setTimeout(() => {
        if (copiedId === item.id) copiedId = null;
      }, 1500);
      setTimeout(() => {
        if (successMessage.includes(item.title || '')) successMessage = '';
      }, 3000);

      if (settings.close_on_copy) {
        onCloseWindow();
      }
    } catch (err: any) {
      alert(typeof err === 'string' ? err : 'Dosya panoya kopyalanamadı');
    }
  }

  async function handleExportFile(item: VaultItem) {
    try {
      const savedPath = await invoke<string>('export_vault_file', { id: item.id });
      successMessage = `"${item.file_name || item.title}" İndirilenler klasörüne kaydedildi: ${savedPath}`;
      setTimeout(() => {
        successMessage = '';
      }, 4000);
    } catch (err: any) {
      alert(typeof err === 'string' ? err : 'Dosya dışa aktarılamadı');
    }
  }

  async function handleOpenFile(item: VaultItem) {
    try {
      await invoke('open_vault_file', { id: item.id });
    } catch (err: any) {
      alert(typeof err === 'string' ? err : 'Dosya açılamadı');
    }
  }

  function toggleReveal(id: string) {
    revealedSecrets[id] = !revealedSecrets[id];
  }

  function getItemIcon(item: VaultItem) {
    switch (item.item_type) {
      case 'file':
        return 'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z';
      case 'password':
        return 'M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z';
      case 'link':
        return 'M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1';
      case 'note':
        return 'M7 8h10M7 12h4m1 8l-4-4H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-3l-4 4z';
      default:
        return 'M10 6H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V8a2 2 0 00-2-2h-5m-4 0V5a2 2 0 114 0v1m-4 0a2 2 0 104 0m-5 8a2 2 0 100-4 2 2 0 000 4zm0 0c1.306 0 2.417.835 2.83 2M9 14a3.001 3.001 0 00-2.83 2M15 11h3m-3 4h2';
    }
  }

  async function handleOpenUrl(url: string) {
    if (!url) return;
    try {
      await invoke('open_external_url', { url });
    } catch (err: any) {
      alert(typeof err === 'string' ? err : 'Bağlantı açılamadı');
    }
  }

  // Backup & Restore State & Methods
  let isBackupModalOpen = $state(false);
  let backupStatusMessage = $state('');
  let backupErrorMessage = $state('');
  let isBackingUp = $state(false);

  async function handleExportBackup() {
    isBackingUp = true;
    backupStatusMessage = '';
    backupErrorMessage = '';
    try {
      const savedPath = await invoke<string>('export_vault_backup');
      backupStatusMessage = `Kasa yedeği başarıyla kaydedildi:\n${savedPath}`;
      successMessage = 'Kasa yedeği başarıyla oluşturuldu.';
      setTimeout(() => {
        if (successMessage.includes('Kasa yedeği')) successMessage = '';
      }, 4000);
    } catch (err: any) {
      backupErrorMessage = typeof err === 'string' ? err : 'Yedekleme başarısız';
    } finally {
      isBackingUp = false;
    }
  }

  async function handleRestoreBackup() {
    isBackingUp = true;
    backupStatusMessage = '';
    backupErrorMessage = '';
    try {
      await invoke('restore_vault_backup');
      isBackupModalOpen = false;
      successMessage = 'Kasa yedeği başarıyla geri yüklendi!';
      await refreshStatus();
      setTimeout(() => {
        if (successMessage.includes('geri yüklendi')) successMessage = '';
      }, 4000);
    } catch (err: any) {
      backupErrorMessage = typeof err === 'string' ? err : 'Geri yükleme başarısız';
    } finally {
      isBackingUp = false;
    }
  }

  onMount(() => {
    refreshStatus();
  });
</script>

<div class="h-full flex flex-col bg-transparent overflow-hidden select-none" style="color: {currentTheme ? currentTheme.textColor : '#f1f5f9'};">
  {#if isLoading}
    <div class="h-full flex items-center justify-center" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2" style="border-color: {currentTheme ? currentTheme.accentColor : '#f59e0b'};"></div>
    </div>
  {:else if !status?.is_initialized}
    <!-- 1. INITIAL SETUP VIEW: Dual Mode Choice -->
    <div class="flex-1 flex flex-col items-center justify-center p-5 text-center max-w-md mx-auto">
      <div
        class="w-12 h-12 rounded-2xl flex items-center justify-center mb-2.5 shadow-inner"
        style="
          background-color: {currentTheme ? hexToRgba(currentTheme.accentColor, 15) : 'rgba(245, 158, 11, 0.2)'};
          border: 1px solid {currentTheme ? hexToRgba(currentTheme.accentColor, 35) : 'rgba(245, 158, 11, 0.3)'};
          color: {currentTheme ? currentTheme.accentColor : '#fbbf24'};
        "
      >
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
        </svg>
      </div>

      <h2 class="text-base font-semibold" style="color: {currentTheme ? currentTheme.textColor : '#ffffff'};">Güvenli Kasa Kurulumu</h2>
      <p class="text-xs mt-1 mb-3.5 leading-relaxed" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">
        Şifreleriniz ve dosyalarınız <strong>AES-256-GCM</strong> ile şifrelenir. Başlamak için tercihinizi seçin:
      </p>

      {#if errorMessage}
        <div class="w-full mb-3 px-3 py-1.5 bg-rose-950/60 border border-rose-500/50 text-rose-300 text-xs rounded-lg text-left">
          {errorMessage}
        </div>
      {/if}

      <div class="w-full space-y-2.5 text-left">
        <!-- Option 1: Fast Start (Auto Machine Encryption) -->
        <div
          class="p-3 rounded-xl transition-all shadow-sm"
          style="
            background-color: {currentTheme ? hexToRgba(currentTheme.cardColor, 90) : 'rgba(30, 41, 59, 0.8)'};
            border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 80) : 'rgba(245, 158, 11, 0.4)'};
          "
        >
          <div class="flex items-center justify-between mb-1">
            <span class="text-xs font-semibold flex items-center gap-1.5" style="color: {currentTheme ? currentTheme.accentColor : '#fde68a'};">
              <span>🛡️</span> Otomatik Cihaz Şifrelemesi
            </span>
            <span
              class="px-1.5 py-0.5 text-[10px] rounded font-medium"
              style="
                background-color: {currentTheme ? hexToRgba(currentTheme.accentColor, 20) : 'rgba(245, 158, 11, 0.2)'};
                color: {currentTheme ? currentTheme.accentColor : '#fbbf24'};
                border: 1px solid {currentTheme ? hexToRgba(currentTheme.accentColor, 30) : 'rgba(245, 158, 11, 0.3)'};
              "
            >
              Önerilen
            </span>
          </div>
          <p class="text-[11px] leading-snug" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">
            PIN kodu girmeden anında açılır. Verileriniz disk üzerinde cihaza özel 256-bit AES anahtarı ile şifrelenir.
          </p>
          <button
            onclick={handleInitAutoVault}
            class="mt-2.5 w-full py-1.5 font-bold text-xs rounded-lg shadow-sm transition-all active:scale-[0.99] flex items-center justify-center gap-1.5 cursor-pointer"
            style="
              background-color: {currentTheme ? currentTheme.accentColor : '#f59e0b'};
              color: #ffffff;
            "
          >
            <span>🚀 Hemen Başla (PIN'siz / Otomatik)</span>
          </button>
        </div>

        <!-- Divider -->
        <div class="flex items-center gap-2 text-[11px] px-1 py-0.5" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#64748b'};">
          <div class="flex-1 h-px" style="background-color: {currentTheme ? hexToRgba(currentTheme.borderColor, 40) : '#334155'};"></div>
          <span>veya</span>
          <div class="flex-1 h-px" style="background-color: {currentTheme ? hexToRgba(currentTheme.borderColor, 40) : '#334155'};"></div>
        </div>

        <!-- Option 2: PIN Protected -->
        <div
          class="p-3 rounded-xl"
          style="
            background-color: {currentTheme ? hexToRgba(currentTheme.cardColor, 70) : 'rgba(30, 41, 59, 0.5)'};
            border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : '#334155'};
          "
        >
          <span class="text-xs font-semibold flex items-center gap-1.5 mb-1" style="color: {currentTheme ? currentTheme.textColor : '#f1f5f9'};">
            <span>🔒</span> PIN Koruması Belirle (İsteğe Bağlı)
          </span>
          <p class="text-[11px] mb-2" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">
            Her oturumda kasanın kilidini açmak için bir anahtar PIN kodu istenir.
          </p>

          <div class="space-y-2">
            <input
              type="password"
              bind:value={pinInput}
              placeholder="Anahtar PIN Belirleyin (en az 4 hane)"
              class="w-full px-3 py-1.5 text-xs rounded-lg focus:outline-none"
              style="
                background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 90) : '#0f172a'};
                color: {currentTheme ? currentTheme.textColor : '#ffffff'};
                border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 80) : '#334155'};
              "
              autocomplete="new-password"
            />
            <input
              type="password"
              bind:value={confirmPinInput}
              placeholder="PIN Kodunu Doğrulayın"
              class="w-full px-3 py-1.5 text-xs rounded-lg focus:outline-none"
              style="
                background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 90) : '#0f172a'};
                color: {currentTheme ? currentTheme.textColor : '#ffffff'};
                border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 80) : '#334155'};
              "
              autocomplete="new-password"
              onkeydown={(e) => e.key === 'Enter' && handleInitPinVault()}
            />
            <button
              onclick={handleInitPinVault}
              class="w-full py-1.5 font-medium text-xs rounded-lg transition-colors cursor-pointer"
              style="
                background-color: {currentTheme ? currentTheme.accentColor : '#475569'};
                color: #ffffff;
              "
            >
              PIN ile Koru ve Başlat
            </button>
          </div>
        </div>
      </div>
    </div>

  {:else if !status?.is_unlocked}
    <!-- 2. LOCKED VIEW: Enter Master PIN -->
    <div class="flex-1 flex flex-col items-center justify-center p-6 text-center max-w-sm mx-auto">
      <div
        class="w-14 h-14 rounded-2xl flex items-center justify-center mb-3 shadow-inner"
        style="
          background-color: {currentTheme ? hexToRgba(currentTheme.accentColor, 15) : 'rgba(245, 158, 11, 0.2)'};
          border: 1px solid {currentTheme ? hexToRgba(currentTheme.accentColor, 35) : 'rgba(245, 158, 11, 0.3)'};
          color: {currentTheme ? currentTheme.accentColor : '#fbbf24'};
        "
      >
        <svg class="w-7 h-7" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
        </svg>
      </div>

      <h2 class="text-base font-semibold" style="color: {currentTheme ? currentTheme.textColor : '#ffffff'};">Kasa Kilitli</h2>
      <p class="text-xs mt-1 mb-4" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">
        Bu oturum için anahtar PIN kodunuzu girin. Bilgisayar açık kaldığı sürece kasanız açık kalır.
      </p>

      {#if errorMessage}
        <div class="w-full mb-3 px-3 py-1.5 bg-rose-950/60 border border-rose-500/50 text-rose-300 text-xs rounded-lg text-left">
          {errorMessage}
        </div>
      {/if}

      <div class="w-full space-y-2.5">
        <input
          id="vault-unlock-pin"
          type="password"
          bind:value={pinInput}
          placeholder="PIN Kodunu Girin..."
          class="w-full px-3.5 py-2 text-sm rounded-xl shadow-sm text-center tracking-widest font-mono text-base focus:outline-none"
          style="
            background-color: {currentTheme ? hexToRgba(currentTheme.cardColor, 90) : 'rgba(30, 41, 59, 0.9)'};
            color: {currentTheme ? currentTheme.textColor : '#ffffff'};
            border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 80) : '#334155'};
          "
          autocomplete="current-password"
          onkeydown={(e) => e.key === 'Enter' && handleUnlockVault()}
        />
        <button
          onclick={handleUnlockVault}
          class="w-full py-2 font-medium text-xs rounded-xl shadow-md transition-all active:scale-[0.99] cursor-pointer"
          style="
            background-color: {currentTheme ? currentTheme.accentColor : '#f59e0b'};
            color: #ffffff;
          "
        >
          Kilidi Aç
        </button>
      </div>
    </div>

  {:else}
    <!-- 3. UNLOCKED VIEW: Full Dashboard matching User's Layout Reference -->

    {#if successMessage}
      <div class="mx-3 mt-2 px-3 py-1.5 bg-emerald-950/90 border border-emerald-500/50 text-emerald-300 text-xs rounded-lg flex items-center justify-between shadow-sm">
        <span class="flex items-center gap-1.5 font-medium">
          <span>✓</span>
          <span>{successMessage}</span>
        </span>
        <button onclick={() => (successMessage = '')} class="text-emerald-400 hover:text-white text-xs ml-2">✕</button>
      </div>
    {/if}

    <!-- TOP PINNED BAR -->
    {#if pinnedItems.length > 0}
      <div
        class="m-3 mb-1 p-2.5 rounded-xl flex items-center justify-between"
        style="
          background-color: {currentTheme ? hexToRgba(currentTheme.cardColor, 85) : 'rgba(30, 41, 59, 0.8)'};
          border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : 'rgba(245, 158, 11, 0.3)'};
        "
      >
        <div class="flex items-center space-x-2 overflow-x-auto py-0.5 scrollbar-thin">
          {#each pinnedItems as item (item.id)}
            <button
              onclick={() => handleItemClick(item)}
              class="px-3 py-1 rounded-lg text-xs font-medium flex items-center space-x-1.5 shrink-0 shadow-xs transition-all active:scale-95 cursor-pointer"
              style="
                background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 80) : 'rgba(15, 23, 42, 0.9)'};
                color: {currentTheme ? currentTheme.textColor : '#fde68a'};
                border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 80) : 'rgba(245, 158, 11, 0.4)'};
              "
              title={item.item_type === 'file' ? 'Belgeyi panoya kopyalamak için tıkla' : 'Kopyalamak için tıkla'}
            >
              <svg class="w-3 h-3 shrink-0" style="color: {currentTheme ? currentTheme.accentColor : '#fbbf24'};" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d={getItemIcon(item)} />
              </svg>
              <span>{item.title}</span>
              {#if copiedId === item.id}
                <span class="text-[10px] text-emerald-400 font-bold ml-1">✓</span>
              {/if}
            </button>
          {/each}
        </div>

        <div class="flex items-center space-x-2 text-[11px] shrink-0 pl-2" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#fbbf24'};">
          <span class="font-semibold">{pinnedItems.length} Sabitlendi</span>
        </div>
      </div>
    {/if}

    <!-- 1. KATEGORİ / KONU BARLARI (User Diagram Kutu 1) -->
    <div
      class="px-3 py-1 flex items-center gap-1.5 select-none"
      style="
        border-bottom: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : 'rgba(30, 41, 59, 0.8)'};
        background-color: {currentTheme ? hexToRgba(currentTheme.cardColor, 60) : 'rgba(2, 6, 23, 0.4)'};
      "
    >
      <!-- Tabs Scrollable Area -->
      <div
        bind:this={tabsContainer}
        class="flex-1 flex items-center space-x-1.5 overflow-x-auto no-scrollbar scroll-smooth py-2 px-1"
        onwheel={(e) => {
          if (e.deltaY !== 0) {
            e.preventDefault();
            e.currentTarget.scrollLeft += e.deltaY;
          }
        }}
      >
        {#each tabs as tabName}
          <div class="relative group flex items-center shrink-0">
            <button
              onclick={() => (activeTab = tabName)}
              class="px-3.5 py-1.5 rounded-lg text-xs font-semibold whitespace-nowrap shrink-0 transition-all cursor-pointer shadow-xs"
              style="
                background-color: {activeTab === tabName
                  ? (currentTheme ? currentTheme.accentColor : '#f59e0b')
                  : (currentTheme ? hexToRgba(currentTheme.cardColor, 90) : 'rgba(30, 41, 59, 0.8)')};
                color: {activeTab === tabName
                  ? '#ffffff'
                  : (currentTheme ? currentTheme.secondaryTextColor : '#cbd5e1')};
                border: 1px solid {activeTab === tabName
                  ? (currentTheme ? currentTheme.accentColor : '#f59e0b')
                  : (currentTheme ? hexToRgba(currentTheme.borderColor, 60) : 'rgba(51, 65, 85, 0.6)')};
              "
            >
              {tabName}
            </button>
            {#if tabs.length > 1}
              <button
                onclick={(e) => { e.stopPropagation(); handleDeleteTab(tabName); }}
                class="hidden group-hover:flex absolute -top-1 -right-1 w-4 h-4 bg-rose-600 hover:bg-rose-500 text-white rounded-full items-center justify-center shadow-md cursor-pointer transition-transform hover:scale-110 z-10"
                title="Sekmeyi Sil"
              >
                <svg class="w-2.5 h-2.5 text-white pointer-events-none" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2.5">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            {/if}
          </div>
        {/each}
      </div>

      <!-- Quick Scroll Controls (‹ ›) -->
      <div
        class="flex items-center space-x-0.5 shrink-0 rounded-lg p-0.5"
        style="
          background-color: {currentTheme ? hexToRgba(currentTheme.cardColor, 90) : 'rgba(15, 23, 42, 0.9)'};
          border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : 'rgba(30, 41, 59, 0.8)'};
        "
      >
        <button
          onclick={() => scrollTabs('left')}
          class="w-6 h-6 flex items-center justify-center rounded text-xs transition-colors cursor-pointer hover:opacity-80"
          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};"
          title="Sola Kaydır"
        >
          ‹
        </button>
        <button
          onclick={() => scrollTabs('right')}
          class="w-6 h-6 flex items-center justify-center rounded text-xs transition-colors cursor-pointer hover:opacity-80"
          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};"
          title="Sağa Kaydır"
        >
          ›
        </button>
      </div>

      <!-- Always visible + Yeni Konu Button -->
      <button
        onclick={() => (isAddTabModalOpen = true)}
        class="shrink-0 px-2.5 py-1.5 rounded-lg text-xs font-bold flex items-center gap-1 transition-all cursor-pointer hover:opacity-90 shadow-xs"
        style="
          background-color: {currentTheme ? hexToRgba(currentTheme.cardColor, 90) : 'rgba(30, 41, 59, 0.9)'};
          color: {currentTheme ? currentTheme.accentColor : '#fbbf24'};
          border: 1px solid {currentTheme ? hexToRgba(currentTheme.accentColor, 40) : 'rgba(245, 158, 11, 0.3)'};
        "
        title="Yeni Kategori / Konu Ekle"
      >
        <span class="text-sm font-black leading-none">+</span>
        <span class="text-[11px] font-medium">Yeni</span>
      </button>
    </div>

    <!-- 2. SEARCH BAR & İŞLEM BUTONLARI (User Diagram Kutu 2) -->
    <div
      class="px-4 py-2 flex items-center justify-between gap-3 select-none"
      style="
        border-bottom: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : 'rgba(30, 41, 59, 0.8)'};
        background-color: {currentTheme ? hexToRgba(currentTheme.cardColor, 40) : 'rgba(2, 6, 23, 0.2)'};
      "
    >
      <!-- Live Search Bar -->
      <div class="relative flex-1 flex items-center">
        <svg class="w-4 h-4 absolute left-3 pointer-events-none" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
        </svg>
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Kasada ara... (şifre veya başlık)"
          class="w-full pl-9 pr-7 py-1.5 text-xs rounded-xl focus:outline-none transition-all duration-150 shadow-xs"
          style="
            background-color: {currentTheme ? hexToRgba(currentTheme.cardColor, 90) : 'rgba(30, 41, 59, 0.9)'};
            border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 75) : 'rgba(51, 65, 85, 0.8)'};
            color: {currentTheme ? currentTheme.textColor : '#ffffff'};
          "
        />
        {#if searchQuery}
          <button
            onclick={() => (searchQuery = '')}
            class="absolute right-2 text-xs cursor-pointer p-0.5 hover:opacity-80"
            style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};"
            title="Aramayı Temizle"
          >
            ✕
          </button>
        {/if}
      </div>

      <!-- Action Buttons -->
      <div class="flex items-center space-x-1.5 shrink-0">
        <!-- + Ekle Button -->
        <button
          onclick={() => openAddModal()}
          class="px-3 py-1.5 font-bold rounded-xl text-xs flex items-center space-x-1 shadow-sm transition-all active:scale-95 whitespace-nowrap cursor-pointer shrink-0"
          style="
            background-color: {currentTheme ? currentTheme.accentColor : '#f59e0b'};
            color: #ffffff;
          "
          title="Yeni Bilgi / Dosya Ekle"
        >
          <svg class="w-3.5 h-3.5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2.5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 4v16m8-8H4" />
          </svg>
          <span>Ekle</span>
        </button>

        <!-- Encryption Mode Quick Toggle (🛡️) -->
        {#if status?.has_pin}
          <button
            onclick={() => (isRemovePinConfirmOpen = true)}
            class="p-2 rounded-xl transition-colors cursor-pointer shrink-0 relative hover:opacity-80"
            style="
              background-color: {currentTheme ? hexToRgba(currentTheme.cardColor, 90) : 'rgba(30, 41, 59, 0.8)'};
              border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : 'rgba(245, 158, 11, 0.4)'};
              color: {currentTheme ? currentTheme.accentColor : '#fbbf24'};
            "
            title="PIN korumalı kasa. Otomatik şifrelemeye geçmek için tıklayın."
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
            </svg>
            <span class="absolute top-1 right-1 w-2 h-2 rounded-full animate-pulse" style="background-color: {currentTheme ? currentTheme.accentColor : '#fbbf24'};"></span>
          </button>
        {:else}
          <button
            onclick={() => { newPin = ''; confirmNewPin = ''; pinModalError = ''; isPinModalOpen = true; }}
            class="p-2 rounded-xl transition-colors cursor-pointer shrink-0 hover:opacity-80"
            style="
              background-color: {currentTheme ? hexToRgba(currentTheme.cardColor, 90) : 'rgba(30, 41, 59, 0.8)'};
              border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : 'rgba(51, 65, 85, 0.8)'};
              color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};
            "
            title="Otomatik cihaz şifreli. İsteğe bağlı PIN koruması eklemek için tıklayın."
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
            </svg>
          </button>
        {/if}

        <!-- File / Path Info Button (📁) -->
        <button
          onclick={() => (showPathModal = true)}
          class="p-2 rounded-xl transition-colors cursor-pointer shrink-0 hover:opacity-80"
          style="
            background-color: {currentTheme ? hexToRgba(currentTheme.cardColor, 90) : 'rgba(30, 41, 59, 0.8)'};
            border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : 'rgba(51, 65, 85, 0.8)'};
            color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};
          "
          title="Kasa Dosyası ve Taşınabilirlik"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
          </svg>
        </button>

        <!-- Backup & Restore Button (💾) -->
        <button
          onclick={() => { backupStatusMessage = ''; backupErrorMessage = ''; isBackupModalOpen = true; }}
          class="p-2 rounded-xl transition-colors cursor-pointer shrink-0 hover:opacity-80"
          style="
            background-color: {currentTheme ? hexToRgba(currentTheme.cardColor, 90) : 'rgba(30, 41, 59, 0.8)'};
            border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : 'rgba(51, 65, 85, 0.8)'};
            color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};
          "
          title="Kasa Tam Yedekleme & Geri Yükleme (.vaultbak)"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4" />
          </svg>
        </button>

        <!-- Lock Button (🔒) -->
        <button
          onclick={handleLockVault}
          class="p-2 rounded-xl transition-colors cursor-pointer shrink-0 hover:opacity-80"
          style="
            background-color: {currentTheme ? hexToRgba(currentTheme.cardColor, 90) : 'rgba(30, 41, 59, 0.8)'};
            border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : 'rgba(51, 65, 85, 0.8)'};
            color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};
          "
          title="Kasayı Kilitle"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
          </svg>
        </button>
      </div>
    </div>

    <!-- MAIN DASHBOARD CONTENT: Sections Grid -->
    <div class="flex-1 overflow-y-auto p-4">
      {#if groupedSections.length === 0}
        <div class="h-full flex flex-col items-center justify-center text-center p-8">
          <svg class="w-10 h-10 mb-2" style="color: {currentTheme ? hexToRgba(currentTheme.secondaryTextColor, 40) : '#475569'};" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
          <p class="text-sm font-medium" style="color: {currentTheme ? currentTheme.textColor : '#cbd5e1'};">Bu sekmede henüz kayıt yok</p>
          <p class="text-xs mt-1" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#64748b'};">
            Yukarıdaki "+ Ekle" butonunu kullanarak şifreler, kimlikler veya {settings.max_vault_file_size_mb || 20}MB'a kadar belgeler ekleyebilirsiniz.
          </p>
        </div>
      {:else}
        <!-- Grid matching the reference user uploaded image -->
        <div class="grid grid-cols-1 min-[500px]:grid-cols-2 lg:grid-cols-3 gap-3">
          {#each groupedSections as section (section.groupName)}
            <div
              class="rounded-2xl p-3 shadow-xs flex flex-col justify-between"
              style="
                background-color: {currentTheme ? hexToRgba(currentTheme.cardColor, 85) : 'rgba(30, 41, 59, 0.6)'};
                border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 80) : '#334155'};
              "
            >
              <div>
                <!-- Section Header -->
                <div class="flex items-center justify-between mb-2">
                  <h3 class="text-xs font-semibold flex items-center gap-1.5" style="color: {currentTheme ? currentTheme.textColor : '#f1f5f9'};">
                    <span class="w-1.5 h-1.5 rounded-full" style="background-color: {currentTheme ? currentTheme.accentColor : '#fbbf24'};"></span>
                    {section.groupName}
                  </h3>
                  <button
                    onclick={() => openAddModal(section.groupName)}
                    class="text-[11px] px-1.5 py-0.5 rounded transition-colors cursor-pointer hover:opacity-80"
                    style="color: {currentTheme ? currentTheme.accentColor : '#fbbf24'};"
                    title="Bu gruba yeni ekle"
                  >
                    + Ekle
                  </button>
                </div>

                <!-- Section Pills Flow -->
                <div class="flex flex-wrap gap-1.5 pt-1">
                  {#each section.items as item (item.id)}
                    <div class="relative group">
                      <button
                        onclick={() => handleItemClick(item)}
                        class="px-2.5 py-1.5 rounded-xl text-xs font-medium flex items-center space-x-1.5 transition-all active:scale-95 shadow-xs cursor-pointer hover:opacity-90"
                        style="
                          background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 80) : 'rgba(15, 23, 42, 0.8)'};
                          color: {currentTheme ? currentTheme.textColor : '#f1f5f9'};
                          border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 75) : '#334155'};
                        "
                        title={item.item_type === 'file' ? 'Belgeyi panoya kopyalamak için tıkla' : 'Kopyalamak için tıkla'}
                      >
                        <!-- Icon -->
                        <svg class="w-3.5 h-3.5 shrink-0" style="color: {currentTheme ? currentTheme.accentColor : '#fbbf24'};" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                          <path stroke-linecap="round" stroke-linejoin="round" d={getItemIcon(item)} />
                        </svg>

                        <!-- Title -->
                        <span class="max-w-[130px] truncate">{item.title}</span>

                        <!-- File badge if file -->
                        {#if item.item_type === 'file'}
                          <span
                            class="px-1 py-0.2 text-[9px] rounded font-mono"
                            style="
                              background-color: {currentTheme ? hexToRgba(currentTheme.accentColor, 15) : 'rgba(245, 158, 11, 0.2)'};
                              color: {currentTheme ? currentTheme.accentColor : '#fde68a'};
                            "
                          >
                            {formatBytes(item.file_size_bytes)}
                          </span>
                        {/if}

                        <!-- Star if pinned -->
                        {#if item.pinned}
                          <span class="text-[11px]" style="color: {currentTheme ? currentTheme.accentColor : '#fbbf24'};">★</span>
                        {/if}

                        <!-- Feedback tick -->
                        {#if copiedId === item.id}
                          <span class="text-[10px] text-emerald-400 font-bold">✓</span>
                        {/if}
                      </button>

                      <!-- Pill hover action trigger -->
                      <div
                        class="hidden group-hover:flex absolute -top-2.5 -right-2 rounded-md shadow-md items-center space-x-0.5 p-0.5 z-20"
                        style="
                          background-color: {currentTheme ? currentTheme.cardColor : '#1e293b'};
                          border: 1px solid {currentTheme ? currentTheme.borderColor : '#475569'};
                        "
                      >
                        {#if item.item_type === 'file'}
                          <button
                            onclick={(e) => { e.stopPropagation(); handleOpenFile(item); }}
                            class="p-0.5 rounded cursor-pointer hover:text-amber-400"
                            style="color: {currentTheme ? currentTheme.secondaryTextColor : '#cbd5e1'};"
                            title="Varsayılan Uygulama ile Aç"
                          >
                            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                              <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                              <path stroke-linecap="round" stroke-linejoin="round" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                            </svg>
                          </button>
                          <button
                            onclick={(e) => { e.stopPropagation(); handleExportFile(item); }}
                            class="p-0.5 rounded cursor-pointer hover:text-emerald-400"
                            style="color: {currentTheme ? currentTheme.secondaryTextColor : '#cbd5e1'};"
                            title="İndirilenler Klasörüne Kaydet"
                          >
                            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                              <path stroke-linecap="round" stroke-linejoin="round" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                            </svg>
                          </button>
                        {/if}
                        {#if item.item_type === 'link'}
                          <button
                            onclick={(e) => { e.stopPropagation(); handleOpenUrl(item.secret || item.title); }}
                            class="p-0.5 rounded cursor-pointer hover:text-sky-400"
                            style="color: {currentTheme ? currentTheme.secondaryTextColor : '#cbd5e1'};"
                            title="Tarayıcıda Aç"
                          >
                            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                              <path stroke-linecap="round" stroke-linejoin="round" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                            </svg>
                          </button>
                        {/if}
                        <button
                          onclick={(e) => { e.stopPropagation(); handleTogglePin(item.id); }}
                          class="p-0.5 rounded cursor-pointer hover:opacity-80"
                          style="color: {currentTheme ? (item.pinned ? currentTheme.accentColor : currentTheme.secondaryTextColor) : '#cbd5e1'};"
                          title={item.pinned ? 'Sabitlemeyi Kaldır' : 'Başa Sabitle'}
                        >
                          <svg class="w-3 h-3" fill={item.pinned ? 'currentColor' : 'none'} stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
                          </svg>
                        </button>
                        <button
                          onclick={(e) => { e.stopPropagation(); openEditModal(item); }}
                          class="p-0.5 rounded cursor-pointer hover:text-blue-400"
                          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#cbd5e1'};"
                          title="Düzenle"
                        >
                          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                          </svg>
                        </button>
                        <button
                          onclick={(e) => { e.stopPropagation(); handleDeleteItem(item.id); }}
                          class="p-0.5 rounded cursor-pointer hover:text-rose-400"
                          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#cbd5e1'};"
                          title="Sil"
                        >
                          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                          </svg>
                        </button>
                      </div>
                    </div>
                  {/each}
                </div>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>

<!-- ADD / EDIT MODAL -->
{#if isEditModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-2 overflow-hidden">
    <div
      class="rounded-2xl shadow-2xl w-full max-w-md flex flex-col p-3 overflow-hidden my-auto"
      style="
        max-height: calc(100% - 16px);
        background-color: {currentTheme ? currentTheme.cardColor : '#0f172a'};
        border: 1px solid {currentTheme ? currentTheme.borderColor : '#334155'};
        color: {currentTheme ? currentTheme.textColor : '#f1f5f9'};
      "
    >
      <div class="flex items-center justify-between pb-1.5 shrink-0" style="border-bottom: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : '#1e293b'};">
        <h3 class="text-sm font-semibold" style="color: {currentTheme ? currentTheme.textColor : '#ffffff'};">
          {isEditingExisting ? 'Kaydı Düzenle' : 'Yeni Bilgi / Dosya Ekle'}
        </h3>
        <button
          onclick={() => (isEditModalOpen = false)}
          class="p-1 cursor-pointer hover:opacity-80 rounded-lg"
          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};"
          title="Kapat"
          aria-label="Kapat"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      {#if errorMessage}
        <div class="mt-1 px-2.5 py-1.5 bg-rose-950/70 border border-rose-500/50 text-rose-300 text-xs rounded-lg shrink-0">
          {errorMessage}
        </div>
      {/if}

      <div class="space-y-2 text-xs overflow-y-auto flex-1 min-h-0 pr-1 py-1.5">
        <!-- Kayıt Türü: Segmented Pills -->
        <div>
          <span class="block mb-1 font-medium" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">Kayıt Türü</span>
          <div
            class="grid grid-cols-5 gap-1 p-1 rounded-xl"
            style="
              background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 90) : 'rgba(2, 6, 23, 0.9)'};
              border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : '#334155'};
            "
          >
            <button
              type="button"
              onclick={() => (formItemType = 'text')}
              class="py-1 px-1 rounded-lg text-xs font-semibold transition-all text-center flex items-center justify-center space-x-1 cursor-pointer"
              style="
                background-color: {formItemType === 'text' ? (currentTheme ? currentTheme.accentColor : '#f59e0b') : 'transparent'};
                color: {formItemType === 'text' ? '#ffffff' : (currentTheme ? currentTheme.secondaryTextColor : '#cbd5e1')};
              "
            >
              <span>📄</span>
              <span>Metin</span>
            </button>
            <button
              type="button"
              onclick={() => (formItemType = 'password')}
              class="py-1 px-1 rounded-lg text-xs font-semibold transition-all text-center flex items-center justify-center space-x-1 cursor-pointer"
              style="
                background-color: {formItemType === 'password' ? (currentTheme ? currentTheme.accentColor : '#f59e0b') : 'transparent'};
                color: {formItemType === 'password' ? '#ffffff' : (currentTheme ? currentTheme.secondaryTextColor : '#cbd5e1')};
              "
            >
              <span>🔒</span>
              <span>Şifre</span>
            </button>
            <button
              type="button"
              onclick={() => (formItemType = 'file')}
              class="py-1 px-1 rounded-lg text-xs font-semibold transition-all text-center flex items-center justify-center space-x-1 cursor-pointer"
              style="
                background-color: {formItemType === 'file' ? (currentTheme ? currentTheme.accentColor : '#f59e0b') : 'transparent'};
                color: {formItemType === 'file' ? '#ffffff' : (currentTheme ? currentTheme.secondaryTextColor : '#cbd5e1')};
              "
            >
              <span>📎</span>
              <span>Dosya</span>
            </button>
            <button
              type="button"
              onclick={() => (formItemType = 'note')}
              class="py-1 px-1 rounded-lg text-xs font-semibold transition-all text-center flex items-center justify-center space-x-1 cursor-pointer"
              style="
                background-color: {formItemType === 'note' ? (currentTheme ? currentTheme.accentColor : '#f59e0b') : 'transparent'};
                color: {formItemType === 'note' ? '#ffffff' : (currentTheme ? currentTheme.secondaryTextColor : '#cbd5e1')};
              "
            >
              <span>📝</span>
              <span>Not</span>
            </button>
            <button
              type="button"
              onclick={() => (formItemType = 'link')}
              class="py-1 px-1 rounded-lg text-xs font-semibold transition-all text-center flex items-center justify-center space-x-1 cursor-pointer"
              style="
                background-color: {formItemType === 'link' ? (currentTheme ? currentTheme.accentColor : '#f59e0b') : 'transparent'};
                color: {formItemType === 'link' ? '#ffffff' : (currentTheme ? currentTheme.secondaryTextColor : '#cbd5e1')};
              "
            >
              <span>🌐</span>
              <span>Link</span>
            </button>
          </div>
        </div>

        <!-- Tab & Group Row -->
        <div class="grid grid-cols-2 gap-2">
          <div>
            <label for="form-tab-select" class="block mb-0.5 font-medium" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">Sekme / Pano</label>
            <div class="relative">
              <select
                id="form-tab-select"
                bind:value={formTab}
                style="
                  background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 95) : '#020617'} !important;
                  color: {currentTheme ? currentTheme.textColor : '#f8fafc'} !important;
                  border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 75) : '#334155'};
                "
                class="w-full appearance-none pl-2.5 pr-7 py-1.5 rounded-xl focus:outline-none text-xs font-medium cursor-pointer shadow-inner"
              >
                {#each tabs as t}
                  <option value={t} style="background-color: {currentTheme ? currentTheme.cardColor : '#0f172a'}; color: {currentTheme ? currentTheme.textColor : '#ffffff'};">{t}</option>
                {/each}
              </select>
              <div class="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                </svg>
              </div>
            </div>
          </div>

          <div>
            <label for="form-group-input" class="block mb-0.5 font-medium" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">Grup / Kutu Adı</label>
            <input
              id="form-group-input"
              type="text"
              bind:value={formGroup}
              style="
                background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 95) : '#020617'} !important;
                color: {currentTheme ? currentTheme.textColor : '#f8fafc'} !important;
                border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 75) : '#334155'};
              "
              placeholder="Örn: Kişisel Bilgiler, Belgeler"
              class="w-full px-2.5 py-1.5 rounded-xl focus:outline-none text-xs shadow-inner"
            />
          </div>
        </div>

        <!-- Title Row -->
        <div>
          <label for="vault-form-title" class="block mb-0.5 font-medium" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">Başlık / Tanım</label>
          <input
            id="vault-form-title"
            type="text"
            bind:value={formTitle}
            style="
              background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 95) : '#020617'} !important;
              color: {currentTheme ? currentTheme.textColor : '#f8fafc'} !important;
              border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 75) : '#334155'};
            "
            placeholder="Örn: First and Last Name, Kimlik Tarama, Wi-Fi"
            class="w-full px-2.5 py-1.5 rounded-xl focus:outline-none text-xs shadow-inner"
          />
        </div>

        <!-- Conditional File Input or Secret Input -->
        {#if formItemType === 'file'}
          <div
            class="p-2 rounded-xl space-y-1.5"
            style="
              background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 60) : 'rgba(30, 41, 59, 0.8)'};
              border: 1px dashed {currentTheme ? hexToRgba(currentTheme.borderColor, 80) : '#475569'};
            "
          >
            {#if !(formFileName || formFilePath)}
              <!-- Native File Picker Button (when no file chosen yet) -->
              <div class="flex items-center justify-between">
                <span class="block font-medium text-xs" style="color: {currentTheme ? currentTheme.textColor : '#cbd5e1'};">
                  Belge / Dosya Seçimi (Maks. {settings.max_vault_file_size_mb || 20} MB):
                </span>
              </div>

              <button
                type="button"
                onclick={handlePickNativeFile}
                class="w-full py-2 px-3 rounded-xl text-xs font-semibold flex items-center justify-center gap-2 transition-all cursor-pointer shadow-xs active:scale-95"
                style="
                  background-color: {currentTheme ? hexToRgba(currentTheme.accentColor, 15) : 'rgba(245, 158, 11, 0.2)'};
                  color: {currentTheme ? currentTheme.accentColor : '#fbbf24'};
                  border: 1px solid {currentTheme ? hexToRgba(currentTheme.accentColor, 40) : 'rgba(245, 158, 11, 0.4)'};
                "
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                </svg>
                <span>📁 Sistemden Dosya Seç...</span>
              </button>
            {:else}
              <!-- Compact Selected File Header with Change Button -->
              <div
                class="p-2 rounded-xl text-xs space-y-1.5"
                style="
                  background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 90) : '#020617'};
                  border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : '#334155'};
                "
              >
                <div class="flex items-center justify-between gap-2">
                  <div class="flex items-center gap-1.5 min-w-0 flex-1">
                    <span class="text-sm shrink-0">📎</span>
                    <div class="min-w-0 flex-1">
                      <div class="font-semibold font-mono truncate text-xs" style="color: {currentTheme ? currentTheme.accentColor : '#fbbf24'};" title={formFileName || ''}>
                        {formFileName || 'Seçilen Dosya'}
                      </div>
                      <div class="text-[10px] opacity-75">
                        {formatBytes(formFileSizeBytes ?? undefined)}
                      </div>
                    </div>
                  </div>
                  <button
                    type="button"
                    onclick={handlePickNativeFile}
                    class="px-2 py-0.5 text-[11px] font-medium rounded-lg border transition-all cursor-pointer hover:opacity-85 shrink-0 flex items-center gap-1"
                    style="
                      border-color: {currentTheme ? hexToRgba(currentTheme.accentColor, 50) : '#f59e0b'};
                      color: {currentTheme ? currentTheme.accentColor : '#fbbf24'};
                      background-color: {currentTheme ? hexToRgba(currentTheme.accentColor, 15) : 'rgba(245, 158, 11, 0.15)'};
                    "
                    title="Farklı bir dosya seç"
                  >
                    <span>🔄 Değiştir</span>
                  </button>
                </div>

                {#if formFilePath}
                  <div class="text-[10px] truncate font-mono select-all px-1.5 py-0.5 rounded" style="background-color: {currentTheme ? hexToRgba(currentTheme.cardColor, 60) : '#0f172a'}; color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};" title={formFilePath}>
                    {formFilePath}
                  </div>
                {/if}

                <!-- Storage Mode Selector -->
                <div class="pt-1 border-t space-y-1" style="border-color: {currentTheme ? hexToRgba(currentTheme.borderColor, 50) : '#1e293b'};">
                  <span class="text-[10px] block font-medium" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">Depolama Yöntemi:</span>
                  <div class="grid grid-cols-2 gap-1.5">
                    <button
                      type="button"
                      onclick={() => (formCopyToVault = false)}
                      class="px-2 py-1 rounded-lg text-[11px] font-medium flex items-center justify-center gap-1 transition-all cursor-pointer"
                      style="
                        background-color: {!formCopyToVault
                          ? (currentTheme ? hexToRgba(currentTheme.accentColor, 25) : 'rgba(245, 158, 11, 0.25)')
                          : 'transparent'};
                        color: {!formCopyToVault
                          ? (currentTheme ? currentTheme.accentColor : '#fbbf24')
                          : (currentTheme ? currentTheme.secondaryTextColor : '#94a3b8')};
                        border: 1px solid {!formCopyToVault
                          ? (currentTheme ? currentTheme.accentColor : '#f59e0b')
                          : (currentTheme ? hexToRgba(currentTheme.borderColor, 50) : '#334155')};
                      "
                      title="Orijinal dosya yolunu referans alır, disk alanı kaplamaz."
                    >
                      <span>🔗 Kısayol</span>
                    </button>
                    <button
                      type="button"
                      onclick={() => (formCopyToVault = true)}
                      class="px-2 py-1 rounded-lg text-[11px] font-medium flex items-center justify-center gap-1 transition-all cursor-pointer"
                      style="
                        background-color: {formCopyToVault
                          ? (currentTheme ? hexToRgba(currentTheme.accentColor, 25) : 'rgba(245, 158, 11, 0.25)')
                          : 'transparent'};
                        color: {formCopyToVault
                          ? (currentTheme ? currentTheme.accentColor : '#fbbf24')
                          : (currentTheme ? currentTheme.secondaryTextColor : '#94a3b8')};
                        border: 1px solid {formCopyToVault
                          ? (currentTheme ? currentTheme.accentColor : '#f59e0b')
                          : (currentTheme ? hexToRgba(currentTheme.borderColor, 50) : '#334155')};
                      "
                      title="Dosyayı özel kasa klasörüne kopyalar. Orijinal silinse bile korunur."
                    >
                      <span>📦 Kasaya Kopyala</span>
                    </button>
                  </div>
                  <p class="text-[10px] leading-tight truncate" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#64748b'};">
                    {#if formCopyToVault}
                      ✓ Özel kasa klasörüne kopyalanır (Orijinal silinse de korunur)
                    {:else}
                      ✓ Orijinal dosya yolu referans alınır (Disk alanı harcamaz)
                    {/if}
                  </p>
                </div>
              </div>
            {/if}

            {#if fileError}
              <p class="text-[11px] text-rose-400">{fileError}</p>
            {/if}
          </div>
        {:else}
          <div>
            <label for="vault-form-secret" class="block mb-0.5 font-medium" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">Gizli Değer / İçerik</label>
            <textarea
              id="vault-form-secret"
              bind:value={formSecret}
              style="
                background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 95) : '#020617'} !important;
                color: {currentTheme ? currentTheme.textColor : '#f8fafc'} !important;
                border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 75) : '#334155'};
              "
              placeholder="Şifrenizi veya değerinizi buraya yazın..."
              rows="2"
              class="w-full px-2.5 py-1.5 font-mono rounded-xl focus:outline-none text-xs shadow-inner"
            ></textarea>
          </div>
        {/if}

        <!-- Pinned Checkbox -->
        <label class="flex items-center space-x-2 cursor-pointer pt-0.5">
          <input
            type="checkbox"
            bind:checked={formPinned}
            class="w-3.5 h-3.5 rounded cursor-pointer"
            style="accent-color: {currentTheme ? currentTheme.accentColor : '#f59e0b'};"
          />
          <span style="color: {currentTheme ? currentTheme.textColor : '#cbd5e1'};">En üstteki Sabitlenenler çubuğunda göster</span>
        </label>
      </div>

      <!-- Footer Buttons -->
      <div class="flex items-center justify-end space-x-2 pt-2 shrink-0" style="border-top: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : '#1e293b'};">
        <button
          onclick={() => (isEditModalOpen = false)}
          class="px-3 py-1.5 text-xs rounded-lg transition-colors cursor-pointer hover:opacity-80"
          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};"
        >
          İptal
        </button>
        <button
          onclick={handleSaveItem}
          class="px-4 py-1.5 text-xs font-bold rounded-lg shadow-sm transition-all active:scale-95 cursor-pointer"
          style="
            background-color: {currentTheme ? currentTheme.accentColor : '#f59e0b'};
            color: #ffffff;
          "
        >
          Kaydet
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- ADD TAB MODAL -->
{#if isAddTabModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4">
    <div
      class="rounded-2xl shadow-2xl w-full max-w-xs p-4 space-y-3"
      style="
        background-color: {currentTheme ? currentTheme.cardColor : '#0f172a'};
        border: 1px solid {currentTheme ? currentTheme.borderColor : '#334155'};
        color: {currentTheme ? currentTheme.textColor : '#f1f5f9'};
      "
    >
      <h3 class="text-sm font-semibold" style="color: {currentTheme ? currentTheme.textColor : '#ffffff'};">Yeni Sekme / Pano Ekle</h3>
      <input
        type="text"
        bind:value={newTabName}
        placeholder="Sekme Adı (Örn: AI, Finans)"
        class="w-full px-3 py-1.5 text-xs rounded-lg focus:outline-none"
        style="
          background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 95) : '#1e293b'};
          color: {currentTheme ? currentTheme.textColor : '#ffffff'};
          border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 75) : '#334155'};
        "
        onkeydown={(e) => e.key === 'Enter' && handleAddTab()}
      />
      <div class="flex justify-end space-x-2 pt-1">
        <button
          onclick={() => (isAddTabModalOpen = false)}
          class="px-3 py-1 text-xs rounded-lg cursor-pointer hover:opacity-80"
          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};"
        >
          İptal
        </button>
        <button
          onclick={handleAddTab}
          class="px-3.5 py-1 text-xs font-bold rounded-lg cursor-pointer"
          style="
            background-color: {currentTheme ? currentTheme.accentColor : '#f59e0b'};
            color: #ffffff;
          "
        >
          Ekle
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- FILE PATH MODAL -->
{#if showPathModal}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4">
    <div
      class="rounded-2xl shadow-2xl w-full max-w-md p-4 space-y-3"
      style="
        background-color: {currentTheme ? currentTheme.cardColor : '#0f172a'};
        border: 1px solid {currentTheme ? currentTheme.borderColor : '#334155'};
        color: {currentTheme ? currentTheme.textColor : '#f1f5f9'};
      "
    >
      <div class="flex items-center justify-between pb-2" style="border-bottom: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : '#1e293b'};">
        <h3 class="text-sm font-semibold flex items-center gap-1.5" style="color: {currentTheme ? currentTheme.textColor : '#ffffff'};">
          <span>🛡️</span>
          <span>Kasa Dosyası ve Şifreleme</span>
        </h3>
        <button
          onclick={() => (showPathModal = false)}
          class="p-1 cursor-pointer hover:opacity-80"
          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};"
          title="Kapat"
          aria-label="Kapat"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <p class="text-xs leading-relaxed" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">
        Kasanız ve eklediğiniz dosyalar <strong>AES-256-GCM</strong> ile şifrelenmiş olarak bilgisayarınızda saklanmaktadır.
      </p>

      <div
        class="p-2.5 rounded-lg text-xs font-mono break-all select-all"
        style="
          background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 90) : '#1e293b'};
          border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : '#334155'};
          color: {currentTheme ? currentTheme.accentColor : '#fbbf24'};
        "
      >
        {status?.vault_path}
      </div>

      <div
        class="p-3 rounded-xl space-y-2"
        style="
          background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 50) : 'rgba(30, 41, 59, 0.8)'};
          border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : '#334155'};
        "
      >
        <div class="flex items-center justify-between text-xs">
          <span style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">Şifreleme Modu:</span>
          {#if status?.has_pin}
            <span class="font-semibold flex items-center gap-1" style="color: {currentTheme ? currentTheme.accentColor : '#fbbf24'};">🔒 PIN Korumalı</span>
          {:else}
            <span class="font-semibold flex items-center gap-1 text-emerald-400">🛡️ Otomatik Cihaz Şifrelemesi</span>
          {/if}
        </div>
        <p class="text-[11px] leading-normal" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">
          {#if status?.has_pin}
            Kasanız belirlediğiniz PIN kodu ile korunur. Farklı bir cihaza taşındığında aynı PIN ile açılabilir.
          {:else}
            Kasanız bu bilgisayarda parola sormadan otomatik açılır. Dosyalarınız diskte 256-bit AES ile şifreli kalır.
          {/if}
        </p>
        <div class="pt-1 flex justify-end">
          {#if status?.has_pin}
            <button
              onclick={() => { showPathModal = false; isRemovePinConfirmOpen = true; }}
              class="px-2.5 py-1 text-xs rounded-md transition-colors cursor-pointer hover:opacity-90"
              style="
                background-color: {currentTheme ? hexToRgba(currentTheme.cardColor, 90) : '#334155'};
                color: {currentTheme ? currentTheme.textColor : '#ffffff'};
                border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : '#475569'};
              "
            >
              PIN Korumasını Kaldır (Otomatik Açılışa Geç)
            </button>
          {:else}
            <button
              onclick={() => { showPathModal = false; newPin = ''; confirmNewPin = ''; pinModalError = ''; isPinModalOpen = true; }}
              class="px-2.5 py-1 text-xs font-bold rounded-md transition-colors cursor-pointer"
              style="
                background-color: {currentTheme ? currentTheme.accentColor : '#f59e0b'};
                color: #ffffff;
              "
            >
              + PIN Koruması Ekle
            </button>
          {/if}
        </div>
      </div>

      <div class="flex items-center justify-between pt-1">
        <button
          onclick={() => { showPathModal = false; isBackupModalOpen = true; }}
          class="px-2.5 py-1 text-xs rounded-lg transition-colors cursor-pointer flex items-center gap-1.5"
          style="
            background-color: {currentTheme ? hexToRgba(currentTheme.accentColor, 15) : 'rgba(245, 158, 11, 0.2)'};
            color: {currentTheme ? currentTheme.accentColor : '#fbbf24'};
            border: 1px solid {currentTheme ? hexToRgba(currentTheme.accentColor, 35) : 'rgba(245, 158, 11, 0.4)'};
          "
        >
          <span>📦</span>
          <span>Yedekleme & Geri Yükleme</span>
        </button>

        <button
          onclick={() => (showPathModal = false)}
          class="px-3 py-1.5 text-xs rounded-lg transition-colors cursor-pointer hover:opacity-80"
          style="
            background-color: {currentTheme ? hexToRgba(currentTheme.cardColor, 90) : '#1e293b'};
            color: {currentTheme ? currentTheme.textColor : '#ffffff'};
            border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : '#334155'};
          "
        >
          Kapat
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- PIN MODAL: Set / Change Master PIN -->
{#if isPinModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4 animate-in fade-in duration-150">
    <div
      class="rounded-2xl shadow-2xl w-full max-w-sm p-4 space-y-3.5"
      style="
        background-color: {currentTheme ? currentTheme.cardColor : '#0f172a'};
        border: 1px solid {currentTheme ? currentTheme.borderColor : '#334155'};
        color: {currentTheme ? currentTheme.textColor : '#f1f5f9'};
      "
    >
      <div class="flex items-center justify-between pb-2" style="border-bottom: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : '#1e293b'};">
        <h3 class="text-sm font-semibold flex items-center gap-1.5" style="color: {currentTheme ? currentTheme.textColor : '#ffffff'};">
          <span>🔒</span>
          <span>{status?.has_pin ? 'PIN Kodunu Değiştir' : 'PIN Koruması Ekle'}</span>
        </h3>
        <button
          onclick={() => (isPinModalOpen = false)}
          class="p-1 cursor-pointer hover:opacity-80"
          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};"
          title="Kapat"
        >
          ✕
        </button>
      </div>

      <p class="text-xs leading-relaxed" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">
        Belirleyeceğiniz PIN kodu ile kasanız ve ekli tüm dosyalar şifrelenir. Her oturumda kasayı açmak için bu PIN sorulur.
      </p>

      {#if pinModalError}
        <div class="px-3 py-1.5 bg-rose-950/60 border border-rose-500/50 text-rose-300 text-xs rounded-lg">
          {pinModalError}
        </div>
      {/if}

      <div class="space-y-2.5">
        <div>
          <label class="block text-[11px] font-medium mb-1" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#cbd5e1'};" for="new-pin-input">Yeni PIN (en az 4 hane):</label>
          <input
            id="new-pin-input"
            type="password"
            bind:value={newPin}
            placeholder="****"
            class="w-full px-3 py-1.5 text-sm rounded-lg focus:outline-none font-mono tracking-widest text-center"
            style="
              background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 95) : '#1e293b'};
              color: {currentTheme ? currentTheme.textColor : '#ffffff'};
              border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 75) : '#334155'};
            "
            autocomplete="new-password"
          />
        </div>
        <div>
          <label class="block text-[11px] font-medium mb-1" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#cbd5e1'};" for="confirm-new-pin-input">PIN'i Doğrulayın:</label>
          <input
            id="confirm-new-pin-input"
            type="password"
            bind:value={confirmNewPin}
            placeholder="****"
            class="w-full px-3 py-1.5 text-sm rounded-lg focus:outline-none font-mono tracking-widest text-center"
            style="
              background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 95) : '#1e293b'};
              color: {currentTheme ? currentTheme.textColor : '#ffffff'};
              border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 75) : '#334155'};
            "
            autocomplete="new-password"
            onkeydown={(e) => e.key === 'Enter' && handleSetPin()}
          />
        </div>
      </div>

      <div class="flex justify-end space-x-2 pt-2" style="border-top: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : '#1e293b'};">
        <button
          onclick={() => (isPinModalOpen = false)}
          class="px-3 py-1.5 text-xs rounded-lg cursor-pointer hover:opacity-80"
          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};"
        >
          İptal
        </button>
        <button
          onclick={handleSetPin}
          class="px-3.5 py-1.5 text-xs font-bold rounded-lg shadow-sm cursor-pointer"
          style="
            background-color: {currentTheme ? currentTheme.accentColor : '#f59e0b'};
            color: #ffffff;
          "
        >
          PIN'i Kaydet
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- REMOVE PIN CONFIRMATION MODAL -->
{#if isRemovePinConfirmOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4 animate-in fade-in duration-150">
    <div
      class="rounded-2xl shadow-2xl w-full max-w-sm p-4 space-y-3.5"
      style="
        background-color: {currentTheme ? currentTheme.cardColor : '#0f172a'};
        border: 1px solid {currentTheme ? currentTheme.borderColor : '#334155'};
        color: {currentTheme ? currentTheme.textColor : '#f1f5f9'};
      "
    >
      <div class="flex items-center justify-between pb-2" style="border-bottom: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : '#1e293b'};">
        <h3 class="text-sm font-semibold flex items-center gap-1.5" style="color: {currentTheme ? currentTheme.textColor : '#ffffff'};">
          <span>🔓</span>
          <span>PIN Korumasını Kaldır</span>
        </h3>
        <button
          onclick={() => (isRemovePinConfirmOpen = false)}
          class="p-1 cursor-pointer hover:opacity-80"
          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};"
          title="Kapat"
        >
          ✕
        </button>
      </div>

      <p class="text-xs leading-relaxed" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#cbd5e1'};">
        PIN korumasını kaldırdığınızda kasanız <strong>Otomatik Cihaz Şifrelemesi</strong> moduna geçer.
      </p>
      <div
        class="p-2.5 rounded-lg text-[11px] space-y-1.5"
        style="
          background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 60) : 'rgba(30, 41, 59, 0.8)'};
          color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};
        "
      >
        <div class="text-emerald-400 font-medium">✓ Verileriniz diskte AES-256-GCM ile şifreli kalmaya devam eder.</div>
        <div>✓ Bu bilgisayarda kasayı açarken artık PIN sorulmaz.</div>
      </div>

      <div class="flex justify-end space-x-2 pt-2" style="border-top: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : '#1e293b'};">
        <button
          onclick={() => (isRemovePinConfirmOpen = false)}
          class="px-3 py-1.5 text-xs rounded-lg cursor-pointer hover:opacity-80"
          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};"
        >
          Vazgeç
        </button>
        <button
          onclick={handleRemovePin}
          class="px-3.5 py-1.5 text-xs font-bold rounded-lg shadow-sm cursor-pointer"
          style="
            background-color: {currentTheme ? currentTheme.accentColor : '#f59e0b'};
            color: #ffffff;
          "
        >
          PIN'i Kaldır
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- BACKUP & RESTORE MODAL -->
{#if isBackupModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4 animate-in fade-in duration-150">
    <div
      class="rounded-2xl shadow-2xl w-full max-w-md p-4 space-y-3.5"
      style="
        background-color: {currentTheme ? currentTheme.cardColor : '#0f172a'};
        border: 1px solid {currentTheme ? currentTheme.borderColor : '#334155'};
        color: {currentTheme ? currentTheme.textColor : '#f1f5f9'};
      "
    >
      <div class="flex items-center justify-between pb-2" style="border-bottom: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : '#1e293b'};">
        <h3 class="text-sm font-semibold flex items-center gap-1.5" style="color: {currentTheme ? currentTheme.textColor : '#ffffff'};">
          <span>📦</span>
          <span>Kasa Tam Yedekleme & Geri Yükleme</span>
        </h3>
        <button
          onclick={() => (isBackupModalOpen = false)}
          class="p-1 cursor-pointer hover:opacity-80"
          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};"
          title="Kapat"
        >
          ✕
        </button>
      </div>

      <p class="text-xs leading-relaxed" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#cbd5e1'};">
        Kasanızın tüm şifreleri, notları, sekmeleri ve ekli belgeleri tek bir şifreli yedek paketi (<strong>.vaultbak</strong>) olarak dışa aktarılabilir veya başka bir bilgisayardan geri yüklenebilir.
      </p>

      {#if backupStatusMessage}
        <div class="p-2.5 rounded-lg text-xs bg-emerald-950/80 border border-emerald-500/50 text-emerald-300 whitespace-pre-line font-mono break-all">
          {backupStatusMessage}
        </div>
      {/if}

      {#if backupErrorMessage}
        <div class="p-2.5 rounded-lg text-xs bg-rose-950/80 border border-rose-500/50 text-rose-300 whitespace-pre-line break-all">
          {backupErrorMessage}
        </div>
      {/if}

      <!-- Actions Grid -->
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5 pt-1">
        <!-- Export Button -->
        <button
          onclick={handleExportBackup}
          disabled={isBackingUp}
          class="p-3 rounded-xl border text-left flex flex-col justify-between transition-all hover:brightness-110 active:scale-[0.98] cursor-pointer disabled:opacity-50"
          style="
            background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 70) : 'rgba(30, 41, 59, 0.7)'};
            border-color: {currentTheme ? hexToRgba(currentTheme.borderColor, 80) : '#334155'};
          "
        >
          <div class="flex items-center gap-2 mb-1.5">
            <span class="text-base">📦</span>
            <span class="font-semibold text-xs" style="color: {currentTheme ? currentTheme.accentColor : '#fbbf24'};">Yedek Al</span>
          </div>
          <span class="text-[11px] leading-snug" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">
            Tüm kasayı .vaultbak dosyası olarak dışa aktarır.
          </span>
        </button>

        <!-- Restore Button -->
        <button
          onclick={handleRestoreBackup}
          disabled={isBackingUp}
          class="p-3 rounded-xl border text-left flex flex-col justify-between transition-all hover:brightness-110 active:scale-[0.98] cursor-pointer disabled:opacity-50"
          style="
            background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 70) : 'rgba(30, 41, 59, 0.7)'};
            border-color: {currentTheme ? hexToRgba(currentTheme.borderColor, 80) : '#334155'};
          "
        >
          <div class="flex items-center gap-2 mb-1.5">
            <span class="text-base">🔄</span>
            <span class="font-semibold text-xs" style="color: {currentTheme ? currentTheme.accentColor : '#fbbf24'};">Geri Yükle</span>
          </div>
          <span class="text-[11px] leading-snug" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">
            Daha önce aldığınız .vaultbak yedeğini yükler.
          </span>
        </button>
      </div>

      <div class="flex justify-end pt-2" style="border-top: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : '#1e293b'};">
        <button
          onclick={() => (isBackupModalOpen = false)}
          class="px-3.5 py-1.5 text-xs rounded-lg cursor-pointer hover:opacity-80"
          style="
            background-color: {currentTheme ? hexToRgba(currentTheme.cardColor, 90) : '#1e293b'};
            color: {currentTheme ? currentTheme.textColor : '#ffffff'};
            border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : '#334155'};
          "
        >
          Kapat
        </button>
      </div>
    </div>
  </div>
{/if}
