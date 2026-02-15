<script>
  import { invoke } from '@tauri-apps/api/core';
  import { fsStore } from '../stores/fileSystem.svelte.js';
  import { uiState } from '../stores/uiState.svelte.js';
  import { 
    ChevronRight, ChevronDown, Folder, FolderOpen, 
    File, FileCode, FileJson, Loader2, FilePlus, Pencil, Trash2 
  } from 'lucide-svelte';
  import { cn } from '../utils.js';
  import FileTreeItem from './FileTreeItem.svelte'; // Self import for recursion

  let { item, depth = 0 } = $props();

  let expanded = $state(false);
  let children = $state(null);
  let loading = $state(false);

  // Helper icons
  function getFileIcon(name) {
    const ext = name.split('.').pop()?.toLowerCase();
    switch (ext) {
      case 'json': return FileJson;
      case 'js': case 'ts': case 'jsx': case 'tsx': case 'svelte': case 'rs': case 'css': case 'html': return FileCode;
      default: return File;
    }
  }

  async function toggleExpand(e) {
    e.stopPropagation();
    if (!item.is_dir) {
      fsStore.openFile(item.path);
      return;
    }

    expanded = !expanded;

    if (expanded && !children) {
      loading = true;
      try {
        children = await invoke('list_directory', { path: item.path }).catch(() => []);
      } catch (err) {
        console.error("Failed to list dir", err);
        children = [];
      } finally {
        loading = false;
      }
    }
  }

  function handleContextMenu(e) {
    const actions = [
      { label: 'New File', icon: FilePlus, fn: (i) => console.log('New File in', i.path) },
      { label: 'New Folder', icon: Folder, fn: (i) => console.log('New Folder in', i.path) },
      { separator: true },
      { label: 'Rename', icon: Pencil, fn: (i) => console.log('Rename', i.path) },
      { label: 'Delete', icon: Trash2, danger: true, fn: (i) => console.log('Delete', i.path) },
    ];
    uiState.openContextMenu(e, item, actions);
  }
</script>

<div>
  <button
    class={cn(
      "w-full flex items-center gap-1.5 py-1 px-2 hover:bg-muted/50 transition-colors text-sm select-none",
      fsStore.activeFile?.path === item.path ? "bg-primary/10 text-primary" : "text-muted-foreground hover:text-foreground"
    )}
    style="padding-left: {depth * 12 + 8}px"
    onclick={toggleExpand}
    oncontextmenu={handleContextMenu}
  >
    {#if item.is_dir}
      <span class="text-muted-foreground/70 shrink-0">
        {#if expanded}
          <ChevronDown size={14} />
        {:else}
          <ChevronRight size={14} />
        {/if}
      </span>
      {#if expanded}
        <FolderOpen size={16} class="text-blue-400 shrink-0" />
      {:else}
        <Folder size={16} class="text-blue-400 shrink-0" />
      {/if}
    {:else}
      <span class="w-3.5 shrink-0"></span>
      {@const IconCmp = getFileIcon(item.name)}
      <IconCmp size={16} class="opacity-80 shrink-0" />
    {/if}

    <span class="truncate">{item.name}</span>
    
    {#if loading}
      <Loader2 size={12} class="animate-spin ml-auto" />
    {/if}
  </button>

  {#if expanded && children}
    <div class="border-l border-border/30 ml-2">
      {#each children as child}
        <FileTreeItem item={child} depth={depth + 1} />
      {/each}
      {#if children.length === 0}
        <div class="py-1 px-2 text-xs text-muted-foreground/50 italic" style="padding-left: {(depth + 1) * 12 + 8}px">
          Empty
        </div>
      {/if}
    </div>
  {/if}
</div>
