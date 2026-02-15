<script>
  import { themeStore, themes } from '../../stores/theme.svelte.js';
  import { configStore } from '../../stores/config.svelte.js';
  import { 
    X, Check, Monitor, Type, Layout, Cpu, TerminalSquare, Keyboard, Save, ChevronDown 
  } from 'lucide-svelte';
  import { cn } from '../../utils.js';
  import Select from '../ui/Select.svelte';

  let { onClose } = $props();
  let activeTab = $state('general');

  const tabs = [
    { id: 'general', label: 'General', icon: Monitor },
    { id: 'editor', label: 'Editor', icon: Type },
    { id: 'ai', label: 'AI & Agent', icon: Cpu },
    { id: 'terminal', label: 'Terminal', icon: TerminalSquare },
    { id: 'shortcuts', label: 'Shortcuts', icon: Keyboard },
  ];

  const fontOptions = [
    { value: "'JetBrains Mono', monospace", label: 'JetBrains Mono' },
    { value: "'Fira Code', monospace", label: 'Fira Code' },
    { value: "'Consolas', monospace", label: 'Consolas' },
    { value: "'Source Code Pro', monospace", label: 'Source Code Pro' },
  ];

  const wrapOptions = [
    { value: 'off', label: 'Off' },
    { value: 'on', label: 'On' },
    { value: 'wordWrapColumn', label: 'Column' },
  ];

  function selectTheme(key) {
    themeStore.setTheme(key);
    configStore.setTheme(key);
  }

  function getThemeColor(theme, varName) {
    return `hsl(${theme.colors[varName]})`;
  }

  function updateConfig(section, key, value) {
    if (section) {
        configStore.settings[section] = { ...configStore.settings[section], [key]: value };
    } else {
        configStore.settings[key] = value;
    }
    configStore.save();
  }
  
  function handleKeydown(e) {
    if (e.key === 'Escape') onClose();
  }
</script>

<div 
  class="fixed inset-0 z-[999] flex items-center justify-center bg-black/80 backdrop-blur-sm animate-in fade-in duration-200" 
  onclick={onClose}
  role="dialog"
  aria-modal="true"
  tabindex="-1"
  onkeydown={handleKeydown}
