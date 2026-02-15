<script>
  import { uiState } from '../stores/uiState.svelte.js';
  import { onMount } from 'svelte';

  let menuRef;

  function handleClickOutside(event) {
    if (menuRef && !menuRef.contains(event.target)) {
      uiState.closeContextMenu();
    }
  }

  function handleAction(action) {
    action.fn(uiState.contextMenu.item);
    uiState.closeContextMenu();
  }

  // Adjust position to keep within viewport
  let style = $derived.by(() => {
    let x = uiState.contextMenu.x;
    let y = uiState.contextMenu.y;
    // Simple logic, can be enhanced to detect window bounds
    return `top: ${y}px; left: ${x}px;`;
  });

  $effect(() => {
    if (uiState.contextMenu.visible) {
      window.addEventListener('click', handleClickOutside);
      window.addEventListener('contextmenu', handleClickOutside); // Close if right clicking elsewhere
    } else {
      window.removeEventListener('click', handleClickOutside);
      window.removeEventListener('contextmenu', handleClickOutside);
    }
    return () => {
      window.removeEventListener('click', handleClickOutside);
      window.removeEventListener('contextmenu', handleClickOutside);
    };
  });
</script>

{#if uiState.contextMenu.visible}
  <div 
    bind:this={menuRef}
    class="fixed z-50 min-w-[160px] bg-popover border border-border rounded-lg shadow-xl p-1 animate-in fade-in zoom-in-95 duration-100 flex flex-col"
    style={style}
  >
    {#each uiState.contextMenu.actions as action}
      {#if action.separator}
        <div class="h-[1px] bg-border my-1"></div>
      {:else}
        <button
          class="w-full text-left px-3 py-1.5 text-sm rounded hover:bg-accent hover:text-accent-foreground transition-colors flex items-center gap-2 {action.danger ? 'text-destructive hover:bg-destructive/10' : 'text-popover-foreground'}"
          onclick={() => handleAction(action)}
        >
          {#if action.icon}
            <action.icon size={14} />
          {/if}
          {action.label}
        </button>
      {/if}
    {/each}
  </div>
{/if}
