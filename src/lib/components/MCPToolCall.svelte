<script>
  import { FileText, Search, List, Code, File, FolderOpen } from 'lucide-svelte';
  
  let { toolName, arguments: args, result, error } = $props();
  
  const toolIcons = {
    file_read: FileText,
    file_read_range: FileText,
    list_files: FolderOpen,
    list_symbols: Code,
    search: Search,
    get_project_map: List,
    get_file_info: File,
  };
  
  const Icon = toolIcons[toolName] || Code;
  
  function formatArgs(args) {
    if (!args || Object.keys(args).length === 0) return '';
    return Object.entries(args)
      .map(([k, v]) => `${k}: ${JSON.stringify(v)}`)
      .join(', ');
  }
  
  function truncateResult(text, maxLength = 300) {
    if (!text) return '';
    const str = typeof text === 'string' ? text : JSON.stringify(text, null, 2);
    if (str.length <= maxLength) return str;
    return str.substring(0, maxLength) + '...';
  }
</script>

<div class="mcp-tool-call border border-border/50 rounded-lg p-3 my-2 bg-muted/10">
  <div class="flex items-start gap-2">
    <div class="mt-0.5 text-primary">
      <Icon size={16} />
    </div>
    <div class="flex-1 min-w-0">
      <div class="font-mono text-xs font-medium text-foreground">
        {toolName}
      </div>
      {#if formatArgs(args)}
        <div class="text-xs text-muted-foreground mt-1 font-mono">
          {formatArgs(args)}
        </div>
      {/if}
      
      {#if error}
        <div class="mt-2 text-xs text-red-500 bg-red-500/10 rounded px-2 py-1">
          ❌ {error}
        </div>
      {:else if result}
        <details class="mt-2">
          <summary class="text-xs text-muted-foreground cursor-pointer hover:text-foreground">
            📄 Result ({typeof result === 'string' ? result.length : JSON.stringify(result).length} chars)
          </summary>
          <pre class="mt-2 text-xs bg-background/50 rounded p-2 overflow-x-auto border border-border/30">{truncateResult(result, 500)}</pre>
        </details>
      {/if}
    </div>
  </div>
</div>
