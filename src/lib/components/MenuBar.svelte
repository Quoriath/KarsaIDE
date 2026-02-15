<script>
  import { fsStore } from '../stores/fileSystem.svelte.js';
  import { themeStore } from '../stores/theme.svelte.js';
  import { cn } from '../utils.js';
  import { Menu, ChevronDown, Check } from 'lucide-svelte';

  let { onOpenSettings } = $props();

  let activeMenu = $state(null);
  let menuRef;

  const menus = [
    { label: 'File', items: [
      { label: 'New File', action: () => alert('New File') },
      { label: 'Open File...', action: () => fsStore.openFile() },
      { label: 'Open Folder...', action: () => fsStore.setProjectDir() },
      { label: 'Save', action: () => fsStore.saveActiveFile() },
      { label: 'Settings', action: () => onOpenSettings && onOpenSettings() },
      { label: 'Exit', action: () => window.close() },
    ]},
    { label: 'Edit', items: [
      { label: 'Undo', action: () => alert('Undo') },
      { label: 'Redo', action: () => alert('Redo') },
      { label: 'Cut', action: () => alert('Cut') },
      { label: 'Copy', action: () => alert('Copy') },
      { label: 'Paste', action: () => alert('Paste') },
    ]},
    { label: 'View', items: [
      { label: 'Toggle Sidebar', action: () => alert('Toggle Sidebar') },
      { label: 'Toggle Chat', action: () => alert('Toggle Chat') },
      { label: 'Zoom In', action: () => alert('Zoom In') },
      { label: 'Zoom Out', action: () => alert('Zoom Out') },
    ]},
    { label: 'Themes', items: [
      { label: 'Karsa Dark', action: () => themeStore.setTheme('karsa-dark'), check: () => themeStore.activeTheme === 'karsa-dark' },
      { label: 'Dracula', action: () => themeStore.setTheme('dracula'), check: () => themeStore.activeTheme === 'dracula' },
      { label: 'GitHub Light', action: () => themeStore.setTheme('github-light'), check: () => themeStore.activeTheme === 'github-light' },
    ]},
    { label: 'Help', items: [
      { label: 'Welcome', action: () => alert('Welcome') },
      { label: 'Documentation', action: () => alert('Documentation') },
      { label: 'About', action: () => alert('About') },
    ]},
  ];

  function toggleMenu(label) {
    if (activeMenu === label) {
      activeMenu = null;
    } else {
      activeMenu = label;
    }
  }

  function handleClickOutside(event) {
    if (menuRef && !menuRef.contains(event.target)) {
      activeMenu = null;
    }
  }

  // Close menu when clicking outside
  if (typeof window !== 'undefined') {
    window.addEventListener('click', handleClickOutside);
  }
</script>

<div class="h-8 bg-background border-b border-border flex items-center px-2 select-none z-50 text-xs" bind:this={menuRef}>
  <!-- App Icon/Menu (Optional) -->
  <div class="px-2 py-1 mr-2 text-primary font-bold">Karsa</div>

  {#each menus as menu}
    <div class="relative">
      <button 
        class={cn(
          "px-3 py-1.5 rounded hover:bg-muted text-muted-foreground hover:text-foreground transition-colors",
          activeMenu === menu.label ? "bg-muted text-foreground" : ""
        )}
        onclick={(e) => { e.stopPropagation(); toggleMenu(menu.label); }}
      >
        {menu.label}
      </button>
      
      {#if activeMenu === menu.label}
        <div class="absolute top-full left-0 min-w-[200px] bg-popover border border-border rounded-md shadow-lg py-1 z-50 animate-in fade-in zoom-in-95 duration-100 origin-top-left">
          {#each menu.items as item}
            <button
              class="w-full text-left px-4 py-1.5 hover:bg-accent hover:text-accent-foreground text-popover-foreground flex items-center justify-between group"
              onclick={() => { item.action(); activeMenu = null; }}
            >
              <span>{item.label}</span>
              {#if item.check && item.check()}
                <Check size={12} class="text-primary" />
              {/if}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/each}

  <div class="flex-1"></div>
  
  <!-- Right side controls (optional) -->
  <div class="flex items-center gap-2 px-2 text-muted-foreground">
     <span class="text-[10px] opacity-50">v0.1.0-alpha</span>
  </div>
</div>
