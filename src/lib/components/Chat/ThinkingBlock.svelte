<script>
  import { onMount, tick } from 'svelte';
  import { cn } from '../../utils.js';
  import { BrainCircuit, ChevronDown, ChevronRight, Loader2, Sparkles } from 'lucide-svelte';

  let { 
    content = '', 
    isStreaming = false, 
    isThinkingComplete = false,
    isToolExecuting = false 
  } = $props();

  let isExpanded = $state(true);
  let container;

  $effect(() => {
    if (isStreaming && !isThinkingComplete) {
      isExpanded = true;
    }
  });

  const thinkingClass = $derived(cn(
    "rounded-xl border transition-all duration-500 overflow-hidden mb-4 group",
    isStreaming && !isThinkingComplete 
      ? "border-primary/30 bg-primary/5 shadow-lg shadow-primary/5" 
      : "border-border/40 bg-muted/20"
  ));
</script>

<div class={thinkingClass} bind:this={container}>
  <button 
    class="w-full flex items-center gap-2.5 px-4 py-3 text-[11px] font-bold text-muted-foreground transition-colors hover:bg-muted/30 uppercase tracking-widest"
    onclick={() => isExpanded = !isExpanded}
  >
    <div class="relative">
      <div class={cn(
        "p-1.5 rounded-lg transition-colors",
        isStreaming && !isThinkingComplete ? "bg-primary text-primary-foreground shadow-lg shadow-primary/20" : "bg-muted/50 text-muted-foreground/60"
      )}>
        <BrainCircuit size={14} class={cn(isStreaming && !isThinkingComplete ? "animate-pulse" : "")} />
      </div>
      {#if isStreaming && !isThinkingComplete}
        <span class="absolute -top-1 -right-1 flex h-2.5 w-2.5">
          <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-primary opacity-75"></span>
          <span class="relative inline-flex rounded-full h-2.5 w-2.5 bg-primary"></span>
        </span>
      {/if}
    </div>

    <span class="flex-1 text-left">
      {isStreaming && !isThinkingComplete ? 'Consulting Intelligence...' : 'Logic Trace'}
    </span>

    {#if isToolExecuting}
      <div class="flex items-center gap-1.5 text-primary bg-primary/10 px-2.5 py-1 rounded-full border border-primary/20 animate-in fade-in zoom-in-95 shadow-sm">
        <Loader2 size={10} class="animate-spin" />
        <span class="text-[9px] uppercase tracking-wider font-black">Executing</span>
      </div>
    {/if}

    {#if isExpanded}
      <ChevronDown size={14} class="opacity-40 group-hover:opacity-100 transition-opacity" />
    {:else}
      <ChevronRight size={14} class="opacity-40 group-hover:opacity-100 transition-opacity" />
    {/if}
  </button>
  
  {#if isExpanded}
    <div class="px-5 py-4 text-[12px] leading-relaxed text-muted-foreground/90 font-mono bg-black/20 border-t border-border/20 whitespace-pre-wrap overflow-hidden animate-in slide-in-from-top-1 duration-500 relative">
      <!-- Logic pattern background for thinking -->
      {#if isStreaming && !isThinkingComplete}
        <div class="absolute inset-0 opacity-[0.03] pointer-events-none bg-[radial-gradient(#fff_1px,transparent_1px)] bg-[size:10px_10px]"></div>
      {/if}
      <span class="relative z-10">{content}</span>
      {#if isStreaming && !isThinkingComplete}
        <span class="inline-block w-1.5 h-4 bg-primary animate-pulse ml-1 align-middle rounded-sm shadow-[0_0_8px_var(--primary)]"></span>
      {/if}
    </div>
  {/if}
</div>

<style>
  .font-mono {
    font-variant-ligatures: none;
  }
</style>
