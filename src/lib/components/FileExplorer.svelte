<script>
  import { invoke } from '@tauri-apps/api/core';
  import { fsStore } from '../stores/fileSystem.svelte.js';
  import { uiState } from '../stores/uiState.svelte.js';
  import { 
    ChevronRight, ChevronDown, Folder, FolderOpen, 
    File, FileCode, FileJson, Loader2, FilePlus, Pencil, Trash2, 
    Search, Plus, Filter, MoreVertical
  } from 'lucide-svelte';
  import { cn } from '../utils.js';
  import FileTreeItem from './FileTreeItem.svelte';
  import Dialog from './ui/Dialog.svelte';
  import { onMount, onDestroy } from 'svelte';
  
  let searchQuery = $state('');
  
  async function refreshExplorer() {
    if (fsStore.currentProjectDir) {
       await fsStore.refreshExplorer();
    }
  }

  onMount(() => {
    if (!fsStore.fileTree && fsStore.currentProjectDir) {
      refreshExplorer();
    }
    window.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
  });

  async function handleKeydown(e) {
    // Global shortcuts for File Explorer when focused or active
    // Simple implementation: check if not typing in input
    if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;

    if (e.key === 'Delete') {
      const activePath = fsStore.activeFile?.path;
      if (activePath) {
        uiState.showDialog({
          title: 'Delete File',
          message: `Delete ${activePath.split('/').pop()}?`,
          confirmLabel: 'Delete',
          variant: 'danger',
          onConfirm: () => fsStore.deletePath(activePath)
        });
      }
    }
    
    // Copy/Paste handling would ideally require focus on the specific tree item
    // Here we implement basic global copy/paste for active file if tree is focused-ish
    if (fsStore.activeFile?.path) {
      if ((e.ctrlKey || e.metaKey) && e.key === 'c') {
        fsStore.copyToClipboard(fsStore.activeFile.path);
      }
      if ((e.ctrlKey || e.metaKey) && e.key === 'x') {
        fsStore.cutToClipboard(fsStore.activeFile.path);
      }
    }
    
    if ((e.ctrlKey || e.metaKey) && e.key === 'v') {
      // Paste into current active file's directory
      if (fsStore.activeFile?.path && fsStore.clipboard.path) {
        const parent = fsStore.activeFile.path.substring(0, fsStore.activeFile.path.lastIndexOf('/'));
        await fsStore.pasteFromClipboard(parent);
      } else if (fsStore.currentProjectDir && fsStore.clipboard.path) {
        // Or paste into root if nothing active
        await fsStore.pasteFromClipboard(fsStore.currentProjectDir);
      }
    }
  }
</script>

<div class="h-full flex flex-col bg-background/50 backdrop-blur-md select-none relative">
  <!-- Header -->
  <div class="shrink-0 p-4 space-y-3">
    <div class="flex items-center justify-between">
      <div class="flex flex-col min-w-0">
        <span class="text-[10px] font-black text-muted-foreground uppercase tracking-[0.15em]">Workspace</span>
        <h2 class="text-xs font-bold text-foreground truncate">
          {fsStore.currentProjectDir?.split(/[/\\]/).pop() || 'No Project'}
        </h2>
      </div>
      <div class="flex items-center gap-0.5 shrink-0">
        <button onclick={refreshExplorer} class="p-1.5 hover:bg-muted rounded-md text-muted-foreground hover:text-foreground transition-all" title="Refresh">
          <Plus size={14} class="rotate-45" />
        </button>
        <button class="p-1.5 hover:bg-muted rounded-md text-muted-foreground hover:text-foreground transition-all">
          <MoreVertical size={14} />
        </button>
      </div>
    </div>

    <!-- Filter -->
    <div class="relative group">
      <Search size={12} class="absolute left-2.5 top-2.5 text-muted-foreground/50 group-focus-within:text-primary transition-colors" />
      <input 
        type="text" 
        placeholder="Filter files..." 
        bind:value={searchQuery}
        class="w-full bg-muted/20 border border-border/40 rounded-lg pl-8 pr-3 py-1.5 text-[11px] outline-none focus:ring-1 focus:ring-primary/30 focus:border-primary/40 transition-all placeholder:text-muted-foreground/30"
      />
    </div>
  </div>
  
  <!-- Tree Area -->
  <div class="flex-1 overflow-y-auto overflow-x-hidden py-2 no-scrollbar outline-none" tabindex="0">
    {#if fsStore.fileTree}
      <FileTreeItem item={fsStore.fileTree} depth={0} />
    {:else}
      <div class="h-full flex flex-col items-center justify-center p-8 text-center space-y-4 opacity-40">
        <div class="p-4 rounded-2xl bg-muted/30 border border-border/50">
          <Folder size={24} class="text-muted-foreground" />
        </div>
        <div class="space-y-1">
          <p class="text-xs font-medium">Empty Workspace</p>
          <p class="text-[10px] leading-relaxed">Open a folder to see your files here.</p>
        </div>
      </div>
    {/if}
  </div>

  <!-- Dialogs -->
  <Dialog />
</div>

<style>
  .no-scrollbar::-webkit-scrollbar { display: none; }
  .no-scrollbar { -ms-overflow-style: none; scrollbar-width: none; }
</style>
