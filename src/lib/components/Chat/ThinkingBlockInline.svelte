<script lang="ts">
  import { Brain } from 'lucide-svelte';
  
  interface Props {
    content: string;
    timestamp?: number;
  }
  
  let { content, timestamp }: Props = $props();
  let expanded = $state(false);
  
  // Parse thinking content - remove markdown formatting
  const cleanContent = content
    .replace(/^#+\s*/gm, '') // Remove headers
    .replace(/\*\*/g, '')     // Remove bold
    .replace(/\*/g, '')       // Remove italic
    .trim();
  
  const lines = cleanContent.split('\n').filter(l => l.trim());
  const preview = lines[0] || 'Thinking...';
</script>

<div class="thinking-block">
  <button 
    class="thinking-header"
    onclick={() => expanded = !expanded}
  >
    <Brain size={14} class="thinking-icon" />
    <span class="thinking-preview">{preview}</span>
    <span class="thinking-toggle">{expanded ? '−' : '+'}</span>
  </button>
  
  {#if expanded}
    <div class="thinking-content">
      {#each lines as line}
        <div class="thinking-line">{line}</div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .thinking-block {
    margin: 8px 0;
    border: 1px solid var(--border-color, #3a3a3a);
    border-radius: 6px;
    background: var(--thinking-bg, #1e1e1e);
    font-size: 13px;
  }
  
  .thinking-header {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: transparent;
    border: none;
    color: var(--text-secondary, #888);
    cursor: pointer;
    transition: background 0.2s;
  }
  
  .thinking-header:hover {
    background: var(--hover-bg, #2a2a2a);
  }
  
  .thinking-icon {
    flex-shrink: 0;
    color: var(--accent-color, #007acc);
  }
  
  .thinking-preview {
    flex: 1;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-style: italic;
  }
  
  .thinking-toggle {
    flex-shrink: 0;
    font-size: 16px;
    font-weight: bold;
  }
  
  .thinking-content {
    padding: 0 12px 12px 12px;
    color: var(--text-primary, #ccc);
    line-height: 1.5;
  }
  
  .thinking-line {
    padding: 2px 0;
  }
  
  .thinking-line:empty {
    display: none;
  }
</style>
