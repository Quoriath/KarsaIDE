<script>
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { fsStore } from '../stores/fileSystem.svelte.js';
  import { onMount } from 'svelte';
  import { 
    FilePlus, FolderOpen, FileCode, Clock, Sparkles, Cpu, 
    ArrowUpRight, Github, Book, Command, Terminal, Zap, Layers
  } from 'lucide-svelte';
  import { cn } from '../utils.js';
  
  let recents = $state([]);
  let recentConversations = $state([]);

  onMount(async () => {
    try {
      const [convs, session] = await Promise.all([
        invoke('get_conversations', { mode: 'all', limit: 3 }),
        invoke('get_session')
      ]);
      recentConversations = convs;
      if (session.recent_files) {
        recents = session.recent_files.slice(0, 5).map(path => ({
          name: path.split('/').pop(),
          path,
          ext: path.split('.').pop()
        }));
      }
    } catch (e) {
      console.error('Dashboard init failed:', e);
    }
  });

  async function openFolder() {
    try {
      const selected = await open({ directory: true });
      if (selected) {
        await invoke('set_active_workspace', { path: selected });
        const tree = await invoke('scan_workspace', { path: selected, depth: 4 });
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

<div class="h-full w-full bg-background flex flex-col items-center justify-center p-6 md:p-12 overflow-hidden selection:bg-primary/30">
  
  <!-- Subtle UI Pattern -->
  <div class="absolute inset-0 pointer-events-none opacity-[0.02] dark:opacity-[0.04]">
    <div class="absolute inset-0 bg-[radial-gradient(circle_at_2px_2px,var(--foreground)_1px,transparent_0)] bg-[size:32px_32px]"></div>
  </div>

  <div class="max-w-5xl w-full h-full max-h-[800px] flex flex-col gap-6 animate-in fade-in zoom-in-95 duration-700">
    
    <!-- Top Row: Welcome & Status -->
    <div class="grid grid-cols-12 gap-4 h-32 shrink-0">
      <div class="col-span-8 bento-card flex items-center gap-6 group">
        <div class="w-16 h-16 rounded-2xl bg-primary flex items-center justify-center text-primary-foreground shadow-2xl shadow-primary/40 group-hover:rotate-6 transition-transform">
          <Zap size={32} fill="currentColor" strokeWidth={1} />
        </div>
        <div>
          <h1 class="text-3xl font-black tracking-tighter">KARSA <span class="text-primary italic">IDE</span></h1>
          <p class="text-sm text-muted-foreground font-medium opacity-70">Integrated Autonomous Environment v0.1.0</p>
        </div>
        <div class="ml-auto flex flex-col items-end">
           <div class="flex items-center gap-2 text-[10px] font-bold text-green-500 uppercase tracking-widest bg-green-500/10 px-2.5 py-1 rounded-full border border-green-500/20">
              <div class="w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse"></div>
              Engine Ready
           </div>
        </div>
      </div>
      <div class="col-span-4 bento-card flex flex-col justify-center items-center text-center group">
         <Sparkles size={24} class="text-purple-500 mb-2 group-hover:scale-110 transition-transform" />
         <span class="text-xs font-black uppercase tracking-tighter">Active Intelligence</span>
         <span class="text-[10px] text-muted-foreground mt-1 opacity-60">Connected to Kilo-Node</span>
      </div>
    </div>

    <!-- Main Bento Grid -->
    <div class="flex-1 grid grid-cols-12 gap-4 min-h-0">
      
      <!-- Actions Column -->
      <div class="col-span-4 flex flex-col gap-4">
        <button onclick={openFolder} class="flex-1 bento-card group flex flex-col justify-between hover:bg-primary/5">
          <div class="w-10 h-10 rounded-xl bg-primary/10 flex items-center justify-center text-primary transition-colors group-hover:bg-primary group-hover:text-primary-foreground">
            <FolderOpen size={20} />
          </div>
          <div class="mt-4">
            <div class="text-sm font-bold flex items-center justify-between">
              Open Workspace
              <ArrowUpRight size={14} class="opacity-0 group-hover:opacity-100 transition-all" />
            </div>
            <p class="text-[10px] text-muted-foreground mt-1 opacity-60">Browse your local directories</p>
          </div>
        </button>
        
        <button onclick={openFile} class="flex-1 bento-card group flex flex-col justify-between hover:bg-muted/40">
          <div class="w-10 h-10 rounded-xl bg-muted flex items-center justify-center text-foreground transition-colors group-hover:bg-foreground group-hover:text-background">
            <FilePlus size={20} />
          </div>
          <div class="mt-4">
            <div class="text-sm font-bold flex items-center justify-between">
              New Artifact
              <ArrowUpRight size={14} class="opacity-0 group-hover:opacity-100 transition-all" />
            </div>
            <p class="text-[10px] text-muted-foreground mt-1 opacity-60">Create a new source file</p>
          </div>
        </button>
      </div>

      <!-- Recent Content Column -->
      <div class="col-span-8 bento-card flex flex-col p-0">
        <div class="p-5 border-b border-border/30 flex items-center justify-between shrink-0 bg-muted/5">
           <h2 class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
             <Clock size={14} class="text-primary" />
             Recent Assets
           </h2>
           <button class="text-[10px] font-bold text-primary/60 hover:text-primary uppercase">Clear</button>
        </div>
        
        <div class="flex-1 overflow-y-auto p-2 no-scrollbar">
          {#each recents as item}
            <button 
              onclick={() => fsStore.openFile(item.path)}
              class="w-full flex items-center gap-4 p-3.5 rounded-xl hover:bg-muted/50 transition-all text-left group"
            >
              <div class="w-9 h-9 rounded-lg bg-background border border-border/60 flex items-center justify-center text-muted-foreground group-hover:text-primary group-hover:border-primary/30 transition-all">
                 <FileCode size={18} />
              </div>
              <div class="flex-1 min-w-0">
                <div class="text-sm font-bold truncate">{item.name}</div>
                <div class="text-[9px] text-muted-foreground/40 truncate font-mono tracking-tighter mt-0.5">{item.path}</div>
              </div>
              <div class="text-[9px] font-black uppercase text-muted-foreground/30 px-2 py-1 rounded bg-muted/20">{item.ext}</div>
            </button>
          {/each}

          {#if recents.length === 0}
            <div class="h-full flex flex-col items-center justify-center opacity-30 gap-3">
               <Layers size={32} strokeWidth={1} />
               <span class="text-[10px] font-bold uppercase tracking-widest">Workspace Empty</span>
            </div>
          {/if}
        </div>
      </div>
    </div>

    <!-- Bottom Row: Community & Help -->
    <div class="h-24 grid grid-cols-12 gap-4 shrink-0">
       <div class="col-span-3 bento-card flex items-center gap-3 p-4 group cursor-pointer">
          <div class="w-8 h-8 rounded-lg bg-background border border-border flex items-center justify-center opacity-60 group-hover:text-primary group-hover:opacity-100 group-hover:border-primary/30 transition-all">
             <Github size={16} />
          </div>
          <span class="text-xs font-bold truncate">Sync Source</span>
       </div>
       <div class="col-span-3 bento-card flex items-center gap-3 p-4 group cursor-pointer">
          <div class="w-8 h-8 rounded-lg bg-background border border-border flex items-center justify-center opacity-60 group-hover:text-primary group-hover:opacity-100 group-hover:border-primary/30 transition-all">
             <Book size={16} />
          </div>
          <span class="text-xs font-bold truncate">Read Manual</span>
       </div>
       <div class="col-span-6 bento-card flex items-center justify-between p-4 px-6 group cursor-pointer overflow-hidden">
          <div class="flex items-center gap-4">
             <Command size={18} class="text-muted-foreground group-hover:text-primary transition-colors" />
             <div class="flex flex-col">
                <span class="text-xs font-bold leading-none">Keyboard Shortcuts</span>
                <span class="text-[10px] text-muted-foreground mt-1 opacity-60">Command Palette (Ctrl + Shift + P)</span>
             </div>
          </div>
          <ArrowUpRight size={16} class="opacity-20 group-hover:opacity-100 group-hover:translate-x-0.5 group-hover:-translate-y-0.5 transition-all" />
       </div>
    </div>

  </div>
</div>

<style>
  .no-scrollbar::-webkit-scrollbar { display: none; }
  .no-scrollbar { -ms-overflow-style: none; scrollbar-width: none; }
</style>
