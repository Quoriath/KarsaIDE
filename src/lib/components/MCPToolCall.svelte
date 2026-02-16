<script>
  import { 
    FileText, Search, List, Code, File, FolderOpen, 
    Loader2, CheckCircle2, XCircle, ChevronDown 
  } from 'lucide-svelte';
  import { cn } from '../utils.js';

  let { toolName, arguments: args, result, error, executing = false } = $props();
  
  let expanded = $state(false);
  
  const toolIcons = {
    file_read: FileText,
    file_read_range: FileText,
    file_read_around: FileText,
    list_files: FolderOpen,
    list_symbols: Code,
    search: Search,
    search_advanced: Search,
    get_project_map: List,
    get_file_info: File,
    get_dependencies: List,
    file_write: FileText,
    file_append: FileText,
    file_delete: FileText,
    file_move: FileText,
    file_copy: FileText,
    create_directory: FolderOpen,
    smart_patch: Code,
    analyze_file: Code,
    git_status: FileText,
    git_diff: FileText,
    find_in_file: Search,
    extract_code_block: Code,
  };
  
  const Icon = toolIcons[toolName] || Code;
  
  function formatArgs(args) {
    if (!args || Object.keys(args).length === 0) return '';
    return Object.entries(args)
      .map(([k, v]) => `${k}=${typeof v === 'string' ? v : JSON.stringify(v)}`)
      .join(', ');
  }
  
  const status = $derived(
    executing ? 'running' : 
    error ? 'error' : 
    result ? 'done' : 'pending'
  );
  
  const resultPreview = $derived(() => {
    if (!result) return '';
    const str = typeof result === 'string' ? result : JSON.stringify(result, null, 2);
    if (str.length > 600) return str.substring(0, 600) + '\n...';
    return str;
  });
</script>

<div class="tool-block" class:running={executing}>
  <button 
    class="tool-header"
    onclick={() => expanded = !expanded}
  >
    <!-- Status Icon -->
    <div class="tool-status {status}">
      {#if executing}
        <Loader2 size={14} class="spin" />
      {:else if error}
        <XCircle size={14} />
      {:else if result}
        <CheckCircle2 size={14} />
      {:else}
        <Icon size={14} />
      {/if}
    </div>
    
    <!-- Tool Info -->
    <div class="tool-info">
      <span class="tool-name">{toolName}</span>
      {#if args && Object.keys(args).length > 0}
        <span class="tool-args">{formatArgs(args)}</span>
      {/if}
    </div>
    
    <!-- Status Label -->
    <span class="tool-status-label {status}">
      {#if executing}
        Running...
      {:else if error}
        Error
      {:else if result}
        Done
      {:else}
        Pending
      {/if}
    </span>
    
    <ChevronDown size={14} class="tool-chevron {expanded ? 'expanded' : ''}" />
  </button>

  {#if expanded}
    <div class="tool-content">
      <!-- Arguments -->
      <div class="tool-section">
        <div class="tool-section-label">Arguments</div>
        <pre class="tool-code">{JSON.stringify(args || {}, null, 2)}</pre>
      </div>
      
      <!-- Result -->
      {#if result || error}
        <div class="tool-section">
          <div class="tool-section-label">{error ? 'Error' : 'Result'}</div>
          <pre class="tool-code" class:error>
            {error || resultPreview()}
          </pre>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .tool-block {
    margin: 0.5rem 0;
    border-radius: 0.5rem;
    border: 1px solid hsl(var(--border) / 0.6);
    background: hsl(var(--card));
    overflow: hidden;
    transition: border-color 0.2s;
  }
  
  .tool-block.running {
    border-color: hsl(var(--primary) / 0.4);
  }
  
  .tool-header {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 0.625rem;
    padding: 0.625rem 0.75rem;
    text-align: left;
    transition: background 0.15s;
  }
  
  .tool-header:hover {
    background: hsl(var(--muted) / 0.4);
  }
  
  .tool-status {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1.5rem;
    height: 1.5rem;
    border-radius: 0.375rem;
    flex-shrink: 0;
  }
  
  .tool-status.running {
    background: hsl(var(--primary) / 0.15);
    color: hsl(var(--primary));
  }
  
  .tool-status.done {
    background: hsl(142 76% 36% / 0.15);
    color: hsl(142 76% 36%);
  }
  
  .tool-status.error {
    background: hsl(var(--destructive) / 0.15);
    color: hsl(var(--destructive));
  }
  
  .tool-status.pending {
    background: hsl(var(--muted) / 0.4);
    color: hsl(var(--muted-foreground));
  }
  
  .spin {
    animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
  
  .tool-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }
  
  .tool-name {
    font-size: 0.75rem;
    font-weight: 600;
    font-family: ui-monospace, monospace;
    color: hsl(var(--foreground));
  }
  
  .tool-args {
    font-size: 0.625rem;
    color: hsl(var(--muted-foreground));
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  
  .tool-status-label {
    font-size: 0.625rem;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.025em;
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    flex-shrink: 0;
  }
  
  .tool-status-label.running {
    background: hsl(var(--primary) / 0.15);
    color: hsl(var(--primary));
  }
  
  .tool-status-label.done {
    background: hsl(142 76% 36% / 0.15);
    color: hsl(142 76% 36%);
  }
  
  .tool-status-label.error {
    background: hsl(var(--destructive) / 0.15);
    color: hsl(var(--destructive));
  }
  
  .tool-status-label.pending {
    background: hsl(var(--muted) / 0.3);
    color: hsl(var(--muted-foreground));
  }
  
  .tool-chevron {
    flex-shrink: 0;
    color: hsl(var(--muted-foreground));
    opacity: 0.6;
    transition: transform 0.2s;
  }
  
  .tool-chevron.expanded {
    transform: rotate(180deg);
  }
  
  .tool-content {
    border-top: 1px solid hsl(var(--border) / 0.5);
    background: hsl(var(--background) / 0.5);
    animation: slideDown 0.2s ease-out;
  }
  
  @keyframes slideDown {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }
  
  .tool-section {
    padding: 0.625rem 0.75rem;
    border-bottom: 1px solid hsl(var(--border) / 0.3);
  }
  
  .tool-section:last-child {
    border-bottom: none;
  }
  
  .tool-section-label {
    font-size: 0.625rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: hsl(var(--muted-foreground));
    margin-bottom: 0.375rem;
  }
  
  .tool-code {
    font-size: 0.6875rem;
    font-family: ui-monospace, monospace;
    line-height: 1.5;
    color: hsl(var(--foreground) / 0.8);
    background: hsl(var(--muted) / 0.3);
    padding: 0.5rem;
    border-radius: 0.375rem;
    margin: 0;
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-word;
    max-height: 12rem;
  }
  
  .tool-code.error {
    color: hsl(var(--destructive));
    background: hsl(var(--destructive) / 0.1);
  }
</style>
