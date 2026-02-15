<script>
  import { BrainCircuit, ChevronDown, ChevronRight } from 'lucide-svelte';
  import { cn } from '../../utils.js';

  let { content = '', isStreaming = false } = $props();
  let expanded = $state(true); // Default expanded while thinking
  
  // Auto collapse when done streaming
  $effect(() => {
    if (!isStreaming) expanded = false;
  });
</script>

<div class="my-2 border border-purple-500/20 bg-purple-500/5 rounded-lg overflow-hidden">
  <button 
    class="w-full flex items-center gap-2 px-3 py-2 text-xs font-medium text-purple-400 hover:bg-purple-500/10 transition-colors"
    onclick={() => expanded = !expanded}
  >
    <BrainCircuit size={14} class={cn(isStreaming ? "animate-pulse" : "")} />
    <span>Thinking Process</span>
    {#if isStreaming}
      <span class="flex gap-0.5 ml-1">
        <span class="w-1 h-1 bg-purple-400 rounded-full animate-bounce"></span>
        <span class="w-1 h-1 bg-purple-400 rounded-full animate-bounce delay-150"></span>
        <span class="w-1 h-1 bg-purple-400 rounded-full animate-bounce delay-300"></span>
      </span>
    {/if}
    <div class="ml-auto opacity-50">
      {#if expanded} <ChevronDown size={14} /> {:else} <ChevronRight size={14} /> {/if}
    </div>
  </button>

  {#if expanded}
    <div class="px-3 py-2.5 text-xs text-muted-foreground/80 font-mono bg-black/10 border-t border-purple-500/10 whitespace-pre-wrap leading-relaxed animate-in slide-in-from-top-1">
      {content}
      {#if isStreaming}
        <span class="inline-block w-1.5 h-3 bg-purple-400 animate-pulse ml-0.5 align-middle"></span>
      {/if}
    </div>
  {/if}
</div>
