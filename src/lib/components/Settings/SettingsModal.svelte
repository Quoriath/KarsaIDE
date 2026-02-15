<script>
  import { themeStore, themes } from '../../stores/theme.svelte.js';
  import { configStore } from '../../stores/config.svelte.js';
  import { 
    X, Check, Monitor, Type, Layout, Cpu, TerminalSquare, Keyboard, Save 
  } from 'lucide-svelte';
  import { cn } from '../../utils.js';

  let { onClose } = $props();
  let activeTab = $state('general');

  const tabs = [
    { id: 'general', label: 'General', icon: Monitor },
    { id: 'editor', label: 'Editor', icon: Type },
    { id: 'ai', label: 'AI & Agent', icon: Cpu },
    { id: 'terminal', label: 'Terminal', icon: TerminalSquare },
    { id: 'shortcuts', label: 'Shortcuts', icon: Keyboard },
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
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm animate-in fade-in duration-200" onclick={onClose}>
  <div 
    class="w-[900px] h-[650px] bg-background border border-border rounded-xl shadow-2xl flex flex-col overflow-hidden animate-in zoom-in-95 duration-200"
    onclick={(e) => e.stopPropagation()}
  >
    <!-- Header -->
    <div class="h-16 px-6 border-b border-border flex items-center justify-between bg-muted/10 shrink-0">
      <div>
        <h2 class="text-lg font-semibold">Settings</h2>
        <p class="text-xs text-muted-foreground">Manage your preferences and configurations</p>
      </div>
      <button onclick={onClose} class="p-2 hover:bg-muted rounded-lg transition-colors text-muted-foreground hover:text-foreground">
        <X size={20} />
      </button>
    </div>

    <div class="flex-1 flex overflow-hidden">
      <!-- Sidebar Tabs -->
      <div class="w-56 border-r border-border bg-muted/5 p-4 space-y-1 overflow-y-auto">
        {#each tabs as tab}
          <button
            class={cn(
              "w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors text-left",
              activeTab === tab.id 
                ? "bg-primary/10 text-primary" 
                : "text-muted-foreground hover:bg-muted/50 hover:text-foreground"
            )}
            onclick={() => activeTab = tab.id}
          >
            <tab.icon size={18} />
            {tab.label}
          </button>
        {/each}
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-8 scrollbar-thin scrollbar-thumb-muted">
        
        {#if activeTab === 'general'}
          <div class="space-y-8 max-w-2xl">
            <!-- Theme Section -->
            <section>
              <h3 class="text-base font-medium mb-4 pb-2 border-b border-border">Appearance</h3>
              <div class="space-y-4">
                <label class="text-sm text-muted-foreground">Color Theme</label>
                <div class="grid grid-cols-2 md:grid-cols-3 gap-4">
                  {#each Object.entries(themes) as [key, theme]}
                    <button
                      class={cn(
                        "group relative flex flex-col items-start border rounded-xl overflow-hidden transition-all hover:shadow-md",
                        themeStore.activeTheme === key 
                          ? "border-primary ring-2 ring-primary/20" 
                          : "border-border hover:border-primary/50"
                      )}
                      onclick={() => selectTheme(key)}
                    >
                      <div class="w-full h-20 flex">
                         <div class="w-1/3 h-full" style="background-color: {getThemeColor(theme, '--background')}"></div>
                         <div class="w-1/3 h-full" style="background-color: {getThemeColor(theme, '--card')}"></div>
                         <div class="w-1/3 h-full" style="background-color: {getThemeColor(theme, '--primary')}"></div>
                      </div>
                      <div class="w-full p-2.5 bg-card flex items-center justify-between">
                        <span class="text-xs font-medium">{theme.name}</span>
                        {#if themeStore.activeTheme === key}
                          <div class="bg-primary text-primary-foreground rounded-full p-0.5">
                            <Check size={10} />
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
               <h3 class="text-base font-medium mb-4 pb-2 border-b border-border">Files</h3>
               <div class="flex items-center justify-between p-3 border border-border rounded-lg bg-card/30">
                  <div class="space-y-0.5">
                     <div class="text-sm font-medium">Auto Save</div>
                     <div class="text-xs text-muted-foreground">Save changes automatically after delay</div>
                  </div>
                  <input type="checkbox" checked class="accent-primary w-4 h-4" />
               </div>
            </section>
          </div>

        {:else if activeTab === 'editor'}
          <div class="space-y-8 max-w-2xl">
             <section>
                <h3 class="text-base font-medium mb-4 pb-2 border-b border-border">Typography</h3>
                <div class="space-y-4">
                   <div class="grid gap-2">
                      <label class="text-sm font-medium">Font Family</label>
                      <select 
                        class="w-full px-3 py-2 bg-background border border-border rounded-lg text-sm focus:border-primary outline-none focus:ring-1 focus:ring-primary/20"
                        bind:value={configStore.settings.editor.fontFamily}
                        onchange={() => configStore.save()}
                      >
                         <option value="'JetBrains Mono', monospace">JetBrains Mono</option>
                         <option value="'Fira Code', monospace">Fira Code</option>
                         <option value="'Consolas', monospace">Consolas</option>
                         <option value="'Source Code Pro', monospace">Source Code Pro</option>
                      </select>
                   </div>
                   
                   <div class="grid gap-2">
                      <label class="text-sm font-medium">Font Size: {configStore.settings.editor.fontSize}px</label>
                      <div class="flex items-center gap-4">
                         <input 
                           type="range" min="10" max="24" step="1"
                           bind:value={configStore.settings.editor.fontSize}
                           onchange={() => configStore.save()}
                           class="flex-1 accent-primary h-2 bg-muted rounded-lg appearance-none cursor-pointer" 
                         />
                      </div>
                   </div>
                </div>
             </section>

             <section>
                <h3 class="text-base font-medium mb-4 pb-2 border-b border-border">Display</h3>
                <div class="space-y-3">
                   <div class="flex items-center justify-between py-2 border-b border-border/50">
                      <span class="text-sm">Minimap</span>
                      <input 
                        type="checkbox" 
                        bind:checked={configStore.settings.editor.minimap}
                        onchange={() => configStore.save()}
                        class="accent-primary w-4 h-4" 
                      />
                   </div>
                   
                   <div class="flex items-center justify-between py-2 border-b border-border/50">
                      <span class="text-sm">Word Wrap</span>
                      <select 
                        class="bg-transparent text-sm border-none outline-none text-right"
                        bind:value={configStore.settings.editor.wordWrap}
                        onchange={() => configStore.save()}
                      >
                         <option value="off">Off</option>
                         <option value="on">On</option>
                         <option value="wordWrapColumn">Word Wrap Column</option>
                      </select>
                   </div>

                   <div class="flex items-center justify-between py-2 border-b border-border/50">
                      <span class="text-sm">Line Numbers</span>
                      <select class="bg-transparent text-sm border-none outline-none text-right">
                         <option value="on">On</option>
                         <option value="off">Off</option>
                         <option value="relative">Relative</option>
                      </select>
                   </div>
                </div>
             </section>
          </div>

        {:else if activeTab === 'ai'}
           <div class="space-y-8 max-w-2xl">
              <section>
                 <h3 class="text-base font-medium mb-4 pb-2 border-b border-border">Provider Configuration</h3>
                 
                 <div class="space-y-4">
                    <div class="grid gap-2">
                       <label class="text-sm font-medium">AI Provider</label>
                       <div class="grid grid-cols-3 gap-3">
                          {#each ['kilo', 'openai', 'ollama'] as p}
                             <button 
                               class={cn("px-3 py-2 border rounded-lg text-sm capitalize transition-all", 
                                 configStore.settings.ai.provider === p ? "bg-primary/10 border-primary text-primary" : "bg-card border-border hover:bg-muted")}
                               onclick={() => updateConfig('ai', 'provider', p)}
                             >
                               {p}
                             </button>
                          {/each}
                       </div>
                    </div>

                    <div class="grid gap-2">
                       <label class="text-sm font-medium">API Endpoint (Base URL)</label>
                       <input 
                         type="text" 
                         bind:value={configStore.settings.ai.baseUrl}
                         onchange={() => configStore.save()}
                         class="w-full bg-background border border-border rounded-lg px-3 py-2 text-sm focus:border-primary outline-none font-mono"
                       />
                    </div>

                    <div class="grid gap-2">
                       <label class="text-sm font-medium">API Key</label>
                       <div class="relative">
                          <input 
                            type="password" 
                            bind:value={configStore.settings.ai.apiKey}
                            onchange={() => configStore.save()}
                            placeholder="sk-..."
                            class="w-full bg-background border border-border rounded-lg px-3 py-2 text-sm focus:border-primary outline-none font-mono pl-9"
                          />
                          <div class="absolute left-3 top-2.5 text-muted-foreground">
                             <Save size={14} />
                          </div>
                       </div>
                       <p class="text-[10px] text-muted-foreground">Stored locally in encrypted config.</p>
                    </div>
                 </div>
              </section>

              <section>
                 <h3 class="text-base font-medium mb-4 pb-2 border-b border-border">Model Defaults</h3>
                 <div class="grid gap-2">
                    <label class="text-sm font-medium">Custom Models (CSV)</label>
                    <textarea 
                      class="w-full bg-background border border-border rounded-lg px-3 py-2 text-sm focus:border-primary outline-none font-mono min-h-[80px]"
                      placeholder="model-id-1, model-id-2"
                      value={Array.isArray(configStore.settings.ai.models) ? configStore.settings.ai.models.join(', ') : configStore.settings.ai.models}
                      oninput={(e) => updateConfig('ai', 'models', e.target.value.split(',').map(s => s.trim()))}
                    ></textarea>
                 </div>
              </section>
           </div>

        {:else if activeTab === 'terminal'}
           <div class="space-y-8 max-w-2xl">
              <div class="p-6 text-center border border-dashed border-border rounded-xl bg-muted/5">
                 <TerminalSquare size={48} class="mx-auto text-muted-foreground mb-4" />
                 <h3 class="text-lg font-medium">Terminal Settings</h3>
                 <p class="text-muted-foreground mt-2 text-sm">Shell integration and font customization coming in v0.2.0</p>
              </div>
           </div>

        {:else if activeTab === 'shortcuts'}
           <div class="space-y-4 max-w-2xl">
              <h3 class="text-base font-medium mb-4 pb-2 border-b border-border">Keybindings</h3>
              
              {#each [
                { cmd: 'Command Palette', key: 'Ctrl+Shift+P' },
                { cmd: 'Quick Open', key: 'Ctrl+P' },
                { cmd: 'Toggle Sidebar', key: 'Ctrl+B' },
                { cmd: 'Toggle Terminal', key: 'Ctrl+`' },
                { cmd: 'AI Chat', key: 'Ctrl+L' }
              ] as shortcut}
                <div class="flex items-center justify-between py-3 border-b border-border/50">
                   <span class="text-sm">{shortcut.cmd}</span>
                   <kbd class="px-2 py-1 bg-muted border border-border rounded text-xs font-mono text-muted-foreground">
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
