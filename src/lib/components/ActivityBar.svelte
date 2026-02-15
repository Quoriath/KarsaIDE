<script>
  import { Files, Search, Settings, Sparkles, LayoutGrid, Cpu, Code2, PanelBottom } from 'lucide-svelte';
  import { cn } from '../utils.js';
  import { configStore } from '../../stores/config.svelte.js';
  
  let { activeView = $bindable('explorer'), activeMode = $bindable('editor'), onChatToggle, onOpenSettings, onTerminalToggle } = $props();
  
  const views = [
    { id: 'dashboard', icon: LayoutGrid, label: 'Dashboard' },
    { id: 'explorer', icon: Files, label: 'Explorer' },
    { id: 'search', icon: Search, label: 'Search' },
    { id: 'settings', icon: Settings, label: 'Settings', action: () => onOpenSettings && onOpenSettings() },
  ];
</script>

<div class="w-14 bg-muted/30 flex flex-col items-center py-4 border-r border-border h-full z-20 backdrop-blur-sm">
  
  <!-- Mode Switcher (Top) -->
  <div class="flex flex-col gap-3 mb-6 w-full px-2">
    <button
      class={cn(
        "w-10 h-10 flex items-center justify-center rounded-xl transition-all relative group shadow-sm",
        activeMode === 'editor' 
          ? "bg-primary text-primary-foreground shadow-primary/20" 
          : "bg-muted text-muted-foreground hover:bg-muted/80 hover:text-foreground"
      )}
      onclick={() => activeMode = 'editor'}
      title="Editor Mode"
    >
      <Code2 size={20} />
    </button>
    
    <button
      class={cn(
        "w-10 h-10 flex items-center justify-center rounded-xl transition-all relative group shadow-sm",
        activeMode === 'agent' 
          ? "bg-purple-600 text-white shadow-purple-500/20" 
          : "bg-muted text-muted-foreground hover:bg-muted/80 hover:text-foreground"
      )}
      onclick={() => activeMode = 'agent'}
      title="Agent Mode"
    >
      <Cpu size={20} />
      {#if activeMode === 'agent'}
        <span class="absolute -top-1 -right-1 flex h-3 w-3">
          <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-purple-400 opacity-75"></span>
          <span class="relative inline-flex rounded-full h-3 w-3 bg-purple-500"></span>
        </span>
      {/if}
    </button>
  </div>
  
  <div class="w-8 h-[1px] bg-border mb-4"></div>

  <!-- Editor Views -->
  {#if activeMode === 'editor'}
    <div class="flex flex-col gap-2 w-full px-2 animate-in fade-in slide-in-from-left-4 duration-300">
      {#each views as view}
        <button
          class={cn(
            "w-10 h-10 flex items-center justify-center rounded-lg transition-all relative group",
            activeView === view.id 
              ? "text-foreground bg-background border border-border/50 shadow-sm" 
              : "text-muted-foreground/60 hover:text-foreground hover:bg-muted/50"
          )}
          onclick={() => {
             if (view.action) view.action();
             else activeView = view.id;
          }}
          title={view.label}
        >
          <view.icon size={20} strokeWidth={activeView === view.id ? 2.5 : 2} />
          {#if activeView === view.id}
            <div class="absolute left-0 top-3 bottom-3 w-0.5 bg-primary rounded-r-full"></div>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
  
  <div class="flex-1"></div>
  
  {#if activeMode === 'editor'}
    <!-- Toggle Bottom Panel -->
    <button
      class={cn(
        "w-10 h-10 flex items-center justify-center rounded-lg mb-2 transition-all relative group",
        configStore.settings.layout.bottomPanelVisible ? "text-foreground bg-muted/50" : "text-muted-foreground hover:text-foreground"
      )}
      onclick={onTerminalToggle}
      title="Toggle Terminal Panel"
    >
      <PanelBottom size={20} />
    </button>

    <!-- Toggle Chat Panel -->
    <button
      class={cn(
        "w-10 h-10 flex items-center justify-center rounded-lg mb-2 transition-all relative group",
        configStore.settings.layout.chatVisible ? "text-foreground bg-muted/50" : "text-muted-foreground hover:text-foreground"
      )}
      onclick={onChatToggle}
      title="Toggle AI Chat Panel"
    >
      <Sparkles size={20} />
    </button>
  {/if}
  
  <!-- User Profile / Account (Placeholder) -->
  <div class="w-8 h-8 rounded-full bg-gradient-to-tr from-primary to-purple-500 mb-4 mt-2 shadow-inner cursor-pointer hover:ring-2 ring-primary/50 transition-all"></div>
</div>
