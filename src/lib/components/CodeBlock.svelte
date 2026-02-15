<script>
  import { invoke } from '@tauri-apps/api/core';
  import { Check, AlertCircle, Loader2 } from 'lucide-svelte';
  
  let { code, language = 'javascript', path, find, replace } = $props();
  
  let applying = $state(false);
  let applied = $state(false);
  let error = $state(null);
  
  async function applyPatch() {
    if (!path || !find || !replace) return;
    
    applying = true;
    error = null;
    
    try {
      const result = await invoke('apply_smart_patch', {
        path,
        find,
        replace
      });
      
      applied = true;
      setTimeout(() => applied = false, 3000);
    } catch (e) {
      error = e.toString();
    } finally {
      applying = false;
    }
  }
</script>

<div class="code-block-wrapper">
  <div class="code-header">
    <span class="language-badge">{language}</span>
    {#if path}
      <span class="file-path">{path}</span>
    {/if}
    {#if find && replace}
      <button 
        class="apply-btn"
        class:applied
        class:error={error}
        onclick={applyPatch}
        disabled={applying || applied}
      >
        {#if applying}
          <Loader2 size={14} class="spin" />
          Applying...
        {:else if applied}
          <Check size={14} />
          Applied
        {:else if error}
          <AlertCircle size={14} />
          Failed
        {:else}
          Apply Patch
        {/if}
      </button>
    {/if}
  </div>
  
  <pre class="code-content"><code class="language-{language}">{code}</code></pre>
  
  {#if error}
    <div class="error-message">
      {error}
    </div>
  {/if}
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
  
  .apply-btn {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.375rem 0.75rem;
    background: var(--accent);
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 0.75rem;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .apply-btn:hover:not(:disabled) {
    opacity: 0.9;
    transform: translateY(-1px);
  }
  
  .apply-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  
  .apply-btn.applied {
    background: #10b981;
  }
  
  .apply-btn.error {
    background: #ef4444;
  }
  
  .code-content {
    margin: 0;
    padding: 1rem;
    overflow-x: auto;
    font-size: 0.875rem;
    line-height: 1.5;
  }
  
  .error-message {
    padding: 0.5rem 0.75rem;
    background: #ef444420;
    color: #ef4444;
    font-size: 0.75rem;
    border-top: 1px solid var(--border);
  }
  
  :global(.spin) {
    animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
