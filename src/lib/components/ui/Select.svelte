<script>
  import { ChevronDown, Check } from 'lucide-svelte';
  import { cn } from '../../utils.js';

  let { value = $bindable(), options = [], placeholder = "Select...", className = "" } = $props();
  
  let isOpen = $state(false);
  let containerRef;

  function handleSelect(val) {
    value = val;
    isOpen = false;
  }

  function handleClickOutside(event) {
    if (containerRef && !containerRef.contains(event.target)) {
      isOpen = false;
    }
  }

  $effect(() => {
    if (isOpen) window.addEventListener('click', handleClickOutside);
    return () => window.removeEventListener('click', handleClickOutside);
  });

  let selectedLabel = $derived(options.find(o => o.value === value)?.label || value);
</script>

<div class={cn("relative w-full", className)} bind:this={containerRef}>
  <button
    class={cn(
      "w-full flex items-center justify-between px-3 py-2 text-sm rounded-lg border transition-all duration-200",
      isOpen 
        ? "border-primary ring-1 ring-primary/20 bg-primary/5 text-foreground" 
        : "border-border bg-muted/20 hover:bg-muted/40 hover:border-primary/30 text-muted-foreground hover:text-foreground"
    )}
    onclick={() => isOpen = !isOpen}
  >
    <span class="truncate">{selectedLabel || placeholder}</span>
    <ChevronDown size={14} class={cn("transition-transform duration-200 opacity-50", isOpen ? "rotate-180" : "")} />
  </button>

  {#if isOpen}
    <div class="absolute top-full left-0 right-0 mt-1.5 p-1 bg-popover border border-border/60 rounded-xl shadow-xl z-50 animate-in fade-in zoom-in-95 duration-100 overflow-hidden">
      <div class="max-h-[200px] overflow-y-auto scrollbar-thin scrollbar-thumb-muted">
        {#each options as option}
          <button
            class={cn(
              "w-full flex items-center justify-between px-3 py-2 text-sm rounded-lg transition-colors text-left",
              value === option.value 
                ? "bg-primary/10 text-primary font-medium" 
                : "text-popover-foreground hover:bg-muted"
            )}
            onclick={() => handleSelect(option.value)}
          >
            <span>{option.label}</span>
            {#if value === option.value}
              <Check size={14} />
            {/if}
          </button>
        {/each}
      </div>
    </div>
  {/if}
</div>
