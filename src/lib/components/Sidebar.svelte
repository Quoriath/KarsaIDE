<script>
  import { fsStore } from '../stores/fileSystem.svelte.js';
  import { open } from '@tauri-apps/plugin-dialog';
  import { FolderOpen, Plus, MoreHorizontal, ChevronDown } from 'lucide-svelte';
  import FileTreeItem from './FileTreeItem.svelte';
  import ContextMenu from './ContextMenu.svelte';

  async function openFolder() {
    try {
      const selected = await open({ 
        directory: true,
        multiple: false 
      });
      if (selected) {
        await fsStore.setProjectDir(selected);
      }
    } catch (e) {
      console.error('Failed to open folder', e);
    }
  }
</script>

<aside class="flex flex-col h-full bg-muted/5 select-none">
  <!-- Sidebar Header -->
  <div class="h-9 px-4 border-b border-border flex items-center justify-between shrink-0">
    <span class="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Explorer</span>
    <div class="flex items-center gap-1">
      {#if fsStore.currentProjectDir}
        <button 
          class="p-1 hover:bg-muted rounded-sm text-muted-foreground hover:text-foreground transition-colors"
          title="New File"
        >
          <Plus size={14} />
        </button>
        <button class="p-1 hover:bg-muted rounded-sm text-muted-foreground hover:text-foreground transition-colors">
          <MoreHorizontal size={14} />
        </button>
      {/if}
    </div>
  </div>
  
  <div class="flex-1 overflow-y-auto">
    {#if !fsStore.currentProjectDir}
      <div class="flex flex-col items-center justify-center h-full text-center space-y-4 p-4">
        <div class="p-4 bg-muted/50 rounded-full text-muted-foreground">
          <FolderOpen size={32} />
        </div>
        <div class="space-y-1">
          <p class="text-sm font-medium text-foreground">No Folder Opened</p>
          <p class="text-xs text-muted-foreground">Open a project to start coding.</p>
        </div>
        <button 
          class="bg-primary hover:bg-primary/90 text-primary-foreground px-4 py-2 rounded-md text-sm font-medium transition-colors shadow-sm w-full"
          onclick={openFolder}
        >
          Open Folder
        </button>
      </div>
    {:else}
      <!-- Project Root Header -->
      <div class="px-2 py-2 mb-1 flex items-center gap-2 text-xs font-bold text-foreground hover:bg-muted/30 cursor-pointer sticky top-0 bg-background/95 backdrop-blur-sm z-10 border-b border-border/50">
        <ChevronDown size={14} class="text-muted-foreground" />
        <span class="truncate">{fsStore.currentProjectDir.split(/[/\\]/).pop()}</span>
        <span class="ml-auto text-[10px] text-muted-foreground px-1.5 py-0.5 bg-muted/50 rounded-full">
           {fsStore.directoryContents.length} files
        </span>
      </div>

      <!-- Recursive Tree -->
      <div class="pb-4">
        {#each fsStore.directoryContents as item}
          <FileTreeItem item={item} />
        {/each}
      </div>
    {/if}
  </div>
  
  <ContextMenu />
</aside>
