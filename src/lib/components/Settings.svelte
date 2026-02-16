<script>
  import { invoke } from '@tauri-apps/api/core';
  import { configStore } from '../stores/config.svelte.js';
  import { themeStore, themes } from '../stores/theme.svelte.js';
  import { 
    X, Save, Settings as SettingsIcon, Brain, Monitor, 
    Moon, Sun, Palette, Keyboard, ShieldCheck, Zap,
    Check, Loader2, ChevronDown
  } from 'lucide-svelte';
  import IntelligenceTab from './IntelligenceTab.svelte';
  import { cn } from '../utils.js';

  let { onClose } = $props();
  
  let activeTab = $state('appearance'); // Default to appearance as requested for development

  // General Settings
  let provider = $state(configStore.settings.ai.provider);
  let apiKey = $state(configStore.settings.ai.apiKey);
  let baseUrl = $state(configStore.settings.ai.baseUrl);
  let selectedModel = $state(configStore.settings.ai.selectedModel);
  let isSaving = $state(false);

  async function saveSettings() {
    isSaving = true;
    try {
      configStore.updateAiConfig({
        provider,
        apiKey,
        baseUrl,
        selectedModel
      });
      configStore.updateSettings({ theme: themeStore.activeTheme });
      onClose?.();
    } catch (e) {
      console.error('Failed to save settings:', e);
    } finally {
      isSaving = false;
    }
  }

  const tabs = [
    { id: 'appearance', label: 'Appearance', icon: Palette },
    { id: 'general', label: 'AI & Core', icon: Zap },
    { id: 'intelligence', label: 'Intelligence', icon: Brain },
    { id: 'security', label: 'Security', icon: ShieldCheck },
  ];
</script>

<div 
  class="fixed inset-0 bg-black/60 backdrop-blur-md flex items-center justify-center z-[1000] animate-in fade-in duration-300"
  onclick={onClose}
