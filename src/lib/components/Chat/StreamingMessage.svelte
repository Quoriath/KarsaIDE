<script>
  import { onMount, onDestroy } from 'svelte';
  import MarkdownRenderer from '../MarkdownRenderer.svelte';
  import { ChevronDown, ChevronRight, BrainCircuit } from 'lucide-svelte';
  import { cn } from '../../utils.js';

  let { content = '', isThinking = false, onComplete } = $props();

  let displayedContent = $state('');
  let targetContent = $state('');
  let typingInterval;
  let isTyping = $state(false);
  let isExpanded = $state(true); // For thinking block

  // Separate thinking content from main content if using <think> tags
  let thinkContent = $derived(content.match(/<think>([\s\S]*?)<\/think>/)?.[1] || '');
  let mainContent = $derived(content.replace(/<think>[\s\S]*?<\/think>/, '').trim());
  
  // If explicitly thinking mode (before streaming real answer)
  let activeContent = $derived(isThinking ? content : mainContent);

  $effect(() => {
    // When content updates from parent, update target
    if (activeContent !== targetContent) {
      targetContent = activeContent;
      startTyping();
    }
  });

  function startTyping() {
    if (isTyping) return;
    isTyping = true;

    clearInterval(typingInterval);
    
    // Adaptive speed: faster for long text, slower for short burst
    typingInterval = setInterval(() => {
      if (displayedContent.length < targetContent.length) {
        const nextChar = targetContent[displayedContent.length];
        displayedContent += nextChar;
        
        // Auto scroll handling should be done by parent observing size changes
      } else {
        isTyping = false;
        clearInterval(typingInterval);
        onComplete?.();
      }
    }, 5); // 5ms per char is very fast but smooth
  }

  onDestroy(() => {
    clearInterval(typingInterval);
  });
</script>

<div class="space-y-2 w-full max-w-full overflow-hidden">
  <!-- Thinking Block -->
  {#if thinkContent || isThinking}
    <div class="rounded-lg border border-border/50 bg-muted/30 overflow-hidden">
      <button 
        class="w-full flex items-center gap-2 px-3 py-2 text-xs font-medium text-muted-foreground hover:bg-muted/50 transition-colors"
        onclick={() => isExpanded = !isExpanded}
      >
        <BrainCircuit size={14} class={cn("text-purple-400", isThinking ? "animate-pulse" : "")} />
        <span>{isThinking ? 'Thinking...' : 'Thought Process'}</span>
        {#if isExpanded}
          <ChevronDown size={14} class="ml-auto opacity-50" />
        {:else}
          <ChevronRight size={14} class="ml-auto opacity-50" />
        {/if}
      </button>
      
      {#if isExpanded}
        <div class="px-3 py-2 text-xs text-muted-foreground/80 font-mono bg-black/5 border-t border-border/30 whitespace-pre-wrap animate-in slide-in-from-top-1 duration-200">
          {isThinking ? displayedContent : thinkContent}
          {#if isThinking}
             <span class="inline-block w-1.5 h-3 bg-purple-400 animate-pulse ml-0.5 align-middle"></span>
          {/if}
        </div>
      {/if}
    </div>
  {/if}

  <!-- Main Content -->
  {#if !isThinking && targetContent}
    <div class="markdown-content">
      <MarkdownRenderer content={displayedContent} />
      {#if isTyping}
         <span class="inline-block w-2 h-4 bg-primary animate-pulse ml-0.5 align-middle rounded-sm"></span>
      {/if}
    </div>
  {/if}
</div>
