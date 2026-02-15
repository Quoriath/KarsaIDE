<script>
  import { invoke } from '@tauri-apps/api/core';
  import { Check, Loader2 } from 'lucide-svelte';
  import { onMount } from 'svelte';
  
  let { code, language = 'javascript', path, find, replace, autoApply = true } = $props();
  
  let status = $state('idle'); // idle, applying, synced
  
  onMount(async () => {
    // Auto-apply for code patches
    if (autoApply && path && find && replace) {
      await applyPatch();
    }
  });
  
  async function applyPatch() {
    status = 'applying';
    
    try {
      await invoke('apply_smart_patch', {
        path,
        find,
        replace
      });
      
      status = 'synced';
      setTimeout(() => status = 'idle', 2000);
    } catch (e) {
      console.error('Patch failed:', e);
      status = 'idle';
    }
  }
</script>

<div class="code-block-wrapper">
  <div class="code-header">
    <span class="language-badge">{language}</span>
    {#if path}
      <span class="file-path">{path}</span>
    {/if}
    {#if status !== 'idle'}
      <div class="status-indicator" class:synced={status === 'synced'}>
        {#if status === 'applying'}
          <Loader2 size={12} class="spin" />
          Applying...
        {:else}
          <Check size={12} />
          Synced
        {/if}
      </div>
    {/if}
  </div>
  
  <pre class="code-content"><code class="language-{language}">{code}</code></pre>
</div>

<style>
  .code-block-wrapper {
    margin: 0.5rem 0;
    border: 1px solid var(--border);
    border-radius: 6px;
    overflow: hidden;
    background: var(--bg-secondary);
  }
  
  .code-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    background: var(--bg-primary);
    border-bottom: 1px solid var(--border);
    font-size: 0.75rem;
  }
  
  .language-badge {
    padding: 0.125rem 0.5rem;
    background: var(--accent);
    color: white;
    border-radius: 3px;
    font-weight: 500;
  }
  
  .file-path {
    color: var(--text-secondary);
    font-family: monospace;
  }
  
  .status-indicator {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.25rem 0.5rem;
    background: var(--accent);
    color: white;
    border-radius: 3px;
    font-size: 0.7rem;
    animation: fadeIn 0.2s;
  }
  
  .status-indicator.synced {
    background: #10b981;
  }
  
  .code-content {
    margin: 0;
    padding: 1rem;
    overflow-x: auto;
    font-size: 0.875rem;
    line-height: 1.5;
  }
  
  :global(.spin) {
    animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
  
  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(-2px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
