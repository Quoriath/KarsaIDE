<script>
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { fsStore } from '../stores/fileSystem.svelte.js';
  import { 
    FilePlus, FolderOpen, FileCode, GitBranch, Clock, Command, 
    Settings, HelpCircle, Book, Zap, ArrowRight 
  } from 'lucide-svelte';
  
  let recents = $state([
    { name: 'KarsaIDE', path: '/home/user/projects/karsa-ide', lastOpened: '2h ago' },
    { name: 'website-v2', path: '/home/user/work/website-v2', lastOpened: '5h ago' },
    { name: 'rust-learning', path: '/home/user/learn/rust', lastOpened: '1d ago' },
    { name: 'tauri-app', path: '/home/user/projects/tauri-app', lastOpened: '3d ago' },
    { name: 'backend-api', path: '/home/user/work/backend-api', lastOpened: '4d ago' },
    { name: 'notes', path: '/home/user/personal/notes', lastOpened: '1w ago' },
  ]);

  async function openFolder() {
    try {
      const selected = await open({ directory: true, multiple: false });
      if (selected) fsStore.setProjectDir(selected);
    } catch (err) { console.error(err); }
  }

  async function openFile() {
    try {
      const selected = await open({ multiple: false });
      if (selected) fsStore.openFile(selected);
    } catch (err) { console.error(err); }
  }
</script>

<div class="h-full w-full flex items-center justify-center bg-background text-foreground overflow-hidden relative">
  <!-- Subtle Background Pattern -->
  <div class="absolute inset-0 bg-[linear-gradient(to_right,#80808012_1px,transparent_1px),linear-gradient(to_bottom,#80808012_1px,transparent_1px)] bg-[size:24px_24px] pointer-events-none"></div>
  <div class="absolute inset-0 bg-gradient-to-t from-background via-transparent to-transparent pointer-events-none"></div>

  <div class="max-w-4xl w-full h-[80vh] grid grid-cols-[1.2fr_1fr] gap-12 z-10 p-8">
    
    <!-- Left Column: Branding & Actions -->
    <div class="flex flex-col h-full">
      <div class="mb-10">
        <div class="w-12 h-12 bg-primary/20 rounded-xl flex items-center justify-center text-primary mb-4 border border-primary/20 shadow-[0_0_15px_rgba(59,130,246,0.3)]">
           <Zap size={24} fill="currentColor" />
        </div>
        <h1 class="text-3xl font-bold tracking-tight text-foreground">Karsa <span class="text-primary">IDE</span></h1>
        <p class="text-muted-foreground mt-2 text-sm max-w-sm">
          A high-performance, AI-native editor designed for flow state.
        </p>
      </div>

      <div class="space-y-6 flex-1">
        <div>
          <h2 class="text-[10px] font-bold uppercase tracking-widest text-muted-foreground/50 mb-3">Start</h2>
          <div class="space-y-1">
            <button onclick={openFile} class="group w-full flex items-center gap-3 p-2 rounded-md hover:bg-accent/50 text-left transition-all">
              <FileCode size={16} class="text-blue-400 opacity-80 group-hover:opacity-100" />
              <span class="text-sm text-foreground/80 group-hover:text-foreground">New File...</span>
              <span class="ml-auto text-[10px] text-muted-foreground bg-muted/50 px-1.5 py-0.5 rounded border border-border opacity-0 group-hover:opacity-100 transition-opacity">Ctrl+N</span>
            </button>
            <button onclick={openFolder} class="group w-full flex items-center gap-3 p-2 rounded-md hover:bg-accent/50 text-left transition-all">
              <FolderOpen size={16} class="text-yellow-400 opacity-80 group-hover:opacity-100" />
              <span class="text-sm text-foreground/80 group-hover:text-foreground">Open Folder...</span>
              <span class="ml-auto text-[10px] text-muted-foreground bg-muted/50 px-1.5 py-0.5 rounded border border-border opacity-0 group-hover:opacity-100 transition-opacity">Ctrl+O</span>
            </button>
            <button class="group w-full flex items-center gap-3 p-2 rounded-md hover:bg-accent/50 text-left transition-all">
              <GitBranch size={16} class="text-green-400 opacity-80 group-hover:opacity-100" />
              <span class="text-sm text-foreground/80 group-hover:text-foreground">Clone Repository...</span>
            </button>
          </div>
        </div>

        <div>
          <h2 class="text-[10px] font-bold uppercase tracking-widest text-muted-foreground/50 mb-3">Resources</h2>
          <div class="grid grid-cols-2 gap-2">
             <button class="flex items-center gap-2 p-2 text-xs text-muted-foreground hover:text-primary hover:bg-primary/5 rounded transition-colors">
                <Command size={14} /> Show All Commands
             </button>
             <button class="flex items-center gap-2 p-2 text-xs text-muted-foreground hover:text-primary hover:bg-primary/5 rounded transition-colors">
                <Book size={14} /> Documentation
             </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Right Column: Recent Projects (Scrollable internally) -->
    <div class="flex flex-col h-full bg-card/30 border border-border/50 rounded-xl backdrop-blur-sm overflow-hidden">
      <div class="p-4 border-b border-border/50 flex items-center justify-between shrink-0 bg-muted/10">
         <h2 class="text-xs font-semibold text-foreground flex items-center gap-2">
            <Clock size={14} class="text-primary" />
            Recent
         </h2>
         <button class="text-[10px] text-primary hover:underline">Clear</button>
      </div>
      
      <div class="flex-1 overflow-y-auto p-2 scrollbar-thin scrollbar-thumb-muted">
        {#each recents as item}
          <button class="group w-full p-2.5 rounded-lg hover:bg-accent/50 text-left transition-all border border-transparent hover:border-border/50 mb-1">
            <div class="flex items-center justify-between mb-1">
               <span class="text-sm font-medium text-foreground group-hover:text-primary transition-colors">{item.name}</span>
               <span class="text-[10px] text-muted-foreground">{item.lastOpened}</span>
            </div>
            <div class="text-[11px] text-muted-foreground/60 truncate font-mono group-hover:text-muted-foreground transition-colors">
               {item.path}
            </div>
          </button>
        {/each}
        
        <button class="w-full py-3 text-xs text-muted-foreground hover:text-foreground flex items-center justify-center gap-1 group mt-2 border-t border-border/30 border-dashed">
           View all history <ArrowRight size={12} class="group-hover:translate-x-1 transition-transform" />
        </button>
      </div>
    </div>

  </div>
</div>
