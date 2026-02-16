<script>
  import { invoke } from '@tauri-apps/api/core';
  import { configStore } from '../stores/config.svelte.js';
  import { themeStore, themes } from '../stores/theme.svelte.js';
  import { 
    X, Save, RefreshCw, Settings as SettingsIcon, Brain, 
    Palette, Monitor, Moon, Sun, Shield, Cpu, Database, 
    Globe, Key, Layout, Check, Info
  } from 'lucide-svelte';
  import IntelligenceTab from './IntelligenceTab.svelte';
  import { cn } from '../utils.js';

  let { onClose, initialTab = 'general' } = $props();
  
  let activeTab = $state(initialTab);

  // AI Settings State
  let provider = $state(configStore.settings.ai.provider);
  let apiKey = $state(configStore.settings.ai.apiKey);
  let baseUrl = $state(configStore.settings.ai.baseUrl);
  let selectedModel = $state(configStore.settings.ai.selectedModel);
  let availableModels = $state([]);
  let isFetchingModels = $state(false);
  let isSaving = $state(false);

  const tabs = [
    { id: 'general', label: 'AI Settings', icon: Brain },
    { id: 'appearance', label: 'Appearance', icon: Palette },
    { id: 'intelligence', label: 'Intelligence', icon: Cpu },
    { id: 'advanced', label: 'Advanced', icon: Database },
  ];

  async function fetchModels() {
    if (!apiKey) return;
    isFetchingModels = true;
    try {
      availableModels = await invoke('fetch_kilo_models', { apiKey, forceRefresh: true });
    } catch (e) {
      console.error('Failed to fetch models:', e);
    } finally {
      isFetchingModels = false;
    }
  }

  async function saveSettings() {
    isSaving = true;
    try {
      configStore.updateAiConfig({
        provider,
        apiKey,
        baseUrl,
        selectedModel,
        models: availableModels.length > 0 ? availableModels.map(m => m.id) : configStore.settings.ai.models
      });
      // Save theme settings as well
      configStore.updateSettings({ 
        theme: themeStore.activeTheme,
        appearance: { mode: themeStore.mode }
      });
      onClose?.();
    } catch (e) {
      console.error('Failed to save settings:', e);
    } finally {
      isSaving = false;
    }
  }
</script>

<div 
  class="fixed inset-0 z-[100] flex items-center justify-center bg-black/60 backdrop-blur-sm animate-in fade-in duration-300"
  onclick={onClose}
