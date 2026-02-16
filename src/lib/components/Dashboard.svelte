<script>
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { fsStore } from '../stores/fileSystem.svelte.js';
  import { configStore } from '../stores/config.svelte.js';
  import { onMount } from 'svelte';
  import { 
    FilePlus, FolderOpen, FileCode, GitBranch, Clock, Command, 
    Settings, HelpCircle, Book, Zap, ArrowRight, Sparkles
  } from 'lucide-svelte';
  
  let recents = $state([]);
  let recentConversations = $state([]);

  onMount(async () => {
    try {
      const convs = await invoke('get_conversations', { mode: null, limit: 6 });
      recentConversations = convs;
    } catch (e) {}
    
    try {
      const session = await invoke('get_session');
      if (session.recent_files) {
        recents = session.recent_files.slice(0, 6).map(path => ({
          name: path.split('/').pop(),
          path,
          lastOpened: 'Recently'
        }));
      }
    } catch (e) {}
  });

  async function openFolder() {
    try {
      const selected = await open({ directory: true, multiple: false });
      if (selected) {
        await invoke('set_active_workspace', { path: selected });
        const tree = await invoke('scan_workspace', { path: selected, depth: 5 });
        fsStore.setProjectDir(selected);
        fsStore.setFileTree(tree);
        await invoke('start_indexing', { path: selected });
      }
    } catch (err) { console.error(err); }
  }

  async function openFile() {
    try {
      const selected = await open({ multiple: false });
      if (selected) fsStore.openFile(selected);
    } catch (err) { console.error(err); }
  }
</script>

<div class="h-full w-full flex items-center justify-center bg-background text-foreground overflow-y-auto overflow-x-hidden relative p-4 lg:p-12 selection:bg-primary/20">
  
  <!-- Subtle Visual Elements -->
  <div class="absolute inset-0 bg-[radial-gradient(circle_at_50%_50%,hsl(var(--primary)/0.03),transparent_50%)] pointer-events-none"></div>
  <div class="absolute inset-0 bg-[linear-gradient(to_right,hsl(var(--border)/0.2)_1px,transparent_1px),linear-gradient(to_bottom,hsl(var(--border)/0.2)_1px,transparent_1px)] bg-[size:40px_40px] [mask-image:radial-gradient(ellipse_at_center,black,transparent_80%)] pointer-events-none"></div>

  <div class="max-w-6xl w-full grid grid-cols-1 lg:grid-cols-[1.1fr_1fr] gap-12 lg:gap-20 z-10">
    
    <!-- Branding & Core Actions -->
    <div class="flex flex-col justify-center animate-in slide-in-from-left-8 duration-700">
      <div class="mb-10">
        <div class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-primary/10 border border-primary/20 text-[10px] font-bold text-primary uppercase tracking-[0.2em] mb-6 animate-pulse">
           <Sparkles size={10} /> v0.1.0-alpha
        </div>
        <h1 class="text-5xl lg:text-6xl font-bold tracking-tighter text-foreground mb-4">
          Karsa <span class="text-primary italic">IDE</span>
        </h1>
        <p class="text-base text-muted-foreground max-w-md leading-relaxed">
          The autonomous coding environment built for speed, intelligence, and minimalist precision.
        </p>
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <button onclick={openFolder} class="group flex flex-col gap-3 p-5 rounded-2xl bg-muted/20 border border-border/50 hover:border-primary/40 hover:bg-primary/5 transition-all text-left shadow-sm active:scale-[0.98]">
           <div class="w-10 h-10 rounded-xl bg-primary/10 flex items-center justify-center text-primary group-hover:scale-110 transition-transform">
              <FolderOpen size={20} />
           </div>
           <div>
              <div class="text-sm font-bold flex items-center gap-2">Open Folder <ArrowRight size={14} class="opacity-0 group-hover:opacity-100 -translate-x-2 group-hover:translate-x-0 transition-all" /></div>
              <p class="text-[11px] text-muted-foreground mt-1">Start working on an existing project.</p>
           </div>
        </button>

        <button onclick={openFile} class="group flex flex-col gap-3 p-5 rounded-2xl bg-muted/20 border border-border/50 hover:border-blue-500/40 hover:bg-blue-500/5 transition-all text-left shadow-sm active:scale-[0.98]">
           <div class="w-10 h-10 rounded-xl bg-blue-500/10 flex items-center justify-center text-blue-500 group-hover:scale-110 transition-transform">
              <FileCode size={20} />
           </div>
           <div>
              <div class="text-sm font-bold flex items-center gap-2">New File <ArrowRight size={14} class="opacity-0 group-hover:opacity-100 -translate-x-2 group-hover:translate-x-0 transition-all" /></div>
              <p class="text-[11px] text-muted-foreground mt-1">Create a single script or document.</p>
           </div>
        </button>
      </div>

      <div class="mt-12 flex items-center gap-6 text-[11px] text-muted-foreground">
         <button class="flex items-center gap-2 hover:text-foreground transition-colors">
            <Command size={14} /> Command Palette
         </button>
         <button class="flex items-center gap-2 hover:text-foreground transition-colors">
            <GitBranch size={14} /> Version Control
         </button>
         <button class="flex items-center gap-2 hover:text-foreground transition-colors">
            <Book size={14} /> Docs
         </button>
      </div>
    </div>

    <!-- Recent Section -->
    <div class="flex flex-col bg-background/40 border border-border/40 rounded-3xl backdrop-blur-xl overflow-hidden shadow-2xl animate-in slide-in-from-right-8 duration-700">
      <div class="p-6 border-b border-border/40 flex items-center justify-between shrink-0 bg-muted/10">
         <div class="flex items-center gap-3">
            <div class="w-8 h-8 rounded-lg bg-muted/50 flex items-center justify-center">
               <Clock size={16} class="text-muted-foreground" />
            </div>
            <h2 class="text-sm font-bold">Recent Projects</h2>
         </div>
         <span class="text-[10px] font-bold text-muted-foreground/40 uppercase tracking-widest">{recents.length} Files</span>
      </div>
      
      <div class="flex-1 overflow-y-auto p-4 space-y-2 scrollbar-hide">
        {#if recents.length === 0}
          <div class="h-full flex flex-col items-center justify-center py-12 text-center opacity-30">
            <Clock size={32} class="mb-4" />
            <p class="text-xs">No recent history</p>
          </div>
        {/if}
        
        {#each recents as item}
          <button 
            onclick={() => fsStore.openFile(item.path)}
            class="group w-full p-4 rounded-xl hover:bg-muted/50 text-left transition-all border border-transparent hover:border-border/50 flex items-center gap-4">
            <div class="w-10 h-10 rounded-lg bg-muted/20 flex items-center justify-center group-hover:bg-background transition-colors">
               <FileCode size={18} class="text-muted-foreground group-hover:text-primary transition-colors" />
            </div>
            <div class="flex-1 min-w-0">
               <div class="text-sm font-semibold truncate group-hover:text-primary transition-colors">{item.name}</div>
               <div class="text-[10px] text-muted-foreground/60 truncate font-mono mt-0.5">{item.path}</div>
            </div>
            <ArrowRight size={14} class="text-muted-foreground opacity-0 group-hover:opacity-100 transition-all -translate-x-2 group-hover:translate-x-0" />
          </button>
        {/each}
      </div>

      <div class="p-4 border-t border-border/30 bg-muted/5">
         <button class="w-full py-2.5 rounded-xl border border-border/50 text-xs font-semibold hover:bg-muted transition-all text-muted-foreground hover:text-foreground">
            View All Activity
         </button>
      </div>
    </div>

  </div>
</div>
