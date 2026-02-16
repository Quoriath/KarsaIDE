<script>
  import { fsStore } from '../stores/fileSystem.svelte.js';
  import { themeStore } from '../stores/theme.svelte.js';
  import { cn } from '../utils.js';
  import { Menu, ChevronDown, Check, FolderOpen, Save, Settings as SettingsIcon, LogOut, FileCode } from 'lucide-svelte';

  let { onOpenSettings } = $props();

  let activeMenu = $state(null);
  let menuRef;

  const menus = [
    { label: 'File', items: [
      { label: 'New File', icon: FileCode, action: () => alert('New File') },
      { label: 'Open Folder...', icon: FolderOpen, action: () => fsStore.setProjectDir() },
      { label: 'Save', icon: Save, action: () => fsStore.saveActiveFile() },
      { separator: true },
      { label: 'Settings', icon: SettingsIcon, action: () => onOpenSettings && onOpenSettings() },
      { label: 'Exit', icon: LogOut, action: () => window.close() },
    ]},
    { label: 'Edit', items: [
      { label: 'Undo', action: () => alert('Undo') },
      { label: 'Redo', action: () => alert('Redo') },
      { separator: true },
      { label: 'Cut', action: () => alert('Cut') },
      { label: 'Copy', action: () => alert('Copy') },
      { label: 'Paste', action: () => alert('Paste') },
    ]},
    { label: 'View', items: [
      { label: 'Toggle Sidebar', action: () => alert('Toggle Sidebar') },
      { label: 'Toggle Chat', action: () => alert('Toggle Chat') },
      { separator: true },
      { label: 'Reset Zoom', action: () => alert('Reset Zoom') },
    ]},
    { label: 'Help', items: [
      { label: 'Documentation', action: () => alert('Documentation') },
      { label: 'Changelog', action: () => alert('Changelog') },
      { separator: true },
      { label: 'About Karsa', action: () => alert('About') },
    ]},
  ];

  function toggleMenu(label) {
    activeMenu = activeMenu === label ? null : label;
  }

  function handleClickOutside(event) {
    if (menuRef && !menuRef.contains(event.target)) {
      activeMenu = null;
    }
  }

  if (typeof window !== 'undefined') {
    window.addEventListener('click', handleClickOutside);
  }
</script>

<div class="h-9 bg-background/50 backdrop-blur-md border-b border-border/40 flex items-center px-3 select-none z-50 transition-all duration-300" bind:this={menuRef}>
  <div class="flex items-center gap-1">
    {#each menus as menu}
      <div class="relative">
        <button 
          class={cn(
            "px-3 py-1.5 rounded-lg text-[11.5px] font-medium transition-all duration-200",
            activeMenu === menu.label 
              ? "bg-primary/10 text-primary" 
              : "text-muted-foreground/70 hover:bg-muted/50 hover:text-foreground"
          )}
          onclick={(e) => { e.stopPropagation(); toggleMenu(menu.label); }}
        >
          {menu.label}
        </button>
        
        {#if activeMenu === menu.label}
          <div class="absolute top-full left-0 mt-1.5 min-w-[200px] bg-background/95 backdrop-blur-xl border border-border/60 rounded-xl shadow-2xl py-1.5 z-[100] animate-in fade-in zoom-in-95 duration-200 origin-top-left">
            {#each menu.items as item}
              {#if item.separator}
                <div class="h-[1px] bg-border/40 my-1 mx-2"></div>
              {:else}
                <button
                  class="w-full text-left px-3 py-1.5 hover:bg-primary/10 hover:text-primary text-[11.5px] flex items-center gap-3 group transition-colors mx-1 rounded-md w-[calc(100%-8px)]"
                  onclick={() => { item.action(); activeMenu = null; }}
                >
                  {#if item.icon}
                    <item.icon size={14} class="opacity-50 group-hover:opacity-100" />
                  {:else}
                    <div class="w-3.5"></div>
                  {/if}
                  <span class="flex-1 font-medium">{item.label}</span>
                </button>
              {/if}
            {/each}
          </div>
        {/if}
      </div>
    {/each}
  </div>

  <div class="flex-1"></div>
  
  <!-- Status indicators -->
  <div class="flex items-center gap-4 px-3">
     <div class="flex items-center gap-1.5 opacity-40 hover:opacity-100 transition-opacity">
        <span class="text-[9px] font-bold tracking-widest uppercase">Stability:</span>
        <span class="text-[9px] font-bold text-green-500 uppercase">Alpha</span>
     </div>
  </div>
</div>
