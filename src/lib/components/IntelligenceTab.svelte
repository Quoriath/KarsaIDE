<script>
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { Brain, RefreshCw, CheckCircle, Loader2, Database, Code2, Layers } from 'lucide-svelte';
  
  let stats = $state(null);
  let loading = $state(true);
  let reindexing = $state(false);
  
  async function loadStats() {
    try {
      stats = await invoke('get_codebase_stats');
      loading = false;
    } catch (e) {
      console.error('Failed to load stats:', e);
      loading = false;
    }
  }
  
  async function forceReindex() {
    reindexing = true;
    try {
      await invoke('force_reindex');
      setTimeout(loadStats, 2000);
    } catch (e) {
      console.error('Failed to reindex:', e);
    } finally {
      reindexing = false;
    }
  }
  
  onMount(() => {
    loadStats();
    const interval = setInterval(loadStats, 5000);
    return () => clearInterval(interval);
  });
</script>

<div class="space-y-8 animate-in slide-in-from-right-4 duration-500">
  <header class="flex items-center justify-between">
    <div>
      <h3 class="text-lg font-bold tracking-tight">Codebase Intelligence</h3>
      <p class="text-xs text-muted-foreground">Monitoring your project's semantic structure.</p>
    </div>
    
    <button
      onclick={forceReindex}
      disabled={reindexing}
      class="px-4 py-2 bg-primary/10 text-primary border border-primary/20 rounded-xl text-xs font-bold flex items-center gap-2 hover:bg-primary/20 transition-all active:scale-95 disabled:opacity-50 uppercase tracking-widest"
    >
      {#if reindexing}
        <Loader2 size={14} class="animate-spin" />
        Indexing...
      {:else}
        <RefreshCw size={14} />
        Re-Index
      {/if}
    </button>
  </header>
  
  {#if loading}
    <div class="flex flex-col items-center justify-center py-20 space-y-4 opacity-40">
      <Loader2 size={40} class="animate-spin text-primary" />
      <span class="text-xs font-medium uppercase tracking-[0.2em]">Scanning Architecture...</span>
    </div>
  {:else if stats}
    <div class="grid gap-6">
      <!-- Status Card -->
      <div class="p-5 bg-muted/10 rounded-2xl border border-border/30 flex items-center justify-between backdrop-blur-sm shadow-inner">
        <div class="flex items-center gap-4">
           <div class="w-10 h-10 rounded-xl bg-background border border-border/50 flex items-center justify-center">
              <Database size={20} class={stats.status === 'Ready' ? 'text-green-500' : 'text-blue-500 animate-pulse'} />
           </div>
           <div class="flex flex-col">
              <span class="text-[10px] font-black uppercase tracking-widest text-muted-foreground/60">System Status</span>
              <span class="text-sm font-bold">{stats.status === 'Ready' ? 'Operational' : 'Processing...'}</span>
           </div>
        </div>
        <div class="flex items-center gap-2 bg-background/50 px-3 py-1 rounded-full border border-border/40">
           {#if stats.status === 'Ready'}
             <div class="w-2 h-2 rounded-full bg-green-500"></div>
           {:else}
             <Loader2 size={12} class="animate-spin text-blue-500" />
           {/if}
           <span class="text-[10px] font-bold uppercase">{stats.status}</span>
        </div>
      </div>
      
      <!-- Stats Grid -->
      <div class="grid grid-cols-2 gap-4">
        <div class="p-6 bg-card/30 border border-border/40 rounded-3xl relative overflow-hidden group hover:border-primary/30 transition-colors">
          <div class="absolute -right-4 -bottom-4 opacity-[0.03] group-hover:opacity-[0.08] transition-opacity">
             <Layers size={80} />
          </div>
          <div class="text-3xl font-black tracking-tighter mb-1">{stats.indexed_files}</div>
          <div class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest flex items-center gap-2">
             <div class="w-1 h-1 rounded-full bg-primary"></div>
             Entities Mapped
          </div>
        </div>
        
        <div class="p-6 bg-card/30 border border-border/40 rounded-3xl relative overflow-hidden group hover:border-primary/30 transition-colors">
          <div class="absolute -right-4 -bottom-4 opacity-[0.03] group-hover:opacity-[0.08] transition-opacity">
             <Code2 size={80} />
          </div>
          <div class="text-3xl font-black tracking-tighter mb-1">{stats.total_lines.toLocaleString()}</div>
          <div class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest flex items-center gap-2">
             <div class="w-1 h-1 rounded-full bg-blue-500"></div>
             Total Tokens Analyzed
          </div>
        </div>
      </div>
      
      <!-- Languages -->
      <section class="space-y-4">
        <h4 class="text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground px-1">Language Distribution</h4>
        <div class="grid grid-cols-1 gap-2">
          {#each Object.entries(stats.languages).sort((a, b) => b[1] - a[1]) as [lang, count]}
            <div class="flex items-center justify-between p-3 px-4 bg-muted/5 hover:bg-muted/10 border border-border/20 rounded-xl transition-all group">
              <div class="flex items-center gap-3">
                 <div class="w-2 h-2 rounded-full bg-primary/40 group-hover:bg-primary transition-colors"></div>
                 <span class="text-xs font-bold text-muted-foreground group-hover:text-foreground transition-colors uppercase tracking-wide">{lang}</span>
              </div>
              <span class="text-[10px] font-mono opacity-40 group-hover:opacity-100 transition-opacity bg-background px-2 py-0.5 rounded border border-border/40">{count} files</span>
            </div>
          {/each}
        </div>
      </section>
    </div>
  {:else}
    <div class="flex flex-col items-center justify-center py-20 space-y-6 opacity-30 text-center">
      <div class="p-6 rounded-full bg-muted/20 border border-border/40">
         <Database size={40} />
      </div>
      <div class="space-y-2">
        <p class="text-sm font-bold uppercase tracking-widest">Workspace Disconnected</p>
        <p class="text-xs max-w-[240px] leading-relaxed">Initialize a project directory to enable deep semantic code analysis.</p>
      </div>
    </div>
  {/if}
</div>
