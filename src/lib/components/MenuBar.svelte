<script>
  import { fsStore } from '../stores/fileSystem.svelte.js';
  import { themeStore } from '../stores/theme.svelte.js';
  import { cn } from '../utils.js';
  import { ChevronDown, Check, FileText, Settings, ExternalLink } from 'lucide-svelte';

  let { onOpenSettings } = $props();

  let activeMenu = $state(null);
  let menuRef;

  const menus = [
    { label: 'File', items: [
      { label: 'New File', shortcut: 'Ctrl+N', action: () => alert('New File') },
      { label: 'Open File...', shortcut: 'Ctrl+O', action: () => fsStore.openFile() },
      { label: 'Open Folder...', shortcut: 'Ctrl+K O', action: () => fsStore.setProjectDir() },
      { separator: true },
      { label: 'Settings', icon: Settings, action: () => onOpenSettings && onOpenSettings() },
    ]},
    { label: 'View', items: [
      { label: 'Sidebar', shortcut: 'Ctrl+B', action: () => alert('Sidebar') },
      { label: 'Intelligence', shortcut: 'Ctrl+L', action: () => alert('Chat') },
      { separator: true },
      { label: 'Theme Mode', action: () => onOpenSettings && onOpenSettings('appearance') },
    ]}
  ];

  function toggleMenu(label) {
    if (activeMenu === label) activeMenu = null;
    else activeMenu = label;
  }

  function handleClickOutside(event) {
    if (menuRef && !menuRef.contains(event.target)) activeMenu = null;
  }

  if (typeof window !== 'undefined') {
    window.addEventListener('click', handleClickOutside);
  }
</script>

<div class="h-8 bg-background/50 backdrop-blur-md border-b border-border/20 flex items-center px-2 select-none z-50 text-[11px]" bind:this={menuRef}>
  {#each menus as menu}
    <div class="relative">
      <button 
        class={cn(
          "px-2.5 py-1 rounded-md transition-all font-bold tracking-tight uppercase",
          activeMenu === menu.label ? "bg-primary/10 text-primary" : "text-muted-foreground/70 hover:text-foreground"
        )}
        onclick={(e) => { e.stopPropagation(); toggleMenu(menu.label); }}
      >
        {menu.label}
      </button>
      
      {#if activeMenu === menu.label}
        <div class="absolute top-[110%] left-0 min-w-[180px] bg-popover/90 backdrop-blur-2xl border border-border/40 rounded-xl shadow-2xl py-1.5 z-[100] animate-in fade-in zoom-in-95 duration-200 origin-top-left ring-1 ring-black/5">
          {#each menu.items as item}
            {#if item.separator}
              <div class="my-1 h-px bg-border/20 mx-2"></div>
            {:else}
              <button
                class="w-full text-left px-3 py-1.5 hover:bg-primary/10 hover:text-primary text-popover-foreground flex items-center justify-between group mx-1 rounded-lg w-[calc(100%-8px)]"
                onclick={() => { item.action(); activeMenu = null; }}
              >
                <span class="font-medium text-xs">{item.label}</span>
                {#if item.shortcut}
                  <span class="text-[8px] font-black text-muted-foreground/30 group-hover:text-primary/40 uppercase tracking-tighter ml-4">{item.shortcut}</span>
                {/if}
              </button>
            {/if}
          {/each}
        </div>
      {/if}
    </div>
  {/each}

  <div class="flex-1"></div>
  
  <div class="flex items-center gap-3 px-3 text-[9px] font-black uppercase tracking-widest text-muted-foreground/40">
     <div class="flex items-center gap-1.5 group cursor-default">
        <div class="w-1 h-1 rounded-full bg-green-500"></div>
        <span class="group-hover:text-green-500/60 transition-colors tracking-tight">Sync Status: Optimal</span>
     </div>
  </div>
</div>
