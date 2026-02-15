<script>
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Search, RefreshCw, ChevronDown, Check, Box, Server } from 'lucide-svelte';
  import { cn } from '../utils.js';
  
  let { selectedModel = $bindable(), apiKey, onModelChange, className = '' } = $props();
  
  let models = $state([]);
  let filteredModels = $state([]);
  let searchQuery = $state('');
  let isOpen = $state(false);
  let isFetching = $state(false);
  let error = $state('');
  let menuRef;

  // Mock default models if fetch fails or no key
  const defaultModels = [
    { id: 'minimax/minimax-m2.5:free', name: 'Minimax M2.5' },
    { id: 'z-ai/glm-5:free', name: 'GLM-5' },
    { id: 'gemini-1.5-pro', name: 'Gemini 1.5 Pro' },
    { id: 'claude-3-opus', name: 'Claude 3 Opus' }
  ];
  
  async function fetchModels() {
    isFetching = true;
    error = '';
    try {
      // Try backend invoke
      const result = await invoke('fetch_kilo_models', { apiKey });
      models = result;
      
      localStorage.setItem('kilo-models', JSON.stringify({
        models: result,
        timestamp: Date.now()
      }));
    } catch (e) {
      console.warn('Failed to fetch models, using defaults/cache', e);
      error = 'Offline/Default Mode';
      if (models.length === 0) models = defaultModels;
    } finally {
      filteredModels = models;
      isFetching = false;
    }
  }
  
  function loadCachedModels() {
    const cached = localStorage.getItem('kilo-models');
    if (cached) {
      const { models: cachedModels, timestamp } = JSON.parse(cached);
      // Cache valid for 1 hour
      if (Date.now() - timestamp < 3600000) {
        models = cachedModels;
        filteredModels = cachedModels;
        return true;
      }
    }
    return false;
  }
  
  $effect(() => {
    if (searchQuery) {
      filteredModels = models.filter(m => 
        m.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        m.id.toLowerCase().includes(searchQuery.toLowerCase())
      );
    } else {
      filteredModels = models;
    }
  });
  
  onMount(() => {
    if (!loadCachedModels()) {
      // Only auto-fetch if we have a key, otherwise wait for user/defaults
      if (apiKey) fetchModels();
      else models = defaultModels;
    }
  });

  function handleClickOutside(event) {
    if (menuRef && !menuRef.contains(event.target)) {
      isOpen = false;
    }
  }

  $effect(() => {
    if (isOpen) window.addEventListener('click', handleClickOutside);
    else window.removeEventListener('click', handleClickOutside);
    return () => window.removeEventListener('click', handleClickOutside);
  });
</script>

<div class={cn("relative", className)} bind:this={menuRef}>
  <button 
    class="flex items-center gap-2 px-3 py-1.5 bg-muted/30 hover:bg-muted border border-border hover:border-primary/30 rounded-lg w-full transition-all group"
    onclick={() => isOpen = !isOpen}
    title="Select AI Model"
  >
    <Box size={14} class="text-primary group-hover:scale-110 transition-transform" />
    <span class="flex-1 text-left truncate text-xs font-medium text-foreground">{selectedModel || 'Select model'}</span>
    <ChevronDown size={14} class={cn("text-muted-foreground transition-transform duration-200", isOpen ? 'rotate-180' : '')} />
  </button>
  
  {#if isOpen}
    <div class="absolute top-full mt-1 w-full min-w-[240px] bg-popover border border-border rounded-lg shadow-xl z-50 flex flex-col animate-in fade-in zoom-in-95 duration-100 overflow-hidden">
      
      <!-- Search & Refresh -->
      <div class="p-2 border-b border-border flex gap-2 bg-muted/10">
        <div class="flex-1 relative">
          <Search size={12} class="absolute left-2 top-2 text-muted-foreground" />
          <input 
            type="text"
            bind:value={searchQuery}
            placeholder="Search..."
            class="w-full pl-7 pr-2 py-1.5 bg-background border border-border rounded text-xs focus:ring-1 focus:ring-primary outline-none"
            onclick={(e) => e.stopPropagation()}
          />
        </div>
        <button 
          onclick={(e) => { e.stopPropagation(); fetchModels(); }}
          disabled={isFetching}
          class="p-1.5 bg-background border border-border hover:bg-muted rounded text-muted-foreground hover:text-foreground transition-colors"
          title="Refresh models"
        >
          <RefreshCw size={12} class={isFetching ? 'animate-spin' : ''} />
        </button>
      </div>
      
      <!-- Models List -->
      <div class="overflow-y-auto max-h-60 scrollbar-thin scrollbar-thumb-muted p-1">
        {#if filteredModels.length === 0}
          <div class="p-3 text-muted-foreground text-xs text-center italic">No models found</div>
        {:else}
          <div class="text-[10px] font-semibold text-muted-foreground px-2 py-1 uppercase tracking-wider">Available</div>
          {#each filteredModels as model}
            <button
              class={cn(
                "w-full px-2 py-1.5 text-left flex items-center gap-2 rounded text-sm transition-colors group",
                selectedModel === model.id ? "bg-primary/10 text-primary" : "hover:bg-muted/50 text-foreground"
              )}
              onclick={() => {
                selectedModel = model.id;
                isOpen = false;
                onModelChange?.(model);
              }}
            >
              <Server size={14} class={cn("opacity-70", selectedModel === model.id ? "text-primary" : "text-muted-foreground")} />
              <div class="flex-1 flex flex-col min-w-0">
                 <span class="truncate text-xs font-medium">{model.name}</span>
                 <span class="truncate text-[10px] opacity-60 font-mono">{model.id}</span>
              </div>
              {#if selectedModel === model.id}
                <Check size={12} class="text-primary" />
              {/if}
            </button>
          {/each}
        {/if}
      </div>
      
      <!-- Footer -->
      <div class="p-1.5 border-t border-border text-[10px] text-muted-foreground bg-muted/10 flex justify-between px-3">
        <span>{filteredModels.length} models</span>
        <span class="opacity-50">{error ? 'Default' : 'Synced'}</span>
      </div>
    </div>
  {/if}
</div>
