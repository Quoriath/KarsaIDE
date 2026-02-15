<script>
  import { fsStore } from '../stores/fileSystem.svelte.js';
  import { invoke } from '@tauri-apps/api/core';
  import { 
    Folder, 
    FolderOpen, 
    File, 
    FileJson, 
    FileCode, 
    FileType, 
    ChevronRight, 
    ChevronDown, 
    Plus,
    MoreHorizontal
  } from 'lucide-svelte';
  import { cn } from '../utils.js';

  let expandedDirs = $state(new Set());

  // Helper to determine icon based on file extension
  function getFileIcon(name) {
    const ext = name.split('.').pop()?.toLowerCase();
    switch (ext) {
      case 'json': return FileJson;
      case 'js':
      case 'ts':
      case 'jsx':
      case 'tsx':
      case 'svelte':
      case 'rs':
      case 'py':
      case 'html':
      case 'css':
        return FileCode;
      default: return File;
    }
  }

  function getFileColor(name) {
    const ext = name.split('.').pop()?.toLowerCase();
    const colors = {
      rs: 'text-orange-500',
      js: 'text-yellow-400',
      ts: 'text-blue-400',
      svelte: 'text-orange-600',
      html: 'text-orange-400',
      css: 'text-blue-400',
      json: 'text-yellow-300',
      md: 'text-gray-400',
      toml: 'text-gray-400',
      jsx: 'text-cyan-400',
      tsx: 'text-cyan-400'
    };
    return colors[ext] || 'text-muted-foreground';
  }

  async function handleItemClick(item) {
    if (item.is_dir) {
      // In a real tree, we'd fetch children. 
      // For now, this replaces the view (Finder style as per existing logic)
      // We should ideally append to a tree structure in the store.
      await fsStore.refreshDirectory(item.path);
    } else {
      await fsStore.openFile(item.path);
    }
  }

  async function openFolder() {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({ directory: true });
    if (selected) {
      await fsStore.setProjectDir(selected);
    }
  }
</script>

<aside class="w-64 bg-muted/10 border-r border-border flex flex-col h-full select-none">
  <!-- Sidebar Header -->
  <div class="h-9 px-4 border-b border-border flex items-center justify-between">
    <span class="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Explorer</span>
    <div class="flex items-center gap-1">
      {#if fsStore.currentProjectDir}
        <button 
          class="p-1 hover:bg-muted rounded-sm text-muted-foreground hover:text-foreground transition-colors"
          onclick={openFolder}
          title="Open Folder"
        >
          <Plus size={14} />
        </button>
        <button class="p-1 hover:bg-muted rounded-sm text-muted-foreground hover:text-foreground transition-colors">
          <MoreHorizontal size={14} />
        </button>
      {/if}
    </div>
  </div>
  
  <div class="flex-1 overflow-y-auto p-2">
    {#if !fsStore.currentProjectDir}
      <div class="flex flex-col items-center justify-center h-[50vh] text-center space-y-4">
        <div class="p-4 bg-muted/50 rounded-full text-muted-foreground">
          <FolderOpen size={32} />
        </div>
        <div class="space-y-2">
          <p class="text-sm font-medium text-foreground">No Folder Opened</p>
          <p class="text-xs text-muted-foreground max-w-[150px] mx-auto">Open a folder to start editing files.</p>
        </div>
        <button 
          class="bg-primary hover:bg-primary/90 text-primary-foreground px-4 py-2 rounded-md text-sm font-medium transition-colors shadow-sm"
          onclick={openFolder}
        >
          Open Folder
        </button>
      </div>
    {:else}
      <!-- Project Root Name -->
      <div class="px-2 py-1.5 mb-2 flex items-center gap-2 text-xs font-bold text-foreground bg-muted/30 rounded-md">
        <ChevronDown size={14} class="text-muted-foreground" />
        <span class="truncate">{fsStore.currentProjectDir.split('/').pop()}</span>
      </div>

      <!-- File List -->
      <div class="space-y-0.5">
        {#each fsStore.directoryContents as item}
          {@const IconCmp = getFileIcon(item.name)}
          <button
            class="w-full text-left px-2 py-1.5 rounded-md hover:bg-muted/50 flex items-center gap-2 text-sm group transition-all duration-200"
            onclick={() => handleItemClick(item)}
          >
            {#if item.is_dir}
              <ChevronRight size={14} class="text-muted-foreground/50 group-hover:text-muted-foreground transition-colors" />
              <Folder size={16} class="text-blue-400 fill-blue-400/20" />
              <span class="truncate text-foreground/80 group-hover:text-foreground">{item.name}</span>
            {:else}
              <span class="w-3.5"></span>
              <IconCmp size={16} class={getFileColor(item.name)} />
              <span class="truncate text-muted-foreground group-hover:text-foreground transition-colors">{item.name}</span>
            {/if}
          </button>
        {/each}
        
        {#if fsStore.directoryContents.length === 0}
          <div class="px-8 py-2 text-xs text-muted-foreground italic">Empty directory</div>
        {/if}
      </div>
    {/if}
  </div>
</aside>