>
  <!-- Modal Container -->
  <!-- bg-zinc-950 forces a solid dark background as fallback/override for transparency issues -->
  <div 
    class="w-[900px] h-[650px] bg-background bg-zinc-950 border border-border/40 rounded-xl shadow-2xl flex flex-col overflow-hidden animate-in zoom-in-95 duration-200 relative isolate"
    onclick={(e) => e.stopPropagation()}
    role="document"
    tabindex="0"
  >
    <!-- Header -->
    <div class="h-16 px-6 border-b border-border/40 flex items-center justify-between bg-muted/5 shrink-0 select-none">
      <div class="flex items-center gap-3">
        <div class="p-2 bg-primary/10 rounded-lg text-primary ring-1 ring-primary/20">
           <Settings size={20} />
        </div>
        <div>
          <h2 class="text-base font-semibold tracking-tight">Settings</h2>
          <p class="text-xs text-muted-foreground">Manage your preferences</p>
        </div>
      </div>
      <button onclick={onClose} class="p-2 hover:bg-muted rounded-lg transition-colors text-muted-foreground hover:text-foreground">
        <X size={20} />
      </button>
    </div>

    <div class="flex-1 flex overflow-hidden bg-background/50">
      <!-- Sidebar Tabs -->
      <div class="w-60 border-r border-border/40 bg-muted/5 p-3 space-y-1 overflow-y-auto">
        {#each tabs as tab}
          <button
            class={cn(
              "w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all text-left",
              activeTab === tab.id 
                ? "bg-background shadow-sm text-foreground ring-1 ring-border/50" 
                : "text-muted-foreground hover:bg-muted/50 hover:text-foreground"
            )}
            onclick={() => activeTab = tab.id}
          >
            <tab.icon size={16} class={activeTab === tab.id ? "text-primary" : "opacity-70"} />
            {tab.label}
          </button>
        {/each}
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-8 scrollbar-thin scrollbar-thumb-muted">
        
        {#if activeTab === 'general'}
          <div class="space-y-8 max-w-2xl animate-in fade-in slide-in-from-bottom-2 duration-300">
            <!-- Theme Section -->
            <section>
              <h3 class="text-sm font-medium mb-4 text-foreground/90">Appearance</h3>
              <div class="space-y-4">
                <span class="text-xs text-muted-foreground block mb-2 font-medium uppercase tracking-wider">Color Theme</span>
                <div class="grid grid-cols-2 md:grid-cols-3 gap-4">
                  {#each Object.entries(themes) as [key, theme]}
                    <button
                      class={cn(
                        "group relative flex flex-col items-start border rounded-xl overflow-hidden transition-all hover:shadow-lg hover:-translate-y-0.5",
                        themeStore.activeTheme === key 
                          ? "border-primary ring-2 ring-primary ring-offset-2 ring-offset-background" 
                          : "border-border/60 hover:border-primary/50"
                      )}
                      onclick={() => selectTheme(key)}
                    >
                      <div class="w-full h-24 flex">
                         <div class="w-1/3 h-full" style="background-color: {getThemeColor(theme, '--background')}"></div>
                         <div class="w-1/3 h-full" style="background-color: {getThemeColor(theme, '--card')}"></div>
                         <div class="w-1/3 h-full" style="background-color: {getThemeColor(theme, '--primary')}"></div>
                      </div>
                      <div class="w-full p-3 bg-card flex items-center justify-between border-t border-border/50">
                        <span class="text-xs font-medium">{theme.name}</span>
                        {#if themeStore.activeTheme === key}
                          <div class="bg-primary text-primary-foreground rounded-full p-0.5 shadow-sm">
                            <Check size={12} />
                          </div>
                        {/if}
                      </div>
                    </button>
                  {/each}
                </div>
              </div>
            </section>

            <!-- File Auto Save -->
            <section>
               <h3 class="text-sm font-medium mb-4 text-foreground/90 pt-6 border-t border-border/40">Files</h3>
               <div class="flex items-center justify-between p-4 border border-border/60 rounded-xl bg-card/20 hover:bg-card/40 transition-colors">
                  <div class="space-y-0.5">
                     <label for="auto-save" class="text-sm font-medium cursor-pointer">Auto Save</label>
                     <div class="text-xs text-muted-foreground">Save changes automatically after delay</div>
                  </div>
                  <input id="auto-save" type="checkbox" checked class="accent-primary w-4 h-4 cursor-pointer" />
               </div>
            </section>
          </div>

        {:else if activeTab === 'editor'}
          <div class="space-y-8 max-w-2xl animate-in fade-in slide-in-from-bottom-2 duration-300">
             <section>
                <h3 class="text-sm font-medium mb-4 text-foreground/90">Typography</h3>
                <div class="space-y-5">
                   <div class="grid gap-2">
                      <label for="font-family" class="text-sm text-muted-foreground">Font Family</label>
                      <Select 
                        value={configStore.settings.editor.fontFamily} 
                        options={fontOptions}
                        className="w-full"
                        onchange={() => configStore.save()}
                      />
                   </div>
                   
                   <div class="grid gap-2">
                      <div class="flex justify-between items-center">
                         <label for="font-size" class="text-sm text-muted-foreground">Font Size</label>
                         <span class="text-xs font-mono bg-muted/30 border border-border px-2 py-0.5 rounded text-foreground">{configStore.settings.editor.fontSize}px</span>
                      </div>
                      <input 
                        id="font-size"
                        type="range" min="10" max="24" step="1"
                        bind:value={configStore.settings.editor.fontSize}
                        onchange={() => configStore.save()}
                        class="w-full h-1.5 bg-muted rounded-full accent-primary appearance-none cursor-pointer" 
                      />
                   </div>
                </div>
             </section>

             <section>
                <h3 class="text-sm font-medium mb-4 text-foreground/90 pt-6 border-t border-border/40">Display</h3>
                <div class="space-y-1">
                   <div class="flex items-center justify-between py-3 border-b border-border/30">
                      <label for="minimap" class="text-sm cursor-pointer">Minimap</label>
                      <input 
                        id="minimap"
                        type="checkbox" 
                        bind:checked={configStore.settings.editor.minimap}
                        onchange={() => configStore.save()}
                        class="accent-primary w-4 h-4 cursor-pointer" 
                      />
                   </div>
                   
                   <div class="flex items-center justify-between py-3 border-b border-border/30">
                      <label for="word-wrap" class="text-sm cursor-pointer">Word Wrap</label>
                      <div class="w-40">
                        <Select 
                          value={configStore.settings.editor.wordWrap}
                          options={wrapOptions}
                          onchange={() => configStore.save()}
                        />
                      </div>
                   </div>
                </div>
             </section>
          </div>

        {:else if activeTab === 'ai'}
           <div class="space-y-8 max-w-2xl animate-in fade-in slide-in-from-bottom-2 duration-300">
              <section>
                 <h3 class="text-sm font-medium mb-4 text-foreground/90">Provider Configuration</h3>
                 
                 <div class="space-y-5">
                    <div class="grid gap-2">
                       <span class="text-sm text-muted-foreground">AI Provider</span>
                       <div class="grid grid-cols-3 gap-3">
                          {#each ['kilo', 'openai', 'ollama'] as p}
                             <button 
                               class={cn("px-3 py-2.5 border rounded-xl text-sm capitalize transition-all font-medium", 
                                 configStore.settings.ai.provider === p 
                                   ? "bg-primary/10 border-primary text-primary ring-1 ring-primary/20" 
                                   : "bg-muted/10 border-border hover:bg-muted hover:border-primary/30 text-muted-foreground hover:text-foreground")}
                               onclick={() => updateConfig('ai', 'provider', p)}
                             >
                               {p}
                             </button>
                          {/each}
                       </div>
                    </div>

                    <div class="grid gap-2">
                       <label for="api-url" class="text-sm text-muted-foreground">API Endpoint</label>
                       <input 
                         id="api-url"
                         type="text" 
                         bind:value={configStore.settings.ai.baseUrl}
                         onchange={() => configStore.save()}
                         class="w-full bg-muted/20 border border-border hover:border-primary/30 rounded-lg px-3 py-2 text-sm focus:border-primary outline-none font-mono text-foreground focus:ring-1 focus:ring-primary/20 transition-all"
                       />
                    </div>

                    <div class="grid gap-2">
                       <label for="api-key" class="text-sm text-muted-foreground">API Key</label>
                       <div class="relative">
                          <input 
                            id="api-key"
                            type="password" 
                            bind:value={configStore.settings.ai.apiKey}
                            onchange={() => configStore.save()}
                            placeholder="sk-..."
                            class="w-full bg-muted/20 border border-border hover:border-primary/30 rounded-lg px-3 py-2 text-sm focus:border-primary outline-none font-mono pl-9 text-foreground focus:ring-1 focus:ring-primary/20 transition-all"
                          />
                          <div class="absolute left-3 top-2.5 text-muted-foreground">
                             <Save size={14} />
                          </div>
                       </div>
                       <p class="text-[10px] text-muted-foreground/70">Stored securely on your device.</p>
                    </div>
                 </div>
              </section>

              <section>
                 <h3 class="text-sm font-medium mb-4 text-foreground/90 pt-6 border-t border-border/40">Model Defaults</h3>
                 <div class="grid gap-2">
                    <label for="custom-models" class="text-sm text-muted-foreground">Custom Models (CSV)</label>
                    <textarea 
                      id="custom-models"
                      class="w-full bg-muted/20 border border-border hover:border-primary/30 rounded-lg px-3 py-2 text-sm focus:border-primary outline-none font-mono min-h-[80px] text-foreground focus:ring-1 focus:ring-primary/20 transition-all"
                      placeholder="model-id-1, model-id-2"
                      value={Array.isArray(configStore.settings.ai.models) ? configStore.settings.ai.models.join(', ') : configStore.settings.ai.models}
                      oninput={(e) => updateConfig('ai', 'models', e.target.value.split(',').map(s => s.trim()))}
                    ></textarea>
                 </div>
              </section>
           </div>

        {:else if activeTab === 'terminal'}
           <div class="space-y-8 max-w-2xl animate-in fade-in slide-in-from-bottom-2 duration-300">
              <div class="p-10 text-center border border-dashed border-border rounded-xl bg-muted/5">
                 <TerminalSquare size={48} class="mx-auto text-muted-foreground/50 mb-4" />
                 <h3 class="text-lg font-medium">Terminal Settings</h3>
                 <p class="text-muted-foreground mt-2 text-sm">Shell integration and font customization coming in v0.2.0</p>
              </div>
           </div>

        {:else if activeTab === 'shortcuts'}
           <div class="space-y-4 max-w-2xl animate-in fade-in slide-in-from-bottom-2 duration-300">
              <h3 class="text-sm font-medium mb-4 text-foreground/90">Keybindings</h3>
              
              {#each [
                { cmd: 'Command Palette', key: 'Ctrl+Shift+P' },
                { cmd: 'Quick Open', key: 'Ctrl+P' },
                { cmd: 'Toggle Sidebar', key: 'Ctrl+B' },
                { cmd: 'Toggle Terminal', key: 'Ctrl+`' },
                { cmd: 'AI Chat', key: 'Ctrl+L' }
              ] as shortcut}
                <div class="flex items-center justify-between py-3 border-b border-border/30 last:border-0 hover:bg-muted/30 px-2 -mx-2 rounded transition-colors">
                   <span class="text-sm text-foreground/80">{shortcut.cmd}</span>
                   <kbd class="px-2 py-1 bg-background border border-border/60 rounded text-xs font-mono text-muted-foreground font-medium shadow-sm">
                      {shortcut.key}
                   </kbd>
                </div>
              {/each}
           </div>
        {/if}
      </div>
    </div>
  </div>
</div>
