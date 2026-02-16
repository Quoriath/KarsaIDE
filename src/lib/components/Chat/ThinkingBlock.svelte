<script>
  import { Brain, ChevronDown } from 'lucide-svelte';

  let { content = '', isStreaming = false } = $props();
  let expanded = $state(true);
  
  // Auto collapse when done streaming
  $effect(() => {
    if (!isStreaming && content) {
      const timer = setTimeout(() => {
        expanded = false;
      }, 500);
      return () => clearTimeout(timer);
    }
  });
</script>

{#if content}
  <div class="thinking-block" class:streaming={isStreaming}>
    <button 
      class="thinking-header"
      onclick={() => expanded = !expanded}
    >
      <div class="thinking-icon">
        <Brain size={14} />
      </div>
      <span class="thinking-label">
        {isStreaming ? 'Thinking...' : 'Thought'}
      </span>
      <ChevronDown 
        size={14} 
        class="thinking-chevron {expanded ? 'expanded' : ''}"
      />
    </button>

    {#if expanded}
      <div class="thinking-content">
        <div class="thinking-text">
          {content}
          {#if isStreaming}
            <span class="thinking-cursor"></span>
          {/if}
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .thinking-block {
    margin: 0.5rem 0;
    border-radius: 0.5rem;
    border: 1px solid hsl(var(--border) / 0.5);
    background: hsl(var(--muted) / 0.2);
    overflow: hidden;
  }
  
  .thinking-block.streaming {
    border-color: hsl(var(--primary) / 0.3);
  }
  
  .thinking-header {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    font-size: 0.75rem;
    color: hsl(var(--muted-foreground));
    transition: background 0.15s;
  }
  
  .thinking-header:hover {
    background: hsl(var(--muted) / 0.3);
  }
  
  .thinking-icon {
    display: flex;
    align-items: center;
    color: hsl(var(--primary) / 0.6);
  }
  
  .thinking-block.streaming .thinking-icon {
    animation: pulse 2s infinite;
  }
  
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
  
  .thinking-label {
    font-weight: 500;
  }
  
  .thinking-chevron {
    margin-left: auto;
    transition: transform 0.2s;
    opacity: 0.5;
  }
  
  .thinking-chevron.expanded {
    transform: rotate(180deg);
  }
  
  .thinking-content {
    border-top: 1px solid hsl(var(--border) / 0.3);
    background: hsl(var(--background) / 0.5);
    animation: slideDown 0.2s ease-out;
  }
  
  @keyframes slideDown {
    from { opacity: 0; max-height: 0; }
    to { opacity: 1; max-height: 500px; }
  }
  
  .thinking-text {
    padding: 0.75rem;
    font-size: 0.6875rem;
    font-family: ui-monospace, monospace;
    line-height: 1.5;
    color: hsl(var(--muted-foreground));
    white-space: pre-wrap;
    word-break: break-word;
  }
  
  .thinking-cursor {
    display: inline-block;
    width: 2px;
    height: 0.75rem;
    background: hsl(var(--primary) / 0.5);
    margin-left: 1px;
    animation: blink 1s infinite;
    vertical-align: text-bottom;
  }
  
  @keyframes blink {
    0%, 50% { opacity: 1; }
    51%, 100% { opacity: 0; }
  }
</style>
