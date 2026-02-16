<script>
  import { fsStore } from '../stores/fileSystem.svelte.js';
  import { 
    ChevronRight, ChevronDown, File, Folder, FolderOpen, 
    MoreVertical, Search, Filter, RefreshCw, FolderPlus, FilePlus
  } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import FileTreeItem from './FileTreeItem.svelte';
  import { cn } from '../utils.js';
  
  let searchQuery = $state('');
  let isRefreshing = $state(false);

  async function refreshExplorer() {
    if (!fsStore.currentProjectDir) return;
    isRefreshing = true;
    try {
      const tree = await invoke('scan_workspace', { path: fsStore.currentProjectDir, depth: 4 });
      fsStore.setFileTree(tree);
    } catch (e) {
      console.error('Refresh failed:', e);
    } finally {
      isRefreshing = false;
    }
  }
</script>

<div class="h-full flex flex-col bg-background/50 backdrop-blur-xl border-r border-border/50 select-none animate-in slide-in-from-left-4 duration-300">
  
  <!-- Explorer Header -->
  <div class="h-12 min-h-[48px] border-b border-border/40 flex items-center justify-between px-4 bg-muted/10 shrink-0 shadow-sm">
    <div class="flex items-center gap-2">
       <h2 class="text-[11px] font-black uppercase tracking-widest text-muted-foreground/80">Explorer</h2>
       {#if fsStore.currentProjectDir}
         <div class="w-1.5 h-1.5 rounded-full bg-primary shadow-[0_0_8px_hsl(var(--primary))]"></div>
       {/if}
    </div>
    
    <div class="flex items-center gap-0.5">
       <button class="p-1.5 hover:bg-muted rounded-md text-muted-foreground hover:text-foreground transition-all" title="New File">
          <FilePlus size={14} />
       </button>
       <button class="p-1.5 hover:bg-muted rounded-md text-muted-foreground hover:text-foreground transition-all" title="New Folder">
          <FolderPlus size={14} />
       </button>
       <button 
        class="p-1.5 hover:bg-muted rounded-md text-muted-foreground hover:text-foreground transition-all"
        onclick={refreshExplorer}
        title="Refresh"
       >
          <RefreshCw size={14} class={isRefreshing ? 'animate-spin' : ''} />
       </button>
       <button class="p-1.5 hover:bg-muted rounded-md text-muted-foreground hover:text-foreground transition-all" title="More">
          <MoreVertical size={14} />
       </button>
    </div>
  </div>

  <!-- Search / Filter -->
  <div class="p-3 border-b border-border/30">
    <div class="relative flex items-center group">
       <Search size={12} class="absolute left-2.5 text-muted-foreground/50 group-focus-within:text-primary transition-colors" />
       <input 
         type="text" 
         bind:value={searchQuery} 
         placeholder="Filter files..." 
         class="w-full bg-muted/20 border border-border/40 rounded-lg pl-8 pr-3 py-1.5 text-xs focus:ring-2 focus:ring-primary/20 focus:border-primary/40 outline-none transition-all placeholder:text-muted-foreground/30 font-medium"
       />
    </div>
  </div>

  <!-- Workspace Header -->
  {#if fsStore.currentProjectDir}
    <div class="px-4 py-3 border-b border-border/20 bg-muted/5">
       <div class="flex items-center gap-2.5">
          <div class="w-6 h-6 rounded-lg bg-primary/10 flex items-center justify-center text-primary border border-primary/20">
             <FolderOpen size={14} />
          </div>
          <div class="flex flex-col min-w-0">
             <span class="text-[11px] font-bold text-foreground truncate uppercase tracking-tight">{fsStore.currentProjectDir.split('/').pop()}</span>
             <span class="text-[9px] text-muted-foreground truncate opacity-50">{fsStore.currentProjectDir}</span>
          </div>
       </div>
    </div>
  {/if}
  
  <!-- File Tree -->
  <div class="flex-1 overflow-y-auto no-scrollbar py-2">
    {#if fsStore.fileTree}
       <div class="space-y-[1px]">
          <FileTreeItem item={fsStore.fileTree} depth={0} />
       </div>
    {:else}
      <div class="h-full flex flex-col items-center justify-center text-center p-8 opacity-40 animate-in fade-in duration-700">
        <div class="w-16 h-16 rounded-3xl bg-muted/50 border border-border flex items-center justify-center mb-6">
          <Folder size={32} class="text-muted-foreground" />
        </div>
        <h3 class="text-xs font-bold text-foreground mb-2">No workspace open</h3>
        <p class="text-[10px] text-muted-foreground max-w-[150px] leading-relaxed">
          Open a folder to start managing your project files.
        </p>
        <button 
          onclick={() => fsStore.setProjectDir()}
          class="mt-4 px-4 py-2 bg-primary/10 text-primary border border-primary/20 rounded-xl text-[11px] font-bold hover:bg-primary hover:text-primary-foreground transition-all shadow-lg shadow-primary/5 active:scale-95"
        >
           Open Folder
        </button>
      </div>
    {/if}
  </div>

  <!-- Explorer Footer (Stats) -->
  <div class="h-6 px-3 border-t border-border/30 flex items-center justify-between bg-muted/5 shrink-0 text-[10px] text-muted-foreground">
     <div class="flex items-center gap-2">
        <div class="w-1.5 h-1.5 rounded-full bg-green-500"></div>
        <span>Watching</span>
     </div>
     <span>{fsStore.openFiles.length} files open</span>
  </div>
</div>

<style>
  .no-scrollbar::-webkit-scrollbar { display: none; }
  .no-scrollbar { -ms-overflow-style: none; scrollbar-width: none; }
</style>
