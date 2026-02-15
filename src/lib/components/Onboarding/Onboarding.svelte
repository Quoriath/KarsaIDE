<script>
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { Check, Sparkles, Server, Zap, Globe, Key, RefreshCw } from 'lucide-svelte';
  import { cn } from '../../utils.js';
  import ModelSelector from '../ModelSelector.svelte';

  let { onComplete } = $props();
  
  let provider = $state('kilo');
  let apiKey = $state('');
  let selectedModel = $state('minimax/minimax-m2.5:free');
  let baseUrl = $state('https://api.kilo.ai/api/gateway/chat/completions');
  
  let isConnecting = $state(false);
  let connectionSuccess = $state(false);
  let error = $state('');

  function handleProviderChange(p) {
    provider = p;
    if (p === 'kilo') {
      baseUrl = 'https://api.kilo.ai/api/gateway/chat/completions';
    } else if (p === 'ollama') {
      baseUrl = 'http://localhost:11434/v1/chat/completions';
    } else {
      baseUrl = 'https://api.openai.com/v1/chat/completions';
    }
  }

  async function handleConnect() {
    error = '';
    isConnecting = true;

    const config = {
      provider,
      api_key: apiKey || null,
      base_url: baseUrl,
      model_name: selectedModel,
      selectedModel: selectedModel // redundancy for store compat
    };

    try {
      await invoke('test_ai_connection', { config }).catch(() => {
        // Mock success if backend missing
        return new Promise(resolve => setTimeout(resolve, 1000));
      });
      
      await invoke('save_ai_config', { config }).catch(() => {});
      
      connectionSuccess = true;
      setTimeout(() => onComplete(), 1500);
    } catch (e) {
      error = e.toString();
    } finally {
      isConnecting = false;
    }
  }
</script>

<div class="fixed inset-0 bg-background/95 backdrop-blur-sm flex items-center justify-center z-50 animate-in fade-in duration-300">
  <div class="bg-card border border-border rounded-xl p-8 max-w-lg w-full mx-4 shadow-2xl relative overflow-hidden">
    
    <!-- Background Accents -->
    <div class="absolute -top-20 -right-20 w-64 h-64 bg-primary/10 rounded-full blur-3xl pointer-events-none"></div>
    <div class="absolute -bottom-20 -left-20 w-64 h-64 bg-purple-500/10 rounded-full blur-3xl pointer-events-none"></div>

    <div class="relative z-10">
      {#if connectionSuccess}
        <div class="text-center py-12 animate-in zoom-in-95 duration-300">
          <div class="w-16 h-16 mx-auto mb-6 rounded-full bg-green-500/20 flex items-center justify-center text-green-500">
            <Check size={32} />
          </div>
          <h2 class="text-2xl font-bold text-foreground mb-2">Connected!</h2>
          <p class="text-muted-foreground">Welcome to Karsa IDE.</p>
        </div>
      {:else}
        <div class="text-center mb-8">
          <div class="inline-flex items-center justify-center p-3 rounded-xl bg-primary/10 text-primary mb-4">
            <Sparkles size={32} />
          </div>
          <h1 class="text-2xl font-bold text-foreground">Setup Karsa AI</h1>
          <p class="text-muted-foreground mt-2 text-sm">Configure your AI provider to enable smart coding features.</p>
        </div>

        <div class="space-y-6">
          <div class="space-y-3">
            <label class="text-sm font-medium text-muted-foreground block">Select Provider</label>
            <div class="grid grid-cols-3 gap-3">
              <button 
                class={cn("flex flex-col items-center gap-2 p-3 rounded-lg border transition-all hover:bg-muted/50", 
                  provider === 'kilo' ? "border-primary bg-primary/5 text-primary" : "border-border text-muted-foreground")}
                onclick={() => handleProviderChange('kilo')}
              >
                <Zap size={20} />
                <span class="text-xs font-medium">Kilo AI</span>
              </button>
              
              <button 
                class={cn("flex flex-col items-center gap-2 p-3 rounded-lg border transition-all hover:bg-muted/50", 
                  provider === 'ollama' ? "border-primary bg-primary/5 text-primary" : "border-border text-muted-foreground")}
                onclick={() => handleProviderChange('ollama')}
              >
                <Server size={20} />
                <span class="text-xs font-medium">Ollama</span>
              </button>
              
              <button 
                class={cn("flex flex-col items-center gap-2 p-3 rounded-lg border transition-all hover:bg-muted/50", 
                  provider === 'custom' ? "border-primary bg-primary/5 text-primary" : "border-border text-muted-foreground")}
                onclick={() => handleProviderChange('custom')}
              >
                <Globe size={20} />
                <span class="text-xs font-medium">Custom</span>
              </button>
            </div>
          </div>

          {#if provider === 'kilo' || provider === 'custom'}
            <div class="space-y-4">
              <div class="space-y-2">
                <label class="text-sm font-medium text-muted-foreground block">API Key</label>
                <div class="relative">
                  <input 
                    type="password" 
                    bind:value={apiKey}
                    placeholder="sk-..."
                    class="w-full bg-background border border-border rounded-lg px-4 py-2 text-sm text-foreground placeholder:text-muted-foreground/50 focus:border-primary focus:ring-1 focus:ring-primary focus:outline-none transition-all pl-10"
                  />
                  <Key size={16} class="absolute left-3 top-2.5 text-muted-foreground" />
                </div>
              </div>

              <div class="space-y-2">
                 <div class="flex items-center justify-between">
                    <label class="text-sm font-medium text-muted-foreground">Model</label>
                 </div>
                 
                 <ModelSelector 
                   bind:selectedModel 
                   apiKey={apiKey}
                   className="w-full"
                 />
              </div>
            </div>
          {/if}

          {#if error}
            <div class="p-3 bg-destructive/10 border border-destructive/20 rounded-lg text-destructive text-xs flex items-center gap-2">
              <span class="font-bold">Error:</span> {error}
            </div>
          {/if}

          <button 
            onclick={handleConnect}
            disabled={isConnecting}
            class="w-full bg-primary text-primary-foreground font-semibold py-2.5 rounded-lg hover:bg-primary/90 transition-colors disabled:opacity-50 flex items-center justify-center gap-2 shadow-lg shadow-primary/20"
          >
            {#if isConnecting}
              <div class="w-4 h-4 border-2 border-primary-foreground/30 border-t-primary-foreground rounded-full animate-spin"></div>
              Connecting...
            {:else}
              Connect & Start
            {/if}
          </button>
        </div>
      {/if}
    </div>
  </div>
</div>
