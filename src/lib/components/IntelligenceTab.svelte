<script>
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { Brain, RefreshCw, CheckCircle, Loader2 } from 'lucide-svelte';
  
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
    const interval = setInterval(loadStats, 3000);
    return () => clearInterval(interval);
  });
</script>

<div class="p-6 space-y-6">
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-2">
      <Brain size={20} class="text-primary" />
      <h3 class="text-lg font-semibold">Codebase Intelligence</h3>
    </div>
    
    <button
      onclick={forceReindex}
      disabled={reindexing}
      class="px-3 py-1.5 bg-primary text-primary-foreground rounded text-sm flex items-center gap-2 hover:bg-primary/90 disabled:opacity-50"
    >
      {#if reindexing}
        <Loader2 size={14} class="animate-spin" />
      {:else}
        <RefreshCw size={14} />
      {/if}
      Force Re-Index
    </button>
  </div>
  
  {#if loading}
    <div class="flex items-center justify-center py-12">
      <Loader2 size={32} class="animate-spin text-muted-foreground" />
    </div>
  {:else if stats}
    <div class="space-y-4">
      <!-- Status -->
      <div class="p-4 bg-muted/30 rounded-lg">
        <div class="flex items-center justify-between">
          <span class="text-sm text-muted-foreground">Status</span>
          <div class="flex items-center gap-2">
            {#if stats.status === 'Ready'}
              <CheckCircle size={16} class="text-green-500" />
            {:else}
              <Loader2 size={16} class="animate-spin text-blue-500" />
            {/if}
            <span class="font-medium">{stats.status}</span>
          </div>
        </div>
      </div>
      
      <!-- Stats Grid -->
      <div class="grid grid-cols-2 gap-4">
        <div class="p-4 bg-muted/30 rounded-lg">
          <div class="text-2xl font-bold">{stats.indexed_files}</div>
          <div class="text-xs text-muted-foreground mt-1">Indexed Files</div>
        </div>
        
        <div class="p-4 bg-muted/30 rounded-lg">
          <div class="text-2xl font-bold">{stats.total_lines.toLocaleString()}</div>
          <div class="text-xs text-muted-foreground mt-1">Total Lines</div>
        </div>
      </div>
      
      <!-- Languages -->
      <div class="p-4 bg-muted/30 rounded-lg">
        <div class="text-sm font-medium mb-3">Languages Detected</div>
        <div class="space-y-2">
          {#each Object.entries(stats.languages).sort((a, b) => b[1] - a[1]) as [lang, count]}
            <div class="flex items-center justify-between text-sm">
              <span class="text-muted-foreground">{lang}</span>
              <span class="font-medium">{count} files</span>
            </div>
          {/each}
        </div>
      </div>
    </div>
  {:else}
    <div class="text-center py-12 text-muted-foreground">
      <p>No active workspace</p>
      <p class="text-xs mt-2">Open a folder to start indexing</p>
    </div>
  {/if}
</div>
