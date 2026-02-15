<script>
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { fsStore } from '../stores/fileSystem.svelte.js';
  import { FilePlus, FolderOpen, FileCode, GitBranch, Clock, Command } from 'lucide-svelte';
  
  let recents = $state([
    { name: 'KarsaIDE', path: '/home/user/projects/karsa-ide', lastOpened: '2 hours ago' },
    { name: 'website-v2', path: '/home/user/work/website-v2', lastOpened: 'Yesterday' },
    { name: 'rust-learning', path: '/home/user/learn/rust', lastOpened: '3 days ago' },
  ]);

  async function openFolder() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });
      if (selected) {
        fsStore.setProjectDir(selected);
      }
    } catch (err) {
      console.error(err);
    }
  }

  async function openFile() {
    try {
      const selected = await open({
        multiple: false,
      });
      if (selected) {
        fsStore.openFile(selected);
      }
    } catch (err) {
      console.error(err);
    }
  }
</script>

<div class="h-full flex flex-col bg-background text-foreground overflow-y-auto p-12 animate-in fade-in duration-500">
  <div class="max-w-4xl mx-auto w-full space-y-12">
    
    <!-- Hero -->
    <div class="space-y-4 text-center sm:text-left">
      <h1 class="text-4xl font-bold tracking-tight bg-gradient-to-r from-primary to-purple-400 bg-clip-text text-transparent w-fit">
        Karsa IDE
      </h1>
      <p class="text-muted-foreground text-lg max-w-2xl">
        AI-native code editor designed for speed and autonomy.
      </p>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
      <!-- Quick Actions -->
      <div class="space-y-4">
        <h2 class="text-xl font-semibold flex items-center gap-2">
          <Command size={20} class="text-primary" /> Start
        </h2>
        <div class="grid gap-3">
          <button onclick={openFolder} class="group flex items-center gap-4 p-4 rounded-xl border border-border bg-card hover:bg-muted/50 transition-all hover:border-primary/50 text-left">
            <div class="p-3 rounded-lg bg-primary/10 text-primary group-hover:bg-primary group-hover:text-primary-foreground transition-colors">
              <FolderOpen size={24} />
            </div>
            <div>
              <div class="font-medium">Open Folder</div>
              <div class="text-xs text-muted-foreground">Navigate to an existing project</div>
            </div>
          </button>

          <button onclick={openFile} class="group flex items-center gap-4 p-4 rounded-xl border border-border bg-card hover:bg-muted/50 transition-all hover:border-primary/50 text-left">
            <div class="p-3 rounded-lg bg-purple-500/10 text-purple-400 group-hover:bg-purple-500 group-hover:text-white transition-colors">
              <FileCode size={24} />
            </div>
            <div>
              <div class="font-medium">Open File</div>
              <div class="text-xs text-muted-foreground">Edit a single file</div>
            </div>
          </button>

          <button class="group flex items-center gap-4 p-4 rounded-xl border border-border bg-card hover:bg-muted/50 transition-all hover:border-primary/50 text-left opacity-50 cursor-not-allowed">
            <div class="p-3 rounded-lg bg-green-500/10 text-green-400 group-hover:bg-green-500 group-hover:text-white transition-colors">
              <GitBranch size={24} />
            </div>
            <div>
              <div class="font-medium">Clone Repository</div>
              <div class="text-xs text-muted-foreground">Get code from Git</div>
            </div>
          </button>
        </div>
      </div>

      <!-- Recent Projects -->
      <div class="space-y-4">
        <h2 class="text-xl font-semibold flex items-center gap-2">
          <Clock size={20} class="text-primary" /> Recent
        </h2>
        <div class="flex flex-col gap-2">
          {#each recents as item}
            <button class="flex items-center justify-between p-3 rounded-lg hover:bg-muted/50 text-left group transition-colors">
              <div class="flex flex-col">
                <span class="font-medium group-hover:text-primary transition-colors">{item.name}</span>
                <span class="text-xs text-muted-foreground truncate max-w-[200px]">{item.path}</span>
              </div>
              <span class="text-xs text-muted-foreground whitespace-nowrap">{item.lastOpened}</span>
            </button>
          {/each}
          <div class="pt-2 text-center">
             <button class="text-xs text-primary hover:underline">View all history</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Footer / Tips -->
    <div class="pt-8 border-t border-border grid grid-cols-1 md:grid-cols-3 gap-6 text-sm">
      <div class="space-y-2">
        <h3 class="font-semibold text-foreground">Pro Tip</h3>
        <p class="text-muted-foreground">Use <kbd class="px-1.5 py-0.5 rounded bg-muted border border-border text-xs font-mono">Cmd+P</kbd> to quickly search for files in your project.</p>
      </div>
      <div class="space-y-2">
        <h3 class="font-semibold text-foreground">AI Assist</h3>
        <p class="text-muted-foreground">Highlight code and press <kbd class="px-1.5 py-0.5 rounded bg-muted border border-border text-xs font-mono">Cmd+L</kbd> to chat with Karsa.</p>
      </div>
       <div class="space-y-2">
        <h3 class="font-semibold text-foreground">Status</h3>
        <div class="flex items-center gap-2 text-muted-foreground">
           <span class="relative flex h-2 w-2">
              <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
              <span class="relative inline-flex rounded-full h-2 w-2 bg-green-500"></span>
            </span>
            Karsa Engine Active
        </div>
      </div>
    </div>
  </div>
</div>
