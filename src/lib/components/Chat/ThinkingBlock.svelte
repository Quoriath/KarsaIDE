<script>
  import { onMount, tick } from 'svelte';
  import { cn } from '../../utils.js';
  import { BrainCircuit, ChevronDown, ChevronRight, Loader2 } from 'lucide-svelte';

  let { 
    content = '', 
    isStreaming = false, 
    isThinkingComplete = false,
    isToolExecuting = false 
  } = $props();

  let isExpanded = $state(true);

  $effect(() => {
    if (isStreaming && !isThinkingComplete) {
      isExpanded = true;
    }
  });

  const thinkingClass = $derived(cn(
    "rounded-xl border transition-all duration-300 overflow-hidden mb-2",
    isStreaming && !isThinkingComplete 
      ? "border-purple-500/10 bg-purple-500/5" 
      : "border-border/20 bg-muted/5"
  ));
</script>

<div class={thinkingClass}>
  <button 
    class="w-full flex items-center gap-2 px-3 py-1.5 text-[9px] font-bold uppercase tracking-[0.15em] text-muted-foreground/50 transition-colors hover:bg-muted/10"
    onclick={() => isExpanded = !isExpanded}
  >
    <div class={cn(
      "p-1 rounded-lg transition-colors",
      isStreaming && !isThinkingComplete ? "text-purple-400" : "text-muted-foreground/30"
    )}>
      <BrainCircuit size={11} class={cn(isStreaming && !isThinkingComplete ? "animate-pulse" : "")} />
    </div>

    <span class="flex-1 text-left">
      {isStreaming && !isThinkingComplete ? 'Thinking' : 'Process'}
    </span>

    {#if isToolExecuting}
      <div class="flex items-center gap-1 text-blue-400/60 animate-in fade-in">
        <Loader2 size={8} class="animate-spin" />
        <span class="text-[7px] font-black">TOOL</span>
      </div>
    {/if}

    <div class="opacity-20 ml-1">
      {#if isExpanded} <ChevronDown size={10} /> {:else} <ChevronRight size={10} /> {/if}
    </div>
  </button>
  
  {#if isExpanded}
    <div class="px-3 py-2 text-[10.5px] leading-[1.4] text-muted-foreground/60 font-mono bg-black/[0.02] border-t border-border/5 whitespace-pre-wrap overflow-hidden animate-in slide-in-from-top-1 duration-200">
      {content}
      {#if isStreaming && !isThinkingComplete}
        <span class="inline-block w-1 h-3 bg-purple-400/40 animate-pulse ml-0.5 align-middle rounded-full"></span>
      {/if}
    </div>
  {/if}
</div>
