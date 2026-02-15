<script>
  import { themeStore, themes } from '../../stores/theme.svelte.js';
  import { configStore } from '../../stores/config.svelte.js';
  import { X, Check, Monitor, Type, Layout, Cpu } from 'lucide-svelte';
  import { cn } from '../../utils.js';

  let { onClose } = $props();
  let activeTab = $state('appearance');

  const tabs = [
    { id: 'appearance', label: 'Appearance', icon: Monitor },
    { id: 'editor', label: 'Editor', icon: Type },
    { id: 'ai', label: 'AI Agent', icon: Cpu },
  ];

  function selectTheme(key) {
    themeStore.setTheme(key);
    configStore.setTheme(key);
  }

  // Helper to extract a representative color from theme string (HSL)
  function getThemeColor(theme, varName) {
    // This is a naive extraction for UI preview purposes
    // Ideally we'd compute the actual color
    return `hsl(${theme.colors[varName]})`;
  }
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm animate-in fade-in duration-200" onclick={onClose}>
  <div 
    class="w-[800px] h-[600px] bg-background border border-border rounded-xl shadow-2xl flex flex-col overflow-hidden animate-in zoom-in-95 duration-200"
    onclick={(e) => e.stopPropagation()}
  >
    <!-- Header -->
    <div class="h-14 px-6 border-b border-border flex items-center justify-between bg-muted/20">
      <h2 class="text-lg font-semibold">Settings</h2>
      <button onclick={onClose} class="p-2 hover:bg-muted rounded-lg transition-colors">
        <X size={20} />
      </button>
    </div>

    <div class="flex-1 flex overflow-hidden">
      <!-- Sidebar Tabs -->
      <div class="w-48 border-r border-border bg-muted/10 p-4 space-y-1">
        {#each tabs as tab}
          <button
            class={cn(
              "w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors",
              activeTab === tab.id 
                ? "bg-primary/10 text-primary" 
                : "text-muted-foreground hover:bg-muted hover:text-foreground"
            )}
            onclick={() => activeTab = tab.id}
          >
            <tab.icon size={18} />
            {tab.label}
          </button>
        {/each}
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-8">
        
        {#if activeTab === 'appearance'}
          <div class="space-y-8">
            <div>
              <h3 class="text-base font-medium mb-4">Color Theme</h3>
              <div class="grid grid-cols-2 md:grid-cols-3 gap-4">
                {#each Object.entries(themes) as [key, theme]}
                  <button
                    class={cn(
                      "group relative flex flex-col items-start border rounded-xl overflow-hidden transition-all hover:scale-[1.02]",
                      themeStore.activeTheme === key 
                        ? "border-primary ring-2 ring-primary/20" 
                        : "border-border hover:border-primary/50"
                    )}
                    onclick={() => selectTheme(key)}
                  >
                    <!-- Preview Swatch -->
                    <div class="w-full h-24 flex">
                       <div class="w-1/3 h-full" style="background-color: {getThemeColor(theme, '--background')}"></div>
                       <div class="w-1/3 h-full" style="background-color: {getThemeColor(theme, '--card')}"></div>
                       <div class="w-1/3 h-full" style="background-color: {getThemeColor(theme, '--primary')}"></div>
                    </div>
                    
                    <div class="w-full p-3 bg-card flex items-center justify-between">
                      <span class="text-sm font-medium">{theme.name}</span>
                      {#if themeStore.activeTheme === key}
                        <div class="bg-primary text-primary-foreground rounded-full p-0.5">
                          <Check size={12} />
                        </div>
                      {/if}
                    </div>
                  </button>
                {/each}
              </div>
            </div>

            <div>
               <h3 class="text-base font-medium mb-4">UI Density</h3>
               <div class="flex items-center gap-4">
                  <button class="px-4 py-2 border border-border rounded-lg text-sm hover:border-primary/50 transition-colors">Compact</button>
                  <button class="px-4 py-2 border border-primary bg-primary/5 text-primary rounded-lg text-sm font-medium">Comfortable</button>
                  <button class="px-4 py-2 border border-border rounded-lg text-sm hover:border-primary/50 transition-colors">Spacious</button>
               </div>
            </div>
          </div>

        {:else if activeTab === 'editor'}
          <div class="space-y-6">
             <div class="space-y-2">
                <label class="text-sm font-medium">Font Family</label>
                <select class="w-full px-3 py-2 bg-background border border-border rounded-lg text-sm focus:border-primary outline-none">
                   <option>'JetBrains Mono', monospace</option>
                   <option>'Fira Code', monospace</option>
                   <option>'Consolas', monospace</option>
                </select>
             </div>
             
             <div class="space-y-2">
                <label class="text-sm font-medium">Font Size</label>
                <div class="flex items-center gap-4">
                   <input type="range" min="10" max="24" value="14" class="flex-1 accent-primary" />
                   <span class="text-sm font-mono bg-muted px-2 py-1 rounded">14px</span>
                </div>
             </div>

             <div class="flex items-center justify-between py-2">
                <span class="text-sm">Minimap</span>
                <input type="checkbox" checked class="accent-primary w-4 h-4" />
             </div>
             
             <div class="flex items-center justify-between py-2">
                <span class="text-sm">Word Wrap</span>
                <input type="checkbox" checked class="accent-primary w-4 h-4" />
             </div>
          </div>

        {:else if activeTab === 'ai'}
           <div class="space-y-6">
              <div class="p-4 bg-muted/30 rounded-lg border border-border">
                 <h4 class="font-medium mb-2 flex items-center gap-2">
                    <Cpu size={16} class="text-primary" />
                    Provider Configuration
                 </h4>
                 <p class="text-xs text-muted-foreground mb-4">
                    Managed via ConfigStore. Backend integration pending.
                 </p>
                 <div class="grid grid-cols-2 gap-4">
                    <div class="space-y-1">
                       <label class="text-xs text-muted-foreground">Provider</label>
                       <div class="px-3 py-2 bg-background border border-border rounded text-sm">Kilo AI</div>
                    </div>
                    <div class="space-y-1">
                       <label class="text-xs text-muted-foreground">Model</label>
                       <div class="px-3 py-2 bg-background border border-border rounded text-sm">minimax-m2.5</div>
                    </div>
                 </div>
              </div>
           </div>
        {/if}
      </div>
    </div>
  </div>
</div>
