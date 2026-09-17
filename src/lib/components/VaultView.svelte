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
      const g = item.group || 'General';
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
          activeTab = tabs[0] || 'Personal Info';
        }
      } else {
        items = [];
        tabs = [];
      }
    } catch (err: any) {
      errorMessage = typeof err === 'string' ? err : 'Failed to fetch vault status';
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
      errorMessage = typeof err === 'string' ? err : 'Failed to create vault.';
    }
  }

  async function handleInitPinVault() {
    errorMessage = '';
    if (pinInput.trim().length < 4) {
      errorMessage = 'PIN code must be at least 4 characters.';
      return;
    }
    if (pinInput !== confirmPinInput) {
      errorMessage = 'Entered PIN codes do not match!';
      return;
    }

    try {
      await invoke('init_vault', { pin: pinInput.trim() });
      pinInput = '';
      confirmPinInput = '';
      await refreshStatus();
    } catch (err: any) {
      errorMessage = typeof err === 'string' ? err : 'Failed to create vault.';
    }
  }

  async function handleSetPin() {
    pinModalError = '';
    if (newPin.trim().length < 4) {
      pinModalError = 'PIN code must be at least 4 characters.';
      return;
    }
    if (newPin !== confirmNewPin) {
      pinModalError = 'Entered PIN codes do not match!';
      return;
    }

    try {
      await invoke('set_vault_pin', { pin: newPin.trim() });
      newPin = '';
      confirmNewPin = '';
      isPinModalOpen = false;
      successMessage = 'PIN protection enabled successfully.';
      setTimeout(() => { successMessage = ''; }, 3000);
      await refreshStatus();
    } catch (err: any) {
      pinModalError = typeof err === 'string' ? err : 'Failed to set PIN.';
    }
  }

  async function handleRemovePin() {
    try {
      await invoke('remove_vault_pin');
      isRemovePinConfirmOpen = false;
      successMessage = 'PIN protection removed. Vault will now open with automatic device encryption.';
      setTimeout(() => { successMessage = ''; }, 3500);
      await refreshStatus();
    } catch (err: any) {
      alert(typeof err === 'string' ? err : 'Failed to remove PIN.');
    }
  }

  async function handleUnlockVault() {
    errorMessage = '';
    if (!pinInput.trim()) {
      errorMessage = 'Please enter your PIN code.';
      return;
    }

    try {
      await invoke<VaultItem[]>('unlock_vault', { pin: pinInput });
      pinInput = '';
      await refreshStatus();
    } catch (err: any) {
      errorMessage = typeof err === 'string' ? err : 'Incorrect PIN code! Failed to unlock vault.';
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
      alert(typeof err === 'string' ? err : 'Failed to add tab');
    }
  }

  async function handleDeleteTab(tabName: string) {
    if (tabs.length <= 1) {
      alert('At least one tab must remain.');
      return;
    }
    if (!confirm(`Are you sure you want to delete the "${tabName}" tab and its contents?`)) {
      return;
    }
    try {
      tabs = await invoke<string[]>('delete_vault_tab', { name: tabName });
      if (activeTab === tabName) {
        activeTab = tabs[0];
      }
    } catch (err: any) {
      alert(typeof err === 'string' ? err : 'Failed to delete tab');
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

  function openAddModal(defaultGroup = 'Personal Data') {
    isEditingExisting = false;
    currentEditId = null;
    formTitle = '';
    formSecret = '';
    formItemType = 'text';
    formTab = activeTab || (tabs[0] || 'Personal Info');
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
    formGroup = item.group || 'General';
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
          fileError = `File size exceeds ${settings.max_vault_file_size_mb || 20} MB limit! (${(picked.size_bytes / 1048576).toFixed(1)} MB)`;
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
      fileError = typeof err === 'string' ? err : 'Failed to open file picker.';
    }
  }

  function handleFileSelected(e: Event) {
    fileError = '';
    const input = e.target as HTMLInputElement;
    if (!input.files || input.files.length === 0) return;
    const file = input.files[0];

    const maxBytes = (settings.max_vault_file_size_mb || 20) * 1024 * 1024;
    if (file.size > maxBytes) {
      fileError = `File size exceeds ${settings.max_vault_file_size_mb || 20} MB limit! (${(file.size / 1048576).toFixed(1)} MB)`;
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
      fileError = 'Failed to read file.';
    };
    reader.readAsDataURL(file);
  }

  async function handleSaveItem() {
    if (!formTitle.trim()) {
      errorMessage = 'Please enter a title.';
      return;
    }
    if (formItemType !== 'file' && !formSecret.trim()) {
      errorMessage = 'Please enter the secret / value.';
      return;
    }
    if (formItemType === 'file' && !formFileName && !formFilePath && !isEditingExisting) {
      errorMessage = 'Please select a file.';
      return;
    }

    try {
      await invoke('save_vault_item', {
        id: currentEditId,
        title: formTitle.trim(),
        secret: formSecret.trim(),
        itemType: formItemType,
        tab: formTab || activeTab,
        group: formGroup.trim() || 'General',
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
      errorMessage = typeof err === 'string' ? err : 'Failed to save item.';
    }
  }

  async function handleDeleteItem(id: string) {
    if (!confirm('Are you sure you want to permanently delete this item?')) return;
    try {
      await invoke('delete_vault_item', { id });
      await refreshStatus();
    } catch (err: any) {
      alert(typeof err === 'string' ? err : 'Failed to delete item.');
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
      alert(typeof err === 'string' ? err : 'Copy failed');
    }
  }

  async function handleCopyFile(item: VaultItem) {
    try {
      await invoke('copy_vault_file', { id: item.id });
      copiedId = item.id;
      successMessage = `"${item.title || item.file_name || 'Document'}" copied to clipboard.`;
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
      alert(typeof err === 'string' ? err : 'Failed to copy file to clipboard');
    }
  }

  async function handleExportFile(item: VaultItem) {
    try {
      const savedPath = await invoke<string>('export_vault_file', { id: item.id });
      successMessage = `"${item.file_name || item.title}" saved to Downloads: ${savedPath}`;
      setTimeout(() => {
        successMessage = '';
      }, 4000);
    } catch (err: any) {
      alert(typeof err === 'string' ? err : 'Failed to export file');
    }
  }

  async function handleOpenFile(item: VaultItem) {
    try {
      await invoke('open_vault_file', { id: item.id });
    } catch (err: any) {
      alert(typeof err === 'string' ? err : 'Failed to open file');
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
      alert(typeof err === 'string' ? err : 'Failed to open link');
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
      backupStatusMessage = `Vault backup saved successfully:\n${savedPath}`;
      successMessage = 'Vault backup created successfully.';
      setTimeout(() => {
        if (successMessage.includes('Vault backup')) successMessage = '';
      }, 4000);
    } catch (err: any) {
      backupErrorMessage = typeof err === 'string' ? err : 'Backup failed';
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
      successMessage = 'Vault backup restored successfully!';
      await refreshStatus();
      setTimeout(() => {
        if (successMessage.includes('restored')) successMessage = '';
      }, 4000);
    } catch (err: any) {
      backupErrorMessage = typeof err === 'string' ? err : 'Restore failed';
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

      <h2 class="text-base font-semibold" style="color: {currentTheme ? currentTheme.textColor : '#ffffff'};">Secure Vault Setup</h2>
      <p class="text-xs mt-1 mb-3.5 leading-relaxed" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">
        Your passwords and files are protected with <strong>AES-256-GCM</strong> encryption. Choose your preferred startup mode:
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
              <span>🛡️</span> Automatic Device Encryption
            </span>
            <span
              class="px-1.5 py-0.5 text-[10px] rounded font-medium"
              style="
                background-color: {currentTheme ? hexToRgba(currentTheme.accentColor, 20) : 'rgba(245, 158, 11, 0.2)'};
                color: {currentTheme ? currentTheme.accentColor : '#fbbf24'};
                border: 1px solid {currentTheme ? hexToRgba(currentTheme.accentColor, 30) : 'rgba(245, 158, 11, 0.3)'};
              "
            >
              Recommended
            </span>
          </div>
          <p class="text-[11px] leading-snug" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">
            Opens instantly without asking for a PIN. Data is encrypted on disk with a machine-bound 256-bit AES key.
          </p>
          <button
            onclick={handleInitAutoVault}
            class="mt-2.5 w-full py-1.5 font-bold text-xs rounded-lg shadow-sm transition-all active:scale-[0.99] flex items-center justify-center gap-1.5 cursor-pointer"
            style="
              background-color: {currentTheme ? currentTheme.accentColor : '#f59e0b'};
              color: #ffffff;
            "
          >
            <span>🚀 Start Instantly (No PIN / Automatic)</span>
          </button>
        </div>

        <!-- Divider -->
        <div class="flex items-center gap-2 text-[11px] px-1 py-0.5" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#64748b'};">
          <div class="flex-1 h-px" style="background-color: {currentTheme ? hexToRgba(currentTheme.borderColor, 40) : '#334155'};"></div>
          <span>or</span>
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
            <span>🔒</span> Set Master PIN Protection (Optional)
          </span>
          <p class="text-[11px] mb-2" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">
            Requires a master PIN code to unlock the vault each session.
          </p>

          <div class="space-y-2">
            <input
              type="password"
              bind:value={pinInput}
              placeholder="Create Master PIN (at least 4 digits)"
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
              placeholder="Confirm Master PIN"
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
              Protect with PIN & Start
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

      <h2 class="text-base font-semibold" style="color: {currentTheme ? currentTheme.textColor : '#ffffff'};">Vault Locked</h2>
      <p class="text-xs mt-1 mb-4" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">
        Enter your master PIN code for this session. The vault stays unlocked until closed.
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
          placeholder="Enter Master PIN..."
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
          Unlock Vault
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
              title={item.item_type === 'file' ? 'Click to copy document to clipboard' : 'Click to copy'}
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
          <span class="font-semibold">{pinnedItems.length} Pinned</span>
        </div>
      </div>
    {/if}

    <!-- 1. CATEGORY / TAB BAR -->
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
                title="Delete Tab"
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
          title="Scroll Left"
        >
          ‹
        </button>
        <button
          onclick={() => scrollTabs('right')}
          class="w-6 h-6 flex items-center justify-center rounded text-xs transition-colors cursor-pointer hover:opacity-80"
          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};"
          title="Scroll Right"
        >
          ›
        </button>
      </div>

      <!-- Always visible + New Tab Button -->
      <button
        onclick={() => (isAddTabModalOpen = true)}
        class="shrink-0 px-2.5 py-1.5 rounded-lg text-xs font-bold flex items-center gap-1 transition-all cursor-pointer hover:opacity-90 shadow-xs"
        style="
          background-color: {currentTheme ? hexToRgba(currentTheme.cardColor, 90) : 'rgba(30, 41, 59, 0.9)'};
          color: {currentTheme ? currentTheme.accentColor : '#fbbf24'};
          border: 1px solid {currentTheme ? hexToRgba(currentTheme.accentColor, 40) : 'rgba(245, 158, 11, 0.3)'};
        "
        title="Add New Category / Tab"
      >
        <span class="text-sm font-black leading-none">+</span>
        <span class="text-[11px] font-medium">New</span>
      </button>
    </div>

    <!-- 2. SEARCH BAR & ACTION BUTTONS -->
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
          placeholder="Search in vault... (secret or title)"
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
            title="Clear Search"
          >
            ✕
          </button>
        {/if}
      </div>

      <!-- Action Buttons -->
      <div class="flex items-center space-x-1.5 shrink-0">
        <!-- + Add Button -->
        <button
          onclick={() => openAddModal()}
          class="px-3 py-1.5 font-bold rounded-xl text-xs flex items-center space-x-1 shadow-sm transition-all active:scale-95 whitespace-nowrap cursor-pointer shrink-0"
          style="
            background-color: {currentTheme ? currentTheme.accentColor : '#f59e0b'};
            color: #ffffff;
          "
          title="Add New Item / File"
        >
          <svg class="w-3.5 h-3.5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2.5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 4v16m8-8H4" />
          </svg>
          <span>Add</span>
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
            title="PIN-protected vault. Click to switch to automatic device encryption."
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
            title="Automatic device encryption. Click to add optional PIN protection."
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
          title="Vault Storage & Portability"
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
          title="Full Vault Backup & Restore (.vaultbak)"
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
          title="Lock Vault"
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
          <p class="text-sm font-medium" style="color: {currentTheme ? currentTheme.textColor : '#cbd5e1'};">No records in this tab yet</p>
          <p class="text-xs mt-1" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#64748b'};">
            Use the "+ Add" button above to store passwords, credentials, notes, or documents up to {settings.max_vault_file_size_mb || 20} MB.
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
                    title="Add new to this group"
                  >
                    + Add
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
                        title={item.item_type === 'file' ? 'Click to copy document to clipboard' : 'Click to copy'}
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
                            title="Open with Default App"
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
                            title="Save to Downloads"
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
                            title="Open in Browser"
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
                          title={item.pinned ? 'Unpin' : 'Pin to Top'}
                        >
                          <svg class="w-3 h-3" fill={item.pinned ? 'currentColor' : 'none'} stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
                          </svg>
                        </button>
                        <button
                          onclick={(e) => { e.stopPropagation(); openEditModal(item); }}
                          class="p-0.5 rounded cursor-pointer hover:text-blue-400"
                          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#cbd5e1'};"
                          title="Edit"
                        >
                          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                          </svg>
                        </button>
                        <button
                          onclick={(e) => { e.stopPropagation(); handleDeleteItem(item.id); }}
                          class="p-0.5 rounded cursor-pointer hover:text-rose-400"
                          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#cbd5e1'};"
                          title="Delete"
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
          {isEditingExisting ? 'Edit Item' : 'Add New Item / File'}
        </h3>
        <button
          onclick={() => (isEditModalOpen = false)}
          class="p-1 cursor-pointer hover:opacity-80 rounded-lg"
          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};"
          title="Close"
          aria-label="Close"
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
        <!-- Item Type: Segmented Pills -->
        <div>
          <span class="block mb-1 font-medium" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">Item Type</span>
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
              <span>Text</span>
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
              <span>Password</span>
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
              <span>File</span>
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
              <span>Note</span>
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
            <label for="form-tab-select" class="block mb-0.5 font-medium" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">Tab / Board</label>
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
            <label for="form-group-input" class="block mb-0.5 font-medium" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">Group / Box Name</label>
            <input
              id="form-group-input"
              type="text"
              bind:value={formGroup}
              style="
                background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 95) : '#020617'} !important;
                color: {currentTheme ? currentTheme.textColor : '#f8fafc'} !important;
                border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 75) : '#334155'};
              "
              placeholder="e.g. Personal Info, Documents, Work"
              class="w-full px-2.5 py-1.5 rounded-xl focus:outline-none text-xs shadow-inner"
            />
          </div>
        </div>

        <!-- Title Row -->
        <div>
          <label for="vault-form-title" class="block mb-0.5 font-medium" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">Title / Description</label>
          <input
            id="vault-form-title"
            type="text"
            bind:value={formTitle}
            style="
              background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 95) : '#020617'} !important;
              color: {currentTheme ? currentTheme.textColor : '#f8fafc'} !important;
              border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 75) : '#334155'};
            "
            placeholder="e.g. Full Name, ID Scan, Wi-Fi Password"
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
                  Document / File Selection (Max {settings.max_vault_file_size_mb || 20} MB):
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
                <span>📁 Select File from System...</span>
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
                        {formFileName || 'Selected File'}
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
                    title="Select a different file"
                  >
                    <span>🔄 Change</span>
                  </button>
                </div>

                {#if formFilePath}
                  <div class="text-[10px] truncate font-mono select-all px-1.5 py-0.5 rounded" style="background-color: {currentTheme ? hexToRgba(currentTheme.cardColor, 60) : '#0f172a'}; color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};" title={formFilePath}>
                    {formFilePath}
                  </div>
                {/if}

                <!-- Storage Mode Selector -->
                <div class="pt-1 border-t space-y-1" style="border-color: {currentTheme ? hexToRgba(currentTheme.borderColor, 50) : '#1e293b'};">
                  <span class="text-[10px] block font-medium" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">Storage Method:</span>
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
                      title="References original file path without duplicating disk space."
                    >
                      <span>🔗 Shortcut</span>
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
                      title="Copies the file into the secure vault directory. Preserved even if original is deleted."
                    >
                      <span>📦 Copy to Vault</span>
                    </button>
                  </div>
                  <p class="text-[10px] leading-tight truncate" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#64748b'};">
                    {#if formCopyToVault}
                      ✓ Copied to secure vault directory (preserved even if original is deleted)
                    {:else}
                      ✓ References original file path (saves disk space)
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
            <label for="vault-form-secret" class="block mb-0.5 font-medium" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">Secret Value / Content</label>
            <textarea
              id="vault-form-secret"
              bind:value={formSecret}
              style="
                background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 95) : '#020617'} !important;
                color: {currentTheme ? currentTheme.textColor : '#f8fafc'} !important;
                border: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 75) : '#334155'};
              "
              placeholder="Enter your secret or content here..."
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
          <span style="color: {currentTheme ? currentTheme.textColor : '#cbd5e1'};">Show in top Pinned bar</span>
        </label>
      </div>

      <!-- Footer Buttons -->
      <div class="flex items-center justify-end space-x-2 pt-2 shrink-0" style="border-top: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : '#1e293b'};">
        <button
          onclick={() => (isEditModalOpen = false)}
          class="px-3 py-1.5 text-xs rounded-lg transition-colors cursor-pointer hover:opacity-80"
          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};"
        >
          Cancel
        </button>
        <button
          onclick={handleSaveItem}
          class="px-4 py-1.5 text-xs font-bold rounded-lg shadow-sm transition-all active:scale-95 cursor-pointer"
          style="
            background-color: {currentTheme ? currentTheme.accentColor : '#f59e0b'};
            color: #ffffff;
          "
        >
          Save
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
      <h3 class="text-sm font-semibold" style="color: {currentTheme ? currentTheme.textColor : '#ffffff'};">Add New Tab / Category</h3>
      <input
        type="text"
        bind:value={newTabName}
        placeholder="Tab Name (e.g. AI, Finance, Docs)"
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
          Cancel
        </button>
        <button
          onclick={handleAddTab}
          class="px-3.5 py-1 text-xs font-bold rounded-lg cursor-pointer"
          style="
            background-color: {currentTheme ? currentTheme.accentColor : '#f59e0b'};
            color: #ffffff;
          "
        >
          Add
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
          <span>Vault Storage & Encryption</span>
        </h3>
        <button
          onclick={() => (showPathModal = false)}
          class="p-1 cursor-pointer hover:opacity-80"
          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};"
          title="Close"
          aria-label="Close"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <p class="text-xs leading-relaxed" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">
        Your vault and attached documents are stored locally on your system, encrypted with <strong>AES-256-GCM</strong>.
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
          <span style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">Encryption Mode:</span>
          {#if status?.has_pin}
            <span class="font-semibold flex items-center gap-1" style="color: {currentTheme ? currentTheme.accentColor : '#fbbf24'};">🔒 PIN Protected</span>
          {:else}
            <span class="font-semibold flex items-center gap-1 text-emerald-400">🛡️ Automatic Device Encryption</span>
          {/if}
        </div>
        <p class="text-[11px] leading-normal" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">
          {#if status?.has_pin}
            Your vault is protected by your master PIN. If transferred to another device, it can be unlocked using the same PIN.
          {:else}
            Your vault unlocks automatically without prompts on this device. Your data remains encrypted with 256-bit AES on disk.
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
              Remove PIN Protection (Switch to Auto-Unlock)
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
              + Add PIN Protection
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
          <span>Backup & Restore</span>
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
          Close
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
          <span>{status?.has_pin ? 'Change Master PIN' : 'Add PIN Protection'}</span>
        </h3>
        <button
          onclick={() => (isPinModalOpen = false)}
          class="p-1 cursor-pointer hover:opacity-80"
          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};"
          title="Close"
        >
          ✕
        </button>
      </div>

      <p class="text-xs leading-relaxed" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">
        Your vault and attached documents will be encrypted with your chosen master PIN. You will be prompted for this PIN in each session.
      </p>

      {#if pinModalError}
        <div class="px-3 py-1.5 bg-rose-950/60 border border-rose-500/50 text-rose-300 text-xs rounded-lg">
          {pinModalError}
        </div>
      {/if}

      <div class="space-y-2.5">
        <div>
          <label class="block text-[11px] font-medium mb-1" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#cbd5e1'};" for="new-pin-input">New PIN (at least 4 digits):</label>
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
          <label class="block text-[11px] font-medium mb-1" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#cbd5e1'};" for="confirm-new-pin-input">Confirm PIN:</label>
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
          Cancel
        </button>
        <button
          onclick={handleSetPin}
          class="px-3.5 py-1.5 text-xs font-bold rounded-lg shadow-sm cursor-pointer"
          style="
            background-color: {currentTheme ? currentTheme.accentColor : '#f59e0b'};
            color: #ffffff;
          "
        >
          Save PIN
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
          <span>Remove PIN Protection</span>
        </h3>
        <button
          onclick={() => (isRemovePinConfirmOpen = false)}
          class="p-1 cursor-pointer hover:opacity-80"
          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};"
          title="Close"
        >
          ✕
        </button>
      </div>

      <p class="text-xs leading-relaxed" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#cbd5e1'};">
        When you remove PIN protection, your vault switches to <strong>Automatic Device Encryption</strong> mode.
      </p>
      <div
        class="p-2.5 rounded-lg text-[11px] space-y-1.5"
        style="
          background-color: {currentTheme ? hexToRgba(currentTheme.bgColor, 60) : 'rgba(30, 41, 59, 0.8)'};
          color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};
        "
      >
        <div class="text-emerald-400 font-medium">✓ Your data remains securely encrypted on disk with AES-256-GCM.</div>
        <div>✓ You will no longer be asked for a PIN on this device.</div>
      </div>

      <div class="flex justify-end space-x-2 pt-2" style="border-top: 1px solid {currentTheme ? hexToRgba(currentTheme.borderColor, 70) : '#1e293b'};">
        <button
          onclick={() => (isRemovePinConfirmOpen = false)}
          class="px-3 py-1.5 text-xs rounded-lg cursor-pointer hover:opacity-80"
          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};"
        >
          Cancel
        </button>
        <button
          onclick={handleRemovePin}
          class="px-3.5 py-1.5 text-xs font-bold rounded-lg shadow-sm cursor-pointer"
          style="
            background-color: {currentTheme ? currentTheme.accentColor : '#f59e0b'};
            color: #ffffff;
          "
        >
          Remove PIN
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
          <span>Full Vault Backup & Restore</span>
        </h3>
        <button
          onclick={() => (isBackupModalOpen = false)}
          class="p-1 cursor-pointer hover:opacity-80"
          style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};"
          title="Close"
        >
          ✕
        </button>
      </div>

      <p class="text-xs leading-relaxed" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#cbd5e1'};">
        All passwords, notes, categories, and attached documents can be exported as a single encrypted backup archive (<strong>.vaultbak</strong>) or restored to another device.
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
            <span class="font-semibold text-xs" style="color: {currentTheme ? currentTheme.accentColor : '#fbbf24'};">Export Backup</span>
          </div>
          <span class="text-[11px] leading-snug" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">
            Exports the complete vault into a .vaultbak archive file.
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
            <span class="font-semibold text-xs" style="color: {currentTheme ? currentTheme.accentColor : '#fbbf24'};">Restore Backup</span>
          </div>
          <span class="text-[11px] leading-snug" style="color: {currentTheme ? currentTheme.secondaryTextColor : '#94a3b8'};">
            Restores vault data from a previously exported .vaultbak file.
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
          Close
        </button>
      </div>
    </div>
  </div>
{/if}
