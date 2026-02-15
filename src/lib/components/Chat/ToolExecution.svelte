<script>
  import { 
    CheckCircle2, XCircle, Loader2, Terminal, FileText, FolderOpen, Search, ChevronDown, ChevronRight, Code2 
  } from 'lucide-svelte';
  import { cn } from '../../utils.js';

  let { toolName, args, result, status = 'pending', error } = $props(); 
  // status: 'pending' | 'success' | 'error'

  let expanded = $state(false);

  const icons = {
    'read_file': FileText,
    'write_file': Code2,
    'list_directory': FolderOpen,
    'search': Search,
    'execute_command': Terminal,
    'default': Terminal
  };

  const Icon = $derived(icons[toolName] || icons['default']);
  
  // Format args for display
  const argsString = $derived(
    typeof args === 'string' ? args : JSON.stringify(args, null, 2)
  );
</script>

<div class="my-2 rounded-lg border bg-card/50 overflow-hidden transition-all duration-200 {status === 'error' ? 'border-destructive/30' : 'border-border'}">
  <button 
    class="w-full flex items-center gap-3 px-3 py-2.5 text-xs hover:bg-muted/30 transition-colors group"
    onclick={() => expanded = !expanded}
  >
    <!-- Status Icon -->
    <div class={cn(
      "p-1 rounded-md shrink-0",
      status === 'pending' ? "bg-blue-500/10 text-blue-500" :
      status === 'success' ? "bg-green-500/10 text-green-500" :
      "bg-destructive/10 text-destructive"
    )}>
      {#if status === 'pending'}
        <Loader2 size={14} class="animate-spin" />
      {:else if status === 'success'}
        <CheckCircle2 size={14} />
      {:else}
        <XCircle size={14} />
      {/if}
    </div>

    <!-- Tool Info -->
    <div class="flex-1 flex flex-col items-start min-w-0">
      <div class="flex items-center gap-2 font-medium text-foreground">
        <Icon size={12} class="opacity-70" />
        <span class="font-mono text-primary/90">{toolName}</span>
      </div>
      <div class="text-[10px] text-muted-foreground truncate w-full font-mono mt-0.5 opacity-70">
        {argsString.slice(0, 60)}{argsString.length > 60 ? '...' : ''}
      </div>
    </div>

    <!-- Toggle -->
    <div class="text-muted-foreground opacity-0 group-hover:opacity-100 transition-opacity">
      {#if expanded} <ChevronDown size={14} /> {:else} <ChevronRight size={14} /> {/if}
    </div>
  </button>

  {#if expanded}
    <div class="border-t border-border/50 bg-black/20 animate-in slide-in-from-top-1">
      <!-- Input -->
      <div class="px-3 py-2 border-b border-border/30">
        <div class="text-[9px] uppercase tracking-wider text-muted-foreground mb-1 font-semibold">Input</div>
        <pre class="text-xs font-mono text-muted-foreground overflow-x-auto p-2 bg-black/20 rounded border border-border/20">{argsString}</pre>
      </div>

      <!-- Output -->
      {#if status !== 'pending'}
        <div class="px-3 py-2">
          <div class="text-[9px] uppercase tracking-wider text-muted-foreground mb-1 font-semibold flex justify-between">
             <span>{status === 'error' ? 'Error' : 'Output'}</span>
             {#if result && result.length > 500}
               <span class="text-[9px] opacity-50">Truncated</span>
             {/if}
          </div>
          <pre class={cn(
            "text-xs font-mono overflow-x-auto p-2 rounded border max-h-40 scrollbar-thin",
            status === 'error' ? "text-destructive bg-destructive/5 border-destructive/20" : "text-green-400/80 bg-green-500/5 border-green-500/10"
          )}>{error || (typeof result === 'object' ? JSON.stringify(result, null, 2) : result)}</pre>
        </div>
      {/if}
    </div>
  {/if}
</div>