>
  <div 
    class="w-[90%] max-w-4xl h-[80vh] bg-background border border-border shadow-2xl rounded-2xl flex overflow-hidden animate-in zoom-in-95 duration-300"
    onclick={(e) => e.stopPropagation()}
  >
    <!-- Sidebar Tabs -->
    <aside class="w-64 border-r border-border bg-muted/10 flex flex-col shrink-0">
      <div class="p-6 border-b border-border/50">
        <h2 class="text-lg font-bold flex items-center gap-2 text-foreground">
          <SettingsIcon size={20} class="text-primary" />
          Settings
        </h2>
      </div>
      
      <nav class="flex-1 p-2 space-y-1">
        {#each tabs as tab}
          <button
            class={cn(
              "w-full flex items-center gap-3 px-4 py-2.5 rounded-xl text-sm font-medium transition-all duration-200",
              activeTab === tab.id 
                ? "bg-primary text-primary-foreground shadow-lg shadow-primary/20" 
                : "text-muted-foreground hover:bg-muted hover:text-foreground"
            )}
            onclick={() => activeTab = tab.id}
          >
            <tab.icon size={18} />
            {tab.label}
          </button>
        {/each}
      </nav>

      <div class="p-4 border-t border-border/50 text-[10px] text-muted-foreground uppercase tracking-widest font-bold text-center">
        Karsa IDE v0.1.0-alpha
      </div>
    </aside>

    <!-- Main Content Area -->
    <main class="flex-1 flex flex-col min-w-0 bg-background relative">
      <!-- Header -->
      <div class="h-14 border-b border-border flex items-center justify-between px-6 bg-background/50 backdrop-blur shrink-0">
        <span class="text-sm font-semibold capitalize text-foreground">{activeTab} Configuration</span>
        <button 
          onclick={onClose}
          class="p-1.5 hover:bg-muted rounded-lg text-muted-foreground hover:text-foreground transition-colors"
        >
          <X size={18} />
        </button>
      </div>

      <!-- Scrollable Content -->
      <div class="flex-1 overflow-y-auto p-8 space-y-8 scrollbar-thin">
        
        {#if activeTab === 'general'}
          <div class="space-y-6 animate-in slide-in-from-right-4 duration-300">
            <div class="grid gap-4">
              <label class="space-y-2">
                <span class="text-xs font-bold text-muted-foreground uppercase tracking-wider">AI Provider</span>
                <select 
                  bind:value={provider}
                  class="w-full bg-muted/30 border border-border rounded-xl px-3 py-2 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none transition-all text-foreground"
                >
                  <option value="kilo">Kilo AI (Recommended)</option>
                  <option value="openai">OpenAI</option>
                  <option value="anthropic">Anthropic</option>
                  <option value="ollama">Ollama (Local)</option>
                </select>
              </label>

              <label class="space-y-2">
                <span class="text-xs font-bold text-muted-foreground uppercase tracking-wider">API Key</span>
                <div class="relative">
                  <Key size={14} class="absolute left-3 top-3 text-muted-foreground/50" />
                  <input 
                    type="password" 
                    bind:value={apiKey}
                    placeholder="Enter your API key..."
                    class="w-full bg-muted/30 border border-border rounded-xl pl-9 pr-3 py-2 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none transition-all text-foreground"
                  />
                </div>
              </label>

              <label class="space-y-2">
                <span class="text-xs font-bold text-muted-foreground uppercase tracking-wider">Base URL</span>
                <div class="relative">
                  <Globe size={14} class="absolute left-3 top-3 text-muted-foreground/50" />
                  <input 
                    type="text" 
                    bind:value={baseUrl}
                    placeholder="https://api.kilo.ai/v1"
                    class="w-full bg-muted/30 border border-border rounded-xl pl-9 pr-3 py-2 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none transition-all text-foreground"
                  />
                </div>
              </label>

              <label class="space-y-2">
                <span class="text-xs font-bold text-muted-foreground uppercase tracking-wider">Preferred Model</span>
                <div class="flex gap-2">
                  <select 
                    bind:value={selectedModel}
                    class="flex-1 bg-muted/30 border border-border rounded-xl px-3 py-2 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none transition-all text-foreground"
                  >
                    {#each availableModels.length > 0 ? availableModels : configStore.settings.ai.models as model}
                      <option value={typeof model === 'string' ? model : model.id}>
                        {typeof model === 'string' ? model : model.name || model.id}
                      </option>
                    {/each}
                  </select>
                  <button 
                    onclick={fetchModels}
                    disabled={isFetchingModels || !apiKey}
                    class="p-2.5 bg-muted border border-border rounded-xl hover:bg-muted-foreground/10 transition-colors disabled:opacity-50 text-foreground"
                  >
                    <RefreshCw size={18} class={isFetchingModels ? 'animate-spin' : ''} />
                  </button>
                </div>
              </label>
            </div>
          </div>

        {:else if activeTab === 'appearance'}
          <div class="space-y-8 animate-in slide-in-from-right-4 duration-300">
            <!-- Theme Mode -->
            <div class="space-y-4">
              <h3 class="text-sm font-bold text-foreground">Theme Mode</h3>
              <div class="grid grid-cols-3 gap-3">
                {#each [
                  { id: 'light', label: 'Light', icon: Sun },
                  { id: 'dark', label: 'Dark', icon: Moon },
                  { id: 'system', label: 'System', icon: Monitor }
                ] as mode}
                  <button
                    onclick={() => themeStore.setMode(mode.id)}
                    class={cn(
                      "flex flex-col items-center gap-2 p-4 rounded-2xl border-2 transition-all",
                      themeStore.mode === mode.id 
                        ? "border-primary bg-primary/5 text-primary" 
                        : "border-border/50 bg-muted/20 text-muted-foreground hover:border-border hover:bg-muted/40"
                    )}
                  >
                    <mode.icon size={20} />
                    <span class="text-xs font-medium">{mode.label}</span>
                  </button>
                {/each}
              </div>
            </div>

            <!-- Theme Presets -->
            <div class="space-y-4">
              <h3 class="text-sm font-bold text-foreground">Color Palette</h3>
              <div class="grid grid-cols-2 gap-3">
                {#each Object.entries(themes) as [id, theme]}
                  <button
                    onclick={() => themeStore.setTheme(id)}
                    class={cn(
                      "flex items-center justify-between p-4 rounded-2xl border-2 transition-all",
                      themeStore.activeTheme === id 
                        ? "border-primary bg-primary/5" 
                        : "border-border/50 bg-muted/20 hover:border-border hover:bg-muted/40"
                    )}
                  >
                    <div class="flex items-center gap-3">
                       <div 
                         class="w-4 h-4 rounded-full border border-white/20"
                         style="background-color: hsl({theme.colors['--primary']})"
                       ></div>
                       <span class="text-xs font-medium text-foreground">{theme.name}</span>
                    </div>
                    {#if themeStore.activeTheme === id}
                      <Check size={14} class="text-primary" />
                    {/if}
                  </button>
                {/each}
              </div>
            </div>

            <div class="p-4 bg-primary/5 border border-primary/20 rounded-2xl flex gap-3">
               <Info size={18} class="text-primary shrink-0" />
               <p class="text-[11px] text-muted-foreground leading-relaxed">
                 Some visual changes might require a window reload to take full effect on the editor's core.
               </p>
            </div>
          </div>

        {:else if activeTab === 'intelligence'}
          <div class="animate-in slide-in-from-right-4 duration-300 text-foreground">
            <IntelligenceTab />
          </div>

        {:else if activeTab === 'advanced'}
           <div class="flex flex-col items-center justify-center py-12 text-center space-y-4 animate-in fade-in duration-500">
              <div class="w-16 h-16 rounded-full bg-muted flex items-center justify-center text-muted-foreground">
                 <Shield size={32} />
              </div>
              <div>
                <h4 class="text-sm font-bold text-foreground">Advanced Configuration</h4>
                <p class="text-xs text-muted-foreground mt-1">These settings are reserved for power users.</p>
              </div>
              <button class="px-4 py-2 bg-muted text-foreground rounded-xl text-xs font-medium hover:bg-muted/80 transition-colors">
                 Open Config File
              </button>
           </div>
        {/if}
      </div>

      <!-- Footer Actions -->
      <div class="p-6 border-t border-border bg-muted/5 flex items-center justify-between shrink-0">
        <div class="text-[10px] text-muted-foreground flex items-center gap-1.5 font-medium">
           <Shield size={12} /> Privacy: All keys are stored locally on your device.
        </div>
        <div class="flex gap-3">
          <button 
            onclick={onClose}
            class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
          >
            Cancel
          </button>
          <button 
            onclick={saveSettings}
            disabled={isSaving}
            class="px-6 py-2 bg-primary text-primary-foreground rounded-xl text-sm font-bold shadow-lg shadow-primary/20 hover:scale-105 active:scale-95 transition-all flex items-center gap-2"
          >
            {#if isSaving} <RefreshCw size={16} class="animate-spin" /> {:else} <Save size={16} /> {/if}
            {isSaving ? 'Saving...' : 'Save Changes'}
          </button>
        </div>
      </div>
    </main>
  </div>
</div>