>
  <div 
    class="bg-background border border-border/60 shadow-[0_0_50px_-12px_rgba(0,0,0,0.5)] rounded-3xl w-full max-w-4xl h-[650px] flex flex-col overflow-hidden animate-in zoom-in-95 duration-300"
    onclick={(e) => e.stopPropagation()}
  >
    <!-- Header -->
    <div class="h-16 px-8 border-b border-border/40 flex items-center justify-between bg-muted/5">
      <div class="flex items-center gap-4">
        <div class="w-10 h-10 rounded-2xl bg-primary/10 flex items-center justify-center text-primary border border-primary/20 shadow-inner">
          <SettingsIcon size={20} />
        </div>
        <div>
          <h2 class="text-sm font-black tracking-tight uppercase">Settings</h2>
          <p class="text-[10px] text-muted-foreground font-medium uppercase tracking-[0.1em]">Configuration & Environment</p>
        </div>
      </div>
      <button 
        class="p-2 hover:bg-destructive/10 hover:text-destructive rounded-full text-muted-foreground transition-all duration-200"
        onclick={onClose}
      >
        <X size={20} />
      </button>
    </div>

    <div class="flex-1 flex min-h-0">
      <!-- Sidebar Tabs -->
      <div class="w-56 border-r border-border/40 bg-muted/5 p-4 space-y-1.5">
        {#each tabs as tab}
          <button
            class={cn(
              "w-full flex items-center gap-3 px-4 py-2.5 text-[11px] font-bold uppercase tracking-wider rounded-xl transition-all duration-300",
              activeTab === tab.id 
                ? "bg-primary text-primary-foreground shadow-lg shadow-primary/20 scale-[1.02]" 
                : "text-muted-foreground hover:bg-muted hover:text-foreground"
            )}
            onclick={() => activeTab = tab.id}
          >
            <tab.icon size={14} strokeWidth={2.5} />
            {tab.label}
          </button>
        {/each}
      </div>

      <!-- Main Content Area -->
      <div class="flex-1 overflow-y-auto p-8 space-y-10 scroll-smooth no-scrollbar">
        {#if activeTab === 'appearance'}
          <div class="space-y-8 animate-in slide-in-from-right-4 duration-500">
            <section>
              <header class="mb-6">
                <h3 class="text-lg font-bold tracking-tight">Theme Gallery</h3>
                <p class="text-xs text-muted-foreground">Select a visual identity for your workspace.</p>
              </header>

              <div class="grid grid-cols-2 gap-4">
                {#each Object.entries(themes) as [id, theme]}
                  <button
                    class={cn(
                      "group relative flex flex-col rounded-2xl border transition-all duration-500 overflow-hidden text-left bg-card/50",
                      themeStore.activeTheme === id 
                        ? "border-primary ring-1 ring-primary/50 shadow-xl shadow-primary/5" 
                        : "border-border/40 hover:border-border hover:shadow-lg"
                    )}
                    onclick={() => themeStore.setTheme(id)}
                  >
                    <!-- Theme Preview Layout -->
                    <div class="h-24 w-full p-2 gap-1.5 flex flex-col bg-muted/20 border-b border-border/20 group-hover:bg-muted/10 transition-colors" style="background-color: hsl({theme.colors['--background']})">
                       <div class="h-2 w-full rounded-sm opacity-20" style="background-color: hsl({theme.colors['--foreground']})"></div>
                       <div class="flex flex-1 gap-1.5">
                          <div class="w-8 h-full rounded-sm opacity-10" style="background-color: hsl({theme.colors['--primary']})"></div>
                          <div class="flex-1 flex flex-col gap-1.5">
                             <div class="h-full rounded-sm opacity-20 relative overflow-hidden" style="background-color: hsl({theme.colors['--foreground']})">
                                <div class="absolute top-2 left-2 w-1/2 h-1 rounded-full opacity-40" style="background-color: hsl({theme.colors['--primary']})"></div>
                             </div>
                          </div>
                       </div>
                    </div>

                    <!-- Theme Info -->
                    <div class="p-3 flex items-center justify-between">
                      <div class="flex flex-col">
                        <span class="text-xs font-bold tracking-tight">{theme.name}</span>
                        <span class="text-[9px] uppercase font-bold tracking-[0.1em] opacity-40">{theme.type} mode</span>
                      </div>
                      
                      <!-- Color Swatches -->
                      <div class="flex -space-x-1.5">
                         <div class="w-4 h-4 rounded-full border border-black/10 shadow-sm" style="background-color: hsl({theme.colors['--primary']})"></div>
                         <div class="w-4 h-4 rounded-full border border-black/10 shadow-sm" style="background-color: hsl({theme.colors['--secondary']})"></div>
                         <div class="w-4 h-4 rounded-full border border-black/10 shadow-sm" style="background-color: hsl({theme.colors['--accent']})"></div>
                      </div>
                    </div>

                    {#if themeStore.activeTheme === id}
                      <div class="absolute top-2 right-2 p-1 bg-primary text-primary-foreground rounded-full shadow-lg scale-110 animate-in zoom-in duration-300">
                        <Check size={10} strokeWidth={4} />
                      </div>
                    {/if}
                  </button>
                {/each}
              </div>
            </section>

            <section class="pt-4">
               <div class="flex items-center justify-between p-5 bg-muted/10 rounded-2xl border border-border/30 backdrop-blur-sm group hover:bg-muted/20 transition-all duration-300">
                  <div class="flex items-center gap-4">
                     <div class="w-10 h-10 rounded-xl bg-background border border-border/50 flex items-center justify-center text-muted-foreground group-hover:text-primary transition-colors">
                        <Monitor size={20} />
                     </div>
                     <div class="flex flex-col">
                        <span class="text-xs font-bold tracking-tight">System Synchronization</span>
                        <span class="text-[10px] text-muted-foreground leading-tight">Automatically switch themes based on your OS settings.</span>
                     </div>
                  </div>
                  <div class="w-10 h-5 bg-muted border border-border rounded-full relative cursor-pointer opacity-50 group-hover:opacity-100 transition-opacity">
                     <div class="absolute left-1 top-1 w-3 h-3 bg-muted-foreground rounded-full"></div>
                  </div>
               </div>
            </section>
          </div>

        {:else if activeTab === 'general'}
          <div class="space-y-8 animate-in slide-in-from-right-4 duration-500">
            <header>
              <h3 class="text-lg font-bold tracking-tight">AI & Integration</h3>
              <p class="text-xs text-muted-foreground">Configure how Karsa communicates with LLMs.</p>
            </header>

            <div class="grid gap-6">
              <div class="space-y-2">
                <label class="text-[10px] font-black uppercase tracking-[0.15em] text-muted-foreground/70 px-1">Active Provider</label>
                <div class="relative group">
                  <select 
                    bind:value={provider} 
                    class="w-full bg-muted/30 border border-border/50 rounded-xl px-4 py-3 text-sm outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary/40 transition-all appearance-none cursor-pointer text-foreground shadow-inner"
                  >
                    <option value="kilo" class="bg-background text-foreground py-2">Kilo Code (Native Cloud)</option>
                    <option value="ollama" class="bg-background text-foreground py-2">Ollama (Local Infrastructure)</option>
                    <option value="openai" class="bg-background text-foreground py-2">OpenAI (Global Gateway)</option>
                    <option value="custom" class="bg-background text-foreground py-2">Custom LLM Endpoint</option>
                  </select>
                  <ChevronDown size={16} class="absolute right-4 top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none group-focus-within:rotate-180 transition-transform duration-300" />
                </div>
              </div>

              <div class="space-y-2">
                <label class="text-[10px] font-black uppercase tracking-[0.15em] text-muted-foreground/70 px-1">Security Token (API Key)</label>
                <div class="relative">
                  <input 
                    type="password" 
                    bind:value={apiKey}
                    placeholder="Enter your encrypted token..."
                    class="w-full bg-muted/30 border border-border/50 rounded-xl px-4 py-3 text-sm outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary/40 transition-all shadow-inner placeholder:text-muted-foreground/30"
                  />
                </div>
              </div>

              <div class="space-y-2">
                <label class="text-[10px] font-black uppercase tracking-[0.15em] text-muted-foreground/70 px-1">Gateway Architecture (Base URL)</label>
                <input 
                  type="text" 
                  bind:value={baseUrl}
                  placeholder="https://gateway.karsa.ai/v1"
                  class="w-full bg-muted/30 border border-border/50 rounded-xl px-4 py-3 text-sm outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary/40 transition-all shadow-inner placeholder:text-muted-foreground/30"
                />
              </div>
            </div>
          </div>

        {:else if activeTab === 'intelligence'}
          <div class="animate-in slide-in-from-right-4 duration-500">
            <IntelligenceTab />
          </div>
        {/if}
      </div>
    </div>

    <!-- Footer -->
    <div class="h-20 px-8 border-t border-border/40 flex items-center justify-between bg-muted/5">
      <div class="flex items-center gap-2 text-muted-foreground/40 italic">
         <Keyboard size={14} />
         <span class="text-[10px]">Press Esc to dismiss without saving</span>
      </div>
      <div class="flex items-center gap-4">
        <button 
          class="px-6 py-2.5 text-xs font-bold text-muted-foreground hover:text-foreground transition-colors uppercase tracking-widest"
          onclick={onClose}
        >
          Discard
        </button>
        <button 
          class="px-8 py-2.5 text-xs font-black uppercase tracking-widest bg-primary text-primary-foreground rounded-xl shadow-xl shadow-primary/20 hover:shadow-primary/30 hover:bg-primary/90 active:scale-95 transition-all flex items-center gap-3 border border-primary/20"
          onclick={saveSettings}
          disabled={isSaving}
        >
          {#if isSaving}
            <Loader2 size={16} class="animate-spin" />
            Synchronizing...
          {:else}
            <Save size={16} strokeWidth={2.5} />
            Commit Changes
          {/if}
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  .no-scrollbar::-webkit-scrollbar {
    display: none;
  }
  
  /* Fix for the white background bug in dark theme dropdowns */
  select {
    color-scheme: dark;
  }
  
  :global(.light) select {
    color-scheme: light;
  }

  select option {
    background-color: var(--card);
    color: var(--foreground);
    padding: 12px;
  }
</style>
