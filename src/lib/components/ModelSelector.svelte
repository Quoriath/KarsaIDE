<script>
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Search, RefreshCw, ChevronDown, Check, Cpu, Server, Sparkles } from 'lucide-svelte';
  import { cn } from '../utils.js';
  
  let { selectedModel = $bindable(), apiKey, onModelChange, className = '' } = $props();
  
  let models = $state([]);
  let filteredModels = $state([]);
  let searchQuery = $state('');
  let isOpen = $state(false);
  let isFetching = $state(false);
  let error = $state('');
  let menuRef;

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
      const result = await invoke('fetch_kilo_models', { apiKey });
      models = result;
      localStorage.setItem('kilo-models', JSON.stringify({ models: result, timestamp: Date.now() }));
    } catch (e) {
      error = 'Offline Mode';
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
      if (apiKey) fetchModels();
      else models = defaultModels;
    }
  });

  function handleClickOutside(event) {
    if (menuRef && !menuRef.contains(event.target)) isOpen = false;
  }

  $effect(() => {
    if (isOpen) window.addEventListener('click', handleClickOutside);
    else window.removeEventListener('click', handleClickOutside);
    return () => window.removeEventListener('click', handleClickOutside);
  });
</script>

<div class={cn("relative", className)} bind:this={menuRef}>
  <button 
    class="flex items-center gap-2.5 px-3.5 py-2 bg-muted/20 hover:bg-muted/40 border border-border/40 hover:border-primary/40 rounded-xl w-full transition-all group shadow-sm active:scale-[0.98]"
    onclick={() => isOpen = !isOpen}
  >
    <div class="p-1 rounded-md bg-primary/10 text-primary group-hover:bg-primary group-hover:text-primary-foreground transition-all">
       <Cpu size={14} class="group-hover:scale-110 transition-transform" />
    </div>
    <div class="flex-1 text-left min-w-0">
       <div class="text-[10px] text-muted-foreground font-bold uppercase tracking-widest leading-none mb-0.5 opacity-50">AI Model</div>
       <div class="truncate text-xs font-bold text-foreground leading-none">{selectedModel?.split('/').pop() || 'Select model'}</div>
    </div>
    <ChevronDown size={14} class={cn("text-muted-foreground/50 transition-transform duration-300", isOpen ? 'rotate-180' : '')} />
  </button>
  
  {#if isOpen}
    <div class="absolute top-[110%] left-0 w-full min-w-[280px] bg-popover/95 backdrop-blur-2xl border border-border/60 rounded-2xl shadow-2xl z-50 flex flex-col animate-in fade-in zoom-in-95 duration-200 overflow-hidden origin-top ring-1 ring-black/5">
      
      <!-- Search & Refresh -->
      <div class="p-3 border-b border-border/40 flex gap-2 bg-muted/5">
        <div class="flex-1 relative">
          <Search size={12} class="absolute left-2.5 top-2.5 text-muted-foreground/40" />
          <input 
            type="text"
            bind:value={searchQuery}
            placeholder="Search models..."
            class="w-full pl-8 pr-2 py-2 bg-background/50 border border-border/40 rounded-lg text-xs focus:ring-2 focus:ring-primary/20 focus:border-primary/40 outline-none transition-all"
            onclick={(e) => e.stopPropagation()}
          />
        </div>
        <button 
          onclick={(e) => { e.stopPropagation(); fetchModels(); }}
          disabled={isFetching}
          class="p-2 bg-background/50 border border-border/40 hover:bg-muted rounded-lg text-muted-foreground hover:text-foreground transition-all disabled:opacity-50"
        >
          <RefreshCw size={14} class={isFetching ? 'animate-spin' : ''} />
        </button>
      </div>
      
      <!-- Models List -->
      <div class="overflow-y-auto max-h-72 scrollbar-thin scrollbar-thumb-muted p-1.5 space-y-1">
        {#if filteredModels.length === 0}
          <div class="p-8 text-muted-foreground text-xs text-center flex flex-col items-center gap-3">
             <Sparkles size={24} class="opacity-20" />
             <span class="italic">No models matching your search</span>
          </div>
        {:else}
          <div class="text-[9px] font-black text-muted-foreground px-3 py-2 uppercase tracking-[0.2em] opacity-50">Select Intelligence</div>
          {#each filteredModels as model}
            <button
              class={cn(
                "w-full px-3 py-2.5 text-left flex items-center gap-3 rounded-xl transition-all group",
                selectedModel === model.id ? "bg-primary/10 text-primary shadow-inner" : "hover:bg-muted/50 text-foreground/80 hover:text-foreground"
              )}
              onclick={() => {
                selectedModel = model.id;
                isOpen = false;
                onModelChange?.(model);
              }}
            >
              <div class={cn(
                "w-8 h-8 rounded-lg flex items-center justify-center border transition-all",
                selectedModel === model.id ? "bg-primary text-primary-foreground border-primary/20 shadow-lg shadow-primary/20" : "bg-background border-border/40 group-hover:border-primary/30"
              )}>
                 <Server size={14} />
              </div>
              <div class="flex-1 flex flex-col min-w-0">
                 <span class="truncate text-xs font-bold leading-tight">{model.name}</span>
                 <span class="truncate text-[9px] opacity-40 font-mono tracking-tighter mt-0.5">{model.id}</span>
              </div>
              {#if selectedModel === model.id}
                <div class="w-5 h-5 rounded-full bg-primary/20 flex items-center justify-center">
                   <Check size={12} class="text-primary" />
                </div>
              {/if}
            </button>
          {/each}
        {/if}
      </div>
      
      <!-- Footer -->
      <div class="p-2 border-t border-border/40 text-[9px] text-muted-foreground bg-muted/5 flex justify-between px-4 font-bold uppercase tracking-widest">
        <span>{filteredModels.length} Entities</span>
        <span class={cn(error ? 'text-orange-500' : 'text-green-500 opacity-50')}>{error || 'Secure Sync'}</span>
      </div>
    </div>
  {/if}
</div>
